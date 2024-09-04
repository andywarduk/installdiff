use std::error::Error;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use unix_mode::is_file;

use crate::packageman::{decode_hex, PackageFile};

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
    let path = PathBuf::from(OsStr::from_bytes(&line[..spcpos[term_cnt - 11]]));

    // Get mode
    let mode_slice = get_term(term_cnt - 7);

    let mode = u32::from_str_radix(str::from_utf8(mode_slice)?, 8).map_err(|e| {
        format!(
            "Failed to parse file mode in '{}': {e}",
            String::from_utf8_lossy(mode_slice)
        )
    })?;

    let (size, chksum) = if is_file(mode) {
        // Get size
        let size_slice = get_term(term_cnt - 10);

        let size = Some(str::from_utf8(size_slice)?.parse::<usize>().map_err(|e| {
            format!(
                "Failed to parse size '{}': {e}",
                String::from_utf8_lossy(size_slice)
            )
        })?);

        // Get checksum
        let chksum_str = str::from_utf8(get_term(term_cnt - 8))?;

        let chksum = if chksum_str.chars().any(|c| c != '0') {
            Some(decode_hex(chksum_str)?)
        } else {
            None
        };

        (size, chksum)
    } else {
        (None, None)
    };

    // Get time
    let time_slice = get_term(term_cnt - 9);

    let time = Some(str::from_utf8(time_slice)?.parse::<i64>().map_err(|e| {
        format!(
            "Failed to parse time '{}': {e}",
            String::from_utf8_lossy(time_slice)
        )
    })?);

    Ok(PackageFile {
        path,
        package: Some(rpm_elem),
        size,
        mode: Some(mode),
        chksum,
        time,
    })
}
