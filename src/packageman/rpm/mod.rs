use rayon::prelude::*;
use std::{error::Error, process::Command, sync::Mutex};

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

    let rpm_files_mutex = Mutex::new(Vec::new());

    rpms.par_iter().enumerate().for_each(|(rpm_elem, rpm)| {
        if debug > 1 {
            eprintln!("Loading {}", rpm.name_arch());
        }

        // Get RPM contents
        match get_rpm_dump(rpm, rpm_elem) {
            Ok(this_rpm_files) => {
                if debug > 1 {
                    eprintln!(
                        "{} files found in {}",
                        this_rpm_files.len(),
                        rpm.name_arch()
                    );
                }

                // Add to rpm files vector
                let mut rpm_files = rpm_files_mutex.lock().unwrap();

                rpm_files.extend(this_rpm_files);

                drop(rpm_files);
            }
            Err(e) => eprintln!(
                "ERROR: Failed to get RPM file list for {}: {e}",
                rpm.fullnamestr()
            ),
        }
    });

    let rpm_files = rpm_files_mutex.into_inner().unwrap();

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
