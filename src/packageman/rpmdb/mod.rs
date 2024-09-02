use std::{collections::HashMap, error::Error, fs::canonicalize};

use rpmdump::get_rpm_dump;
use rpmlist::get_rpm_list;

use super::PackageDb;

mod rpmdump;
mod rpmlist;

pub fn load_rpmdb(debug: bool) -> Result<PackageDb, Box<dyn Error>> {
    // Get list of RPMs
    let rpms = get_rpm_list(debug)?;

    // Build RPM file list
    let mut rpm_files = Vec::new();

    for (rpm_elem, rpm) in rpms.iter().enumerate() {
        if debug {
            eprintln!("Loading {}", rpm.to_string_lossy());
        }

        let this_rpm_files = get_rpm_dump(rpm, rpm_elem)?;

        if debug {
            eprintln!("{} files found", this_rpm_files.len());
        }

        rpm_files.extend(this_rpm_files);
    }

    rpm_files.sort_by(|a, b| a.path.cmp(&b.path));

    // Build hashmap from canonical name to element
    let cmap = rpm_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let cpath = match canonicalize(&file.path) {
                Ok(path) => path,
                Err(_) => file.path.clone(),
            };

            (cpath, i)
        })
        .collect::<HashMap<_, _>>();

    Ok(PackageDb {
        packages: rpms,
        files: rpm_files,
        cmap,
    })
}
