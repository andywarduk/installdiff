use std::{error::Error, process::Command};

use rpmdump::get_rpm_dump;
use rpmlist::get_rpm_list;

use super::LoadResult;

mod rpmdump;
mod rpmlist;

pub fn load_rpm(debug: u8) -> Result<LoadResult, Box<dyn Error>> {
    // Get list of RPMs
    let rpms = get_rpm_list(debug)?;

    // Build RPM file list
    if debug > 0 {
        eprintln!("Getting RPM file list");
    }

    let mut rpm_files = Vec::new();

    for (rpm_elem, rpm) in rpms.iter().enumerate() {
        if debug > 1 {
            eprintln!("Loading {}", rpm.name_arch());
        }

        let this_rpm_files = get_rpm_dump(rpm, rpm_elem)?;

        if debug > 1 {
            eprintln!(
                "{} files found in {}",
                this_rpm_files.len(),
                rpm.name_arch()
            );
        }

        rpm_files.extend(this_rpm_files);
    }

    if debug > 0 {
        eprintln!("{} files found", rpm_files.len());
    }

    // Default ignores for RPM systems
    let ignores = vec![
        "^/usr/share/man($|/.*)".into(),
        "^/var/lib/rpm($|/.*)".into(),
    ];

    Ok((rpms, rpm_files, ignores))
}

pub fn rpm_available() -> bool {
    match Command::new("rpm").arg("--version").output() {
        Ok(output) => output.status.success(),
        _ => false,
    }
}
