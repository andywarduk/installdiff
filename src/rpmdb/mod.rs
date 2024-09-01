use std::{borrow::Cow, error::Error, ffi::OsString};

use rpmdump::{get_rpm_dump, RpmFile};
use rpmlist::get_rpm_list;

mod rpmdump;
mod rpmlist;

pub struct RpmDb {
    pub rpms: Vec<OsString>,
    pub files: Vec<RpmFile>,
}

impl RpmDb {
    pub fn rpm_to_string(&self, idx: usize) -> Cow<'_, str> {
        self.rpms[idx].to_string_lossy()
    }
}

pub fn load_rpm_database() -> Result<RpmDb, Box<dyn Error>> {
    let mut rpm_files = Vec::new();

    let rpms = get_rpm_list()?;

    for (rpm_elem, rpm) in rpms.iter().enumerate() {
        // TODO debug println!("Loading {}", rpm.to_string_lossy());

        let this_rpm_files = get_rpm_dump(rpm, rpm_elem)?;

        rpm_files.extend(this_rpm_files);
    }

    rpm_files.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(RpmDb {
        rpms,
        files: rpm_files,
    })
}
