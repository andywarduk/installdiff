use memmap2::{Advice, Mmap};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{symlink_metadata, File, Metadata},
    os::unix::fs::MetadataExt,
};

use crate::packageman::{PackageDb, PackageFile};

use super::{report::Report, CheckArgs};

pub fn verify(packagedb: &PackageDb, args: &CheckArgs, reports: &mut Report) {
    // Verify files
    for file in packagedb.files() {
        match symlink_metadata(file.path()) {
            Ok(meta) => {
                if args.changed {
                    verify_file(packagedb, args, reports, file, meta);
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    if args.missing {
                        reports.add_missing(packagedb, file);
                    }
                }
                _ => eprintln!("ERROR: Failed to stat file {} ({e})", file.path().display()),
            },
        }
    }
}

fn verify_file(
    packagedb: &PackageDb,
    args: &CheckArgs,
    reports: &mut Report,
    file: &PackageFile,
    meta: Metadata,
) {
    // Check for mode change
    if let Some(mode) = file.mode() {
        if meta.mode() != *mode {
            reports.add_change(
                packagedb,
                file,
                format!(
                    "mode from {} to {}",
                    unix_mode::to_string(*mode),
                    unix_mode::to_string(meta.mode())
                ),
            );

            return;
        }
    }

    // Check file size
    if let Some(size) = file.size() {
        if meta.size() != *size as u64 {
            reports.add_change(
                packagedb,
                file,
                format!("size from {} to {}", size, meta.size()),
            );

            return;
        }
    }

    // Check checksum
    if args.checksum && file.chksum().is_some() {
        match check_digest(file) {
            Ok(matches) => {
                if !matches {
                    reports.add_change(packagedb, file, String::from("Hash changed"));
                    return;
                }
            }
            Err(e) => eprintln!(
                "ERROR: Failed to check hash for {} ({e})",
                file.path().display()
            ),
        }
    }

    // Check modification date for regular files
    if meta.is_file() {
        if let Some(mtime) = file.time() {
            if meta.mtime() > *mtime {
                reports.add_change(packagedb, file, String::from("Modification time later"));
                #[allow(clippy::needless_return)]
                return;
            }
        }
    }
}

fn check_digest(package_file: &PackageFile) -> Result<bool, Box<dyn Error>> {
    let chksum = package_file.chksum().as_ref().unwrap();

    let hasher: Box<dyn Fn(Mmap) -> bool> = match chksum.len() {
        16 => {
            // MD5
            Box::new(|bytes| -> bool {
                let digest: [u8; 16] = md5::compute(bytes).into();
                digest == chksum.as_slice()
            })
        }
        32 => {
            // SHA256
            Box::new(|bytes| -> bool {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                let hash = hasher.finalize();

                hash[..] == *chksum
            })
        }
        len => Err(format!("ERROR: Unknown hash length {}", len))?,
    };

    // Open the file
    let file = File::open(package_file.path())?;

    // Mem map the file
    let mmap = unsafe { Mmap::map(&file)? };
    let _ = mmap.advise(Advice::Sequential);

    // Hash the file and check
    Ok(hasher(mmap))
}
