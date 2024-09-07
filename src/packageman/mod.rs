use clap::ValueEnum;
pub use package::Package;
pub use packagefile::PackageFile;
use std::{
    collections::HashSet,
    error::Error,
    fs::canonicalize,
    num::ParseIntError,
    path::{Path, PathBuf},
};

use apt::{apt_available, load_apt};
use rpm::{load_rpm, rpm_available};

mod apt;
mod package;
mod packagefile;
mod rpm;

#[derive(ValueEnum, Clone)]
pub enum PackageMgr {
    Rpm,
    Apt,
}

pub struct PackageDb {
    packages: Vec<Package>,
    files: Vec<PackageFile>,
    cset: HashSet<PathBuf>,
    ignores: Vec<String>,
}

pub type LoadResult = (Vec<Package>, Vec<PackageFile>, Vec<String>);

impl PackageDb {
    pub fn detect_mgr() -> Result<PackageMgr, Box<dyn Error>> {
        let rpm_available = rpm_available();
        let apt_available = apt_available();

        if rpm_available {
            if apt_available {
                Err("No package manager specified")?
            }

            Ok(PackageMgr::Rpm)
        } else if apt_available {
            Ok(PackageMgr::Apt)
        } else {
            Err("No supported package managers available")?
        }
    }

    pub fn load(mgr: PackageMgr, debug: u8) -> Result<PackageDb, Box<dyn Error>> {
        match mgr {
            PackageMgr::Rpm => Self::load_rpm(debug),
            PackageMgr::Apt => Self::load_apt(debug),
        }
    }

    fn load_rpm(debug: u8) -> Result<PackageDb, Box<dyn Error>> {
        let (packages, files, ignores) = load_rpm(debug)?;

        Ok(Self::new(packages, files, ignores, debug))
    }

    fn load_apt(debug: u8) -> Result<PackageDb, Box<dyn Error>> {
        let (packages, files, ignores) = load_apt(debug)?;

        Ok(Self::new(packages, files, ignores, debug))
    }

    fn new(
        packages: Vec<Package>,
        mut files: Vec<PackageFile>,
        ignores: Vec<String>,
        debug: u8,
    ) -> PackageDb {
        // Sort file list
        files.sort_by(|a, b| a.path().cmp(b.path()));

        // Fill in any missing directories
        let mut last_dir: &Path = Path::new("/");
        let mut add_files = Vec::new();

        for file in &files {
            let mut anc = file.path().ancestors().collect::<Vec<_>>();
            anc.reverse();

            let next = anc.pop().unwrap();

            for anc in anc.into_iter().rev() {
                if last_dir.starts_with(anc) {
                    break;
                }

                if debug > 2 {
                    eprintln!("Adding missing path {}", anc.display());
                }

                add_files.push(PackageFile::new(
                    anc.to_owned(),
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
            }

            last_dir = next;
        }

        if !add_files.is_empty() {
            files.extend(add_files);

            // Resort file list
            files.sort_by(|a, b| a.path().cmp(b.path()));
        }

        // Build hashset of canonical names
        let cset = files
            .iter()
            .map(|file| match canonicalize(file.path()) {
                Ok(path) => path,
                Err(_) => PathBuf::from(file.path()),
            })
            .collect::<HashSet<_>>();

        PackageDb {
            packages,
            files,
            cset,
            ignores,
        }
    }

    pub fn packages<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Package> + 'a> {
        Box::new(self.packages.iter())
    }

    pub fn files<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PackageFile> + 'a> {
        Box::new(self.files.iter())
    }

    pub fn ignores<'a>(&'a self) -> Box<dyn Iterator<Item = &'a String> + 'a> {
        Box::new(self.ignores.iter())
    }

    pub fn package_to_string(&self, idx: Option<usize>, with_version: bool) -> String {
        match idx {
            Some(idx) => {
                if with_version {
                    self.packages[idx].name_arch()
                } else {
                    self.packages[idx].name_ver_arch()
                }
            }
            None => String::from("None"),
        }
    }

    pub fn find_canonical(&self, path: &Path) -> bool {
        self.cset.contains(path)
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
