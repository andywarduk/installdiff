use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::num::ParseIntError;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use crate::packageman::PackageFile;

pub fn get_rpm_dump(rpm: &OsStr, rpm_elem: usize) -> Result<Vec<PackageFile>, Box<dyn Error>> {
    // Run rpm -q --dump to get list of rpm files
    let output = Command::new("rpm")
        .arg("-q")
        .arg("--dump")
        .arg(rpm)
        .output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        Err(format!(
            "rpm package dump for {} returned {}",
            rpm.to_string_lossy(),
            output.status
        ))?
    }

    // Return rpm dump details
    let rpm_files = output
        .stdout
        .split(|c| *c == 0x0a)
        .filter(|line| !line.is_empty() && line[0] != b'(') // Handle '(contains no files)'
        .map(|line| parse_line(rpm_elem, line))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    Ok(rpm_files)
}

fn parse_line(rpm_elem: usize, line: &[u8]) -> Result<PackageFile, Box<dyn Error>> {
    // Terms are:
    //   File name (may contain spaces grr)
    //   File size
    //   Last modified date (seconds since 01/01/1970)
    //   Checksum (MD5/SHA256)
    //   File mode
    //   Owner
    //   Group
    //   Config file 0/1
    //   Documentation file 0/1
    //   Major/minor device number
    //   Link target, or X (may contain spaces grr)

    // Find spaces in the line
    let spcpos = line
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == b' ')
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    let term_cnt = spcpos.len() + 1;

    let get_term = |i: usize| -> &[u8] {
        let start = if i == 0 { 0 } else { spcpos[i - 1] + 1 };

        let end = if i == spcpos.len() {
            line.len()
        } else {
            spcpos[i]
        };

        &line[start..end]
    };

    // Break out file name
    let path = PathBuf::from(OsString::from_vec((line[..spcpos[term_cnt - 11]]).to_vec()));

    let size_slice = get_term(term_cnt - 10);
    let size = str::from_utf8(size_slice)?.parse::<usize>().map_err(|e| {
        format!(
            "Failed to parse size '{}': {e}",
            String::from_utf8_lossy(size_slice)
        )
    })?;

    let chksum = decode_hex(str::from_utf8(get_term(term_cnt - 8))?)?;

    let mode_slice = get_term(term_cnt - 7);
    let mode = u32::from_str_radix(str::from_utf8(mode_slice)?, 8).map_err(|e| {
        format!(
            "Failed to parse file mode in '{}': {e}",
            String::from_utf8_lossy(mode_slice)
        )
    })?;

    Ok(PackageFile {
        package: rpm_elem,
        path,
        size,
        mode,
        chksum,
    })
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
