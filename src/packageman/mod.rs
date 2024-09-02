use std::{borrow::Cow, collections::HashMap, error::Error, ffi::OsString, path::PathBuf};

use rpmdb::load_rpmdb;

mod rpmdb;

pub struct PackageDb {
    pub packages: Vec<OsString>,
    pub files: Vec<PackageFile>,
    pub cmap: HashMap<PathBuf, usize>,
}

impl PackageDb {
    pub fn load_rpmdb(debug: bool) -> Result<PackageDb, Box<dyn Error>> {
        load_rpmdb(debug)
    }

    pub fn package_to_string(&self, idx: usize) -> Cow<'_, str> {
        self.packages[idx].to_string_lossy()
    }
}

#[derive(Debug)]
pub struct PackageFile {
    pub package: usize,
    pub path: PathBuf,
    pub size: usize,
    pub mode: u32,
    pub chksum: Vec<u8>,
}
