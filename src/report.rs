use std::path::{Path, PathBuf};

use crate::packageman::{PackageDb, PackageFile};

pub enum ReportItem {
    Missing(Missing),
    Changed(Changed),
    New(New),
}

impl ReportItem {
    pub fn path(&self) -> &Path {
        match self {
            ReportItem::Missing(missing) => &missing.path,
            ReportItem::Changed(changed) => &changed.path,
            ReportItem::New(new) => &new.path,
        }
    }
}

impl std::fmt::Display for ReportItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportItem::Missing(missing) => f.write_fmt(format_args!(
                "MISSING {} (package {})",
                missing.path.display(),
                missing.rpm
            )),
            ReportItem::Changed(changed) => f.write_fmt(format_args!(
                "CHANGED {} (package {}, {})",
                changed.path.display(),
                changed.rpm,
                changed.desc
            )),
            ReportItem::New(new) => f.write_fmt(format_args!(
                "NEW     {} ({})",
                new.path.display(),
                unix_mode::to_string(new.mode)
            )),
        }
    }
}

pub struct Missing {
    path: PathBuf,
    rpm: String,
}

pub struct Changed {
    path: PathBuf,
    rpm: String,
    desc: String,
}

pub struct New {
    path: PathBuf,
    mode: u32,
}

pub struct Report {
    reports: Vec<ReportItem>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    pub fn add_missing(&mut self, packagedb: &PackageDb, file: &PackageFile) {
        self.reports.push(ReportItem::Missing(Missing {
            path: file.path.clone(),
            rpm: packagedb.package_to_string(file.package).to_string(),
        }))
    }

    pub fn add_change(&mut self, packagedb: &PackageDb, file: &PackageFile, desc: String) {
        self.reports.push(ReportItem::Changed(Changed {
            path: file.path.clone(),
            rpm: packagedb.package_to_string(file.package).to_string(),
            desc,
        }))
    }

    pub fn add_new(&mut self, file: PathBuf, mode: u32) {
        self.reports.push(ReportItem::New(New { path: file, mode }))
    }

    pub fn sort(&mut self) {
        self.reports.sort_by(|a, b| a.path().cmp(b.path()))
    }

    pub fn print(&self) {
        for rep in &self.reports {
            println!("{rep}");
        }
    }
}
