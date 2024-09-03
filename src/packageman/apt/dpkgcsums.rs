use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStringExt,
    process::Command,
};

use crate::packageman::decode_hex;

pub fn dpkgcsums(package: &OsStr, debug: u8) -> HashMap<OsString, Vec<u8>> {
    // Run dpkg-query --control-show <pkg> md5sums to get map of file to checksum
    match Command::new("dpkg-query")
        .arg("--control-show")
        .arg(package)
        .arg("md5sums")
        .output()
    {
        Ok(output) => {
            // Successful?
            if !output.status.success() {
                if debug > 0 {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!(
                        "Failed to get checksums for {}: {}",
                        package.to_string_lossy(),
                        stderr
                    );
                }

                HashMap::new()
            } else {
                let map = output
                    .stdout
                    .split(|c| *c == 0x0a)
                    .filter_map(|line| parse_line(line, debug))
                    .collect::<HashMap<_, _>>();

                if debug > 1 {
                    eprintln!(
                        "{} checksum entries for {}",
                        map.len(),
                        package.to_string_lossy()
                    )
                }

                map
            }
        }
        Err(e) => {
            if debug > 0 {
                eprintln!(
                    "Failed to get checksums for {}: {e}",
                    package.to_string_lossy()
                )
            }

            HashMap::new()
        }
    }
}

fn parse_line(line: &[u8], debug: u8) -> Option<(OsString, Vec<u8>)> {
    let mut split = line.split(|c| *c == 0x20);

    // Extract checksum
    let chksum_str = std::str::from_utf8(split.next()?).ok()?;
    let sum = decode_hex(chksum_str).ok()?;

    // Skip double space
    split.next()?;

    // Extract path adding / to the start
    let mut path = vec![b'/'];
    path.extend(split.next()?);
    let path = OsString::from_vec(path);

    if debug > 2 {
        eprintln!("Checksum for {}: {:?}", path.to_string_lossy(), sum)
    }

    Some((path, sum))
}
