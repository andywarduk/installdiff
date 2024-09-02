use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{symlink_metadata, File},
    os::unix::fs::MetadataExt,
};
use unix_mode::is_file;

use crate::{
    packageman::{PackageDb, PackageFile},
    report::Reports,
    Check,
};

pub fn verify(packagedb: &PackageDb, check: &Check, reports: &mut Reports) {
    // Verify files
    for file in &packagedb.files {
        match symlink_metadata(&file.path) {
            Ok(meta) => {
                if !check.nochanged {
                    // Check for mode change
                    if meta.mode() != file.mode {
                        reports.add_change(
                            packagedb,
                            file,
                            format!(
                                "mode from {} to {}",
                                unix_mode::to_string(file.mode),
                                unix_mode::to_string(meta.mode())
                            ),
                        );

                        continue;
                    }

                    // Check file size for regular files
                    if is_file(file.mode) {
                        if meta.size() != file.size as u64 {
                            reports.add_change(
                                packagedb,
                                file,
                                format!("size from {} to {}", file.size, meta.size()),
                            );

                            continue;
                        }

                        // Check checksum
                        if !check.nodigest {
                            match check_digest(file) {
                                Ok(matches) => {
                                    if !matches {
                                        reports.add_change(
                                            packagedb,
                                            file,
                                            String::from("Hash changed"),
                                        );
                                    }
                                }
                                Err(e) => eprintln!(
                                    "ERROR: Failed to check hash for {} ({e})",
                                    file.path.display()
                                ),
                            }
                        }
                    }
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    if !check.nomissing {
                        reports.add_missing(packagedb, file);
                    }
                }
                _ => eprintln!("ERROR: Failed to stat file {} ({e})", &file.path.display()),
            },
        }
    }
}

fn check_digest(package_file: &PackageFile) -> Result<bool, Box<dyn Error>> {
    let hasher: Box<dyn Fn(Mmap) -> bool> = match package_file.chksum.len() {
        16 => {
            // MD5
            Box::new(|bytes| -> bool {
                let digest: [u8; 16] = md5::compute(bytes).into();
                digest == package_file.chksum.as_slice()
            })
        }
        32 => {
            // SHA256
            Box::new(|bytes| -> bool {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                let hash = hasher.finalize();

                hash[..] == package_file.chksum
            })
        }
        len => Err(format!("ERROR: Unknown hash length {}", len))?,
    };

    // Open the file
    let file = File::open(&package_file.path)?;

    // Mem map the file
    let mmap = unsafe { Mmap::map(&file)? };

    Ok(hasher(mmap))
}
