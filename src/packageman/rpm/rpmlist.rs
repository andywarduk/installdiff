use std::error::Error;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::process::Command;

use crate::packageman::Package;

pub fn get_rpm_list(debug: u8) -> Result<Vec<Package>, Box<dyn Error>> {
    if debug > 0 {
        eprintln!("Getting RPM list");
    }

    // Run rpm -qa to get list of installed packages
    let output = Command::new("rpm")
        .arg("-qa")
        .arg("--queryformat")
        .arg("%{NAME}\t%{VERSION}\t%{RELEASE}\t%{ARCH}\n")
        .output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        Err(format!("rpm package query returned {}", output.status))?
    }

    // Return list of rpms
    let rpms = output
        .stdout
        .split(|c| *c == 0x0a)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut split = line.split(|c| *c == b'\t');

            let name = term(split.next().unwrap()).unwrap();
            let ver = term(split.next().unwrap());
            let rel = term(split.next().unwrap());
            let arch = term(split.next().unwrap());

            // Build full version
            let version = match ver {
                Some(ver) => match rel {
                    Some(rel) => {
                        let mut joined = OsString::new();

                        joined.push(ver);
                        joined.push("-");
                        joined.push(rel);

                        Some(joined)
                    }
                    None => Some(ver),
                },
                None => rel,
            };

            // Build full name
            let mut fullname = name.clone();

            if let Some(version) = &version {
                fullname.push("-");
                fullname.push(version);
            }

            Package::new(fullname, name, version.unwrap_or_else(OsString::new), arch)
        })
        .collect::<Vec<_>>();

    if debug > 0 {
        eprintln!("{} RPMs found", rpms.len());
    }

    Ok(rpms)
}

fn term(term: &[u8]) -> Option<OsString> {
    let term = OsString::from_vec(term.to_vec());

    if term == "(none)" {
        None
    } else {
        Some(term)
    }
}
