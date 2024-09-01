use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::canonicalize;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use unix_mode::is_symlink;

#[derive(Debug, Clone)]
pub struct RpmFile {
    pub rpm: usize,
    pub path: PathBuf,
    pub size: usize,
    pub mode: u32,
    pub chksum: String,
}

pub fn get_rpm_dump(rpm: &OsStr, rpm_elem: usize) -> Result<Vec<RpmFile>, Box<dyn Error>> {
    // Run rpm -q --dump to get list of rpm files
    let output = Command::new("rpm")
        .arg("-q")
        .arg("--dump")
        .arg(rpm)
        .output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{}", stderr);
        Err(format!(
            "rpm package dump for {} returned {}",
            rpm.to_string_lossy(),
            output.status
        ))?
    }

    // Return rpm dump details
    let mut rpm_files = output
        .stdout
        .split(|c| *c == 0x0a)
        .filter(|line| !line.is_empty() && line[0] != b'(') // Handle '(contains no files)'
        .map(|line| parse_line(rpm_elem, line))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    // Add canonical names to the file list
    let cfiles = rpm_files
        .iter()
        .filter(|file| !is_symlink(file.mode))
        .filter_map(|file| match canonicalize(&file.path) {
            Ok(cpath) => {
                if file.path != cpath {
                    Some(RpmFile {
                        path: cpath,
                        ..file.clone()
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    rpm_files.extend(cfiles);

    Ok(rpm_files)
}

fn parse_line(rpm_elem: usize, line: &[u8]) -> Result<RpmFile, Box<dyn Error>> {
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

    let chksum = String::from_utf8(get_term(term_cnt - 8).to_vec())?;

    let mode_slice = get_term(term_cnt - 7);
    let mode = u32::from_str_radix(&String::from_utf8(mode_slice.to_vec())?, 8).map_err(|e| {
        format!(
            "Failed to parse file mode in '{}': {e}",
            String::from_utf8_lossy(mode_slice)
        )
    })?;

    Ok(RpmFile {
        rpm: rpm_elem,
        path,
        size,
        mode,
        chksum,
    })
}
