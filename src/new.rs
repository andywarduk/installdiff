use libc::{
    statfs64, CGROUP2_SUPER_MAGIC, CGROUP_SUPER_MAGIC, DEBUGFS_MAGIC, HUGETLBFS_MAGIC,
    PROC_SUPER_MAGIC, SYSFS_MAGIC, TMPFS_MAGIC, TRACEFS_MAGIC,
};
use std::{
    cmp::Ordering,
    ffi::CString,
    fs::{self},
    mem::MaybeUninit,
    num::NonZero,
    os::{linux::fs::MetadataExt, unix::ffi::OsStrExt},
    path::{Path, PathBuf},
};

use crate::rpmdb::RpmDb;

pub fn check_new(rpmdb: &RpmDb) {
    // Walk filesystem looking for new files
    let mut rpm_ptr = 0;

    check_new_dir(PathBuf::from("/"), rpmdb, &mut rpm_ptr);
}

fn check_new_dir(dir: PathBuf, rpmdb: &RpmDb, rpm_ptr: &mut usize) {
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
                check_new_ent(ent, rpmdb, rpm_ptr);
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to read directory {} ({e})", &dir.display());
        }
    }
}

fn check_new_ent(ent: PathBuf, rpmdb: &RpmDb, rpm_ptr: &mut usize) {
    loop {
        let rpm_file = &rpmdb.files[*rpm_ptr].path;

        match ent.cmp(rpm_file) {
            Ordering::Equal => {
                *rpm_ptr += 1;

                if should_recurse(&ent) {
                    check_new_dir(ent, rpmdb, rpm_ptr);
                }
            }
            Ordering::Less => {
                report_new(&ent);
            }
            Ordering::Greater => {
                *rpm_ptr += 1;
                continue;
            }
        }

        break;
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
            match stat.f_type {
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

fn report_new(ent: &Path) {
    let mode = match ent.symlink_metadata() {
        Ok(meta) => meta.st_mode(),
        _ => 0,
    };

    eprintln!("NEW: {} {}", unix_mode::to_string(mode), &ent.display());
}
