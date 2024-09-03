use std::{error::Error, ffi::OsString, process::Command};

use rpmdump::get_rpm_dump;
use rpmlist::get_rpm_list;

use super::PackageFile;

mod rpmdump;
mod rpmlist;

pub fn load_rpm(debug: u8) -> Result<(Vec<OsString>, Vec<PackageFile>), Box<dyn Error>> {
    // Get list of RPMs
    let rpms = get_rpm_list(debug)?;

    // Build RPM file list
    let mut rpm_files = Vec::new();

    for (rpm_elem, rpm) in rpms.iter().enumerate() {
        if debug > 0 {
            eprintln!("Loading {}", rpm.to_string_lossy());
        }

        let this_rpm_files = get_rpm_dump(rpm, rpm_elem)?;

        if debug > 0 {
            eprintln!("{} files found", this_rpm_files.len());
        }

        rpm_files.extend(this_rpm_files);
    }

    Ok((rpms, rpm_files))
}

pub fn rpm_available() -> bool {
    match Command::new("rpm").arg("--version").output() {
        Ok(output) => output.status.success(),
        _ => false,
    }
}
