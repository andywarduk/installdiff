use std::collections::HashMap;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::process::Command;

use crate::packageman::PackageFile;

use super::dpkgcsums::dpkgcsums;

pub fn dpkg_query(debug: u8) -> Result<(Vec<OsString>, Vec<PackageFile>), Box<dyn Error>> {
    let mut packages = Vec::new();
    let mut files = Vec::new();

    if debug > 0 {
        eprintln!("Getting dpkg list");
    }

    // Run dpkg-query --show to get list of installed packages and files
    let output = Command::new("dpkg-query")
        .arg("--show")
        .arg("--showformat")
        .arg("${Package}:${Architecture}\t${db:Status-Abbrev}\t${db-fsys:Last-Modified}\n${db-fsys:Files}!END\n")
        .output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        Err(format!("dpkg-query returned {}", output.status))?
    }

    // Get list of packages and files
    let mut in_files = false;
    let mut csums: HashMap<OsString, Vec<u8>> = HashMap::new();
    let mut mtime = None;

    for line in output
        .stdout
        .split(|c| *c == 0x0a)
        .filter(|line| !line.is_empty())
    {
        if !in_files {
            let mut split = line.split(|c| *c == 0x09);

            // Get package name
            let name = OsString::from_vec(split.next().unwrap().to_vec());

            // Get status
            let _ = split.next().unwrap();
            // TODO status ignored for now

            // Get modification time
            mtime = Some(
                std::str::from_utf8(split.next().unwrap())
                    .unwrap()
                    .parse::<i64>()
                    .unwrap(),
            );

            // Get checksums
            csums = dpkgcsums(&name, debug);

            // Add to package list
            packages.push(name);

            in_files = true;
        } else {
            let line = OsStr::from_bytes(line.trim_ascii_start());

            if line == "!END" {
                in_files = false;
            } else if line != "/." {
                // Get checksum if any
                let chksum = csums.get(line).cloned();

                if debug > 2 && chksum.is_none() {
                    eprintln!("no checksum for {}", line.to_string_lossy())
                }

                // Add file
                files.push(PackageFile {
                    path: PathBuf::from(line),
                    package: Some(packages.len() - 1),
                    size: None,
                    mode: None,
                    chksum,
                    time: mtime,
                });
            }
        }
    }

    if debug > 0 {
        eprintln!("{} packages found", packages.len());
    }

    Ok((packages, files))
}
