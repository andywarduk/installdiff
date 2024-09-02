use libc::{
    statfs64, CGROUP2_SUPER_MAGIC, CGROUP_SUPER_MAGIC, DEBUGFS_MAGIC, HUGETLBFS_MAGIC,
    PROC_SUPER_MAGIC, SYSFS_MAGIC, TMPFS_MAGIC, TRACEFS_MAGIC,
};
use std::{
    ffi::CString,
    fs::{self, canonicalize},
    mem::MaybeUninit,
    num::NonZero,
    os::{linux::fs::MetadataExt, unix::ffi::OsStrExt},
    path::{Path, PathBuf},
};

use crate::{packageman::PackageDb, report::Reports};

pub fn check_new(packagedb: &PackageDb, reports: &mut Reports) {
    // Walk filesystem looking for new files
    check_new_dir(PathBuf::from("/"), packagedb, reports);
}

fn check_new_dir(dir: PathBuf, packagedb: &PackageDb, reports: &mut Reports) {
    match fs::read_dir(&dir) {
        Ok(ents) => {
            let mut ents = ents
                .filter(|ent| match ent {
                    Ok(_) => true,
                    Err(e) => {
                        eprintln!(
                            "ERROR: Failed to get directory entry {} ({e})",
                            &dir.display()
                        );
                        false
                    }
                })
                .map(|ent| ent.unwrap().path())
                .collect::<Vec<_>>();

            ents.sort();

            for ent in ents {
                check_new_ent(ent, packagedb, reports);
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to read directory {} ({e})", &dir.display());
        }
    }
}

fn check_new_ent(ent: PathBuf, packagedb: &PackageDb, reports: &mut Reports) {
    let cpath = match canonicalize(&ent) {
        Ok(path) => path,
        _ => ent.clone(),
    };

    if packagedb.cmap.contains_key(&cpath) {
        if should_recurse(&ent) {
            check_new_dir(ent, packagedb, reports);
        }
    } else {
        let mode = match ent.symlink_metadata() {
            Ok(meta) => meta.st_mode(),
            _ => 0,
        };

        reports.add_new(ent, mode);
    }
}

fn should_recurse(ent: &Path) -> bool {
    let mut recurse = false;

    if ent.is_dir() && !ent.is_symlink() {
        recurse = true;

        // Convert path to non-zero bytes
        let cbytes = ent
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|b| NonZero::new(*b).unwrap())
            .collect::<Vec<_>>();

        // Convert non-zero bytes to C string
        let cstr = CString::from(cbytes);

        // Create statfs64 buffer
        let mut stat = MaybeUninit::<statfs64>::zeroed();
        let stat_ptr = stat.as_mut_ptr();

        // Stat the filesystem for the path
        let rc = unsafe { libc::statfs64(cstr.as_ptr(), stat_ptr) };

        if rc == 0 {
            let stat = unsafe { stat.assume_init() };

            // Check filesystem type
            match stat.f_type as libc::c_long {
                PROC_SUPER_MAGIC | TMPFS_MAGIC | SYSFS_MAGIC | DEBUGFS_MAGIC | TRACEFS_MAGIC
                | HUGETLBFS_MAGIC | CGROUP_SUPER_MAGIC | CGROUP2_SUPER_MAGIC => {
                    recurse = false;
                }
                _ => (),
            }
        }
    }

    recurse
}
