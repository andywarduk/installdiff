use std::{fs::symlink_metadata, os::unix::fs::MetadataExt};
use unix_mode::is_file;

use crate::rpmdb::RpmDb;

pub fn verify(rpmdb: &RpmDb) {
    // Verify files
    for f in &rpmdb.files {
        match symlink_metadata(&f.path) {
            Ok(meta) => {
                if meta.mode() != f.mode {
                    eprintln!(
                        "MODE (from {} to {}): {} in {}",
                        unix_mode::to_string(f.mode),
                        unix_mode::to_string(meta.mode()),
                        &f.path.display(),
                        rpmdb.rpm_to_string(f.rpm)
                    );

                    continue;
                }

                if is_file(f.mode) && meta.size() != f.size as u64 {
                    eprintln!(
                        "SIZE (from {} to {}): {} in {}",
                        f.size,
                        meta.size(),
                        &f.path.display(),
                        rpmdb.rpm_to_string(f.rpm)
                    );

                    continue;
                }

                // TODO checksum
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => eprintln!(
                    "MISSING: {} in {}",
                    &f.path.display(),
                    rpmdb.rpm_to_string(f.rpm)
                ),
                _ => eprintln!("ERROR: Failed to stat file {} ({e})", &f.path.display()),
            },
        }
    }
}
