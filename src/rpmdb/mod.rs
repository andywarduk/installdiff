use std::{
    borrow::Cow, collections::HashMap, error::Error, ffi::OsString, fs::canonicalize, path::PathBuf,
};

use rpmdump::get_rpm_dump;
use rpmlist::get_rpm_list;

mod rpmdump;
mod rpmlist;

pub struct RpmDb {
    pub rpms: Vec<OsString>,
    pub files: Vec<RpmFile>,
    pub cmap: HashMap<PathBuf, usize>,
}

impl RpmDb {
    pub fn rpm_to_string(&self, idx: usize) -> Cow<'_, str> {
        self.rpms[idx].to_string_lossy()
    }
}

#[derive(Debug)]
pub struct RpmFile {
    pub rpm: usize,
    pub path: PathBuf,
    pub size: usize,
    pub mode: u32,
    pub chksum: String,
}

pub fn load_rpm_database(debug: bool) -> Result<RpmDb, Box<dyn Error>> {
    // Get list of RPMs
    let rpms = get_rpm_list(debug)?;

    // Build RPM file list
    let mut rpm_files = Vec::new();

    for (rpm_elem, rpm) in rpms.iter().enumerate() {
        if debug {
            println!("Loading {}", rpm.to_string_lossy());
        }

        let this_rpm_files = get_rpm_dump(rpm, rpm_elem)?;

        if debug {
            println!("{} files found", this_rpm_files.len());
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

    Ok(RpmDb {
        rpms,
        files: rpm_files,
        cmap,
    })
}
