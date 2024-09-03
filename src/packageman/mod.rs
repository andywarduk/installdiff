use clap::ValueEnum;
use std::{
    borrow::Cow,
    collections::HashSet,
    error::Error,
    ffi::OsString,
    fs::canonicalize,
    num::ParseIntError,
    path::{Path, PathBuf},
};

use apt::{apt_available, load_apt};
use rpmdb::{load_rpm, rpm_available};

mod apt;
mod rpmdb;

#[derive(ValueEnum, Clone)]
pub enum PackageMgr {
    Rpm,
    Apt,
}

pub struct PackageDb {
    packages: Vec<OsString>,
    files: Vec<PackageFile>,
    cset: HashSet<PathBuf>,
}

impl PackageDb {
    pub fn detect_mgr() -> Result<PackageMgr, Box<dyn Error>> {
        let rpm_available = rpm_available();
        let apt_available = apt_available();

        if rpm_available {
            if apt_available {
                Err("No package manager specified")?
            }

            Ok(PackageMgr::Rpm)
        } else {
            if apt_available {
                Ok(PackageMgr::Apt)
            } else {
                Err("No supported package managers available")?
            }
        }
    }

    pub fn load(mgr: PackageMgr, debug: u8) -> Result<PackageDb, Box<dyn Error>> {
        match mgr {
            PackageMgr::Rpm => Self::load_rpm(debug),
            PackageMgr::Apt => Self::load_apt(debug),
        }
    }

    fn load_rpm(debug: u8) -> Result<PackageDb, Box<dyn Error>> {
        let (packages, files) = load_rpm(debug)?;

        Ok(Self::new(packages, files))
    }

    fn load_apt(debug: u8) -> Result<PackageDb, Box<dyn Error>> {
        let (packages, files) = load_apt(debug)?;

        Ok(Self::new(packages, files))
    }

    fn new(packages: Vec<OsString>, mut files: Vec<PackageFile>) -> PackageDb {
        // Sort file list
        files.sort_by(|a, b| a.path.cmp(&b.path));

        // Build hashset of canonical names
        let cset = files
            .iter()
            .map(|file| match canonicalize(&file.path) {
                Ok(path) => path,
                Err(_) => file.path.clone(),
            })
            .collect::<HashSet<_>>();

        PackageDb {
            packages,
            files,
            cset,
        }
    }

    pub fn files<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PackageFile> + 'a> {
        Box::new(self.files.iter())
    }

    pub fn package_to_string(&self, idx: usize) -> Cow<'_, str> {
        self.packages[idx].to_string_lossy()
    }

    pub fn find_canonical(&self, path: &Path) -> bool {
        self.cset.contains(path)
    }
}

#[derive(Debug)]
pub struct PackageFile {
    pub package: usize,
    pub path: PathBuf,
    pub size: Option<usize>,
    pub mode: Option<u32>,
    pub chksum: Option<Vec<u8>>,
    pub time: Option<i64>,
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
