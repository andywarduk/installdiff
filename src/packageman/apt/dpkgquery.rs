use crate::packageman::{Package, PackageFile};
use rayon::prelude::*;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use super::dpkgcsums::dpkgcsums;

pub fn dpkg_query(debug: u8) -> Result<(Vec<Package>, Vec<PackageFile>), Box<dyn Error>> {
    let packages_mutex = Mutex::new(Vec::new());
    let files_mutex = Mutex::new(Vec::new());

    if debug > 0 {
        eprintln!("Getting dpkg list");
    }

    const FORMAT: &str = "${Package}\t${Version}\t${Architecture}\t${db:Status-Abbrev}\t${db-fsys:Last-Modified}\n${db-fsys:Files}";
    const END: &[u8; 5] = b"!END\n";

    // Run dpkg-query --show to get list of installed packages and files
    let output = Command::new("dpkg-query")
        .arg("--show")
        .arg("--showformat")
        .arg(format!("{}{}", FORMAT, std::str::from_utf8(END).unwrap()))
        .output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        Err(format!("dpkg-query returned {}", output.status))?
    }

    // Get stdout
    let stdout = output.stdout;

    // Find end positions of each chunk and add zero to the start
    let end_pos = [0usize]
        .into_iter()
        .chain(
            stdout
                .windows(END.len())
                .enumerate()
                .filter_map(|(pos, chars)| {
                    if chars == END {
                        Some(pos + END.len())
                    } else {
                        None
                    }
                }),
        )
        .collect::<Vec<_>>();

    // Iterate each chunk
    end_pos.par_windows(2).for_each(|positions| {
        let [start, next] = positions else {
            panic!("Expecting start and next")
        };

        // Extract slice of stdout
        let chunk = &stdout[*start..*next];

        // Split in to lines
        let mut lines = chunk.split(|c| *c == 0x0a);

        // Get package details
        if let Some(package_line) = lines.next() {
            // Extract package details
            let mut split = package_line.split(|c| *c == b'\t');

            let name = OsString::from_vec(split.next().unwrap().to_vec());
            let version = OsString::from_vec(split.next().unwrap().to_vec());
            let arch = OsString::from_vec(split.next().unwrap().to_vec());
            // TODO status ignored for now
            let _status = split.next().unwrap();

            // Build full name
            let mut fullname = OsString::new();
            fullname.push(&name);
            fullname.push("-");
            fullname.push(&version);
            fullname.push(":");
            fullname.push(&arch);

            // Get modification time
            let mtime = Some(
                std::str::from_utf8(split.next().unwrap())
                    .unwrap()
                    .parse::<i64>()
                    .unwrap(),
            );

            // Get checksums
            let csums = dpkgcsums(&fullname, debug);

            // Add to package list
            let mut packages = packages_mutex.lock().unwrap();

            packages.push(Package::new(fullname, name, version, Some(arch)));
            let package_elem = packages.len() - 1;

            drop(packages);

            // Add files
            let mut files = files_mutex.lock().unwrap();

            for line in lines {
                // Trim and convert to OS string
                let line = OsStr::from_bytes(line.trim_ascii_start());

                if !line.is_empty() && line != "!END" && line != "/." {
                    // Get checksum if any
                    let chksum = csums.get(line).cloned();

                    if debug > 2 && chksum.is_none() {
                        eprintln!("no checksum for {}", line.to_string_lossy())
                    }

                    // Add file
                    files.push(PackageFile::new(
                        PathBuf::from(line),
                        Some(package_elem),
                        None,
                        None,
                        chksum,
                        mtime,
                    ));
                }
            }

            drop(files);
        }
    });

    let packages = packages_mutex.into_inner().unwrap();
    let files = files_mutex.into_inner().unwrap();

    if debug > 0 {
        eprintln!("{} packages found", packages.len());
    }

    Ok((packages, files))
}
