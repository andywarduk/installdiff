use std::{fs::symlink_metadata, os::unix::fs::MetadataExt};
use unix_mode::is_file;

use crate::rpmdb::{RpmDb, RpmFile};

pub fn verify(rpmdb: &RpmDb, changed: bool, missing: bool) {
    // Verify files
    for file in &rpmdb.files {
        match symlink_metadata(&file.path) {
            Ok(meta) => {
                if changed {
                    if meta.mode() != file.mode {
                        report_change(
                            rpmdb,
                            file,
                            format!(
                                "mode from {} to {}",
                                unix_mode::to_string(file.mode),
                                unix_mode::to_string(meta.mode())
                            ),
                        );

                        continue;
                    }

                    if is_file(file.mode) && meta.size() != file.size as u64 {
                        report_change(
                            rpmdb,
                            file,
                            format!("size from {} to {}", file.size, meta.size()),
                        );

                        continue;
                    }

                    // TODO checksum
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    if missing {
                        report_missing(rpmdb, file)
                    }
                }
                _ => eprintln!("ERROR: Failed to stat file {} ({e})", &file.path.display()),
            },
        }
    }
}

fn report_change(rpmdb: &RpmDb, file: &RpmFile, desc: String) {
    eprintln!(
        "CHANGE ({}): {} in {}",
        desc,
        &file.path.display(),
        rpmdb.rpm_to_string(file.rpm)
    );
}

fn report_missing(rpmdb: &RpmDb, file: &RpmFile) {
    eprintln!(
        "MISSING: {} in {}",
        &file.path.display(),
        rpmdb.rpm_to_string(file.rpm)
    );
}
