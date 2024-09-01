use std::path::{Path, PathBuf};

use crate::rpmdb::{RpmDb, RpmFile};

pub enum Report {
    Missing(MissingReport),
    Changed(ChangedReport),
    New(NewReport),
}

impl Report {
    pub fn path(&self) -> &Path {
        match self {
            Report::Missing(missing) => &missing.path,
            Report::Changed(changed) => &changed.path,
            Report::New(new) => &new.path,
        }
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Report::Missing(missing) => f.write_fmt(format_args!(
                "MISSING {} (package {})",
                missing.path.display(),
                missing.rpm
            )),
            Report::Changed(changed) => f.write_fmt(format_args!(
                "CHANGED {} (package {}, {})",
                changed.path.display(),
                changed.rpm,
                changed.desc
            )),
            Report::New(new) => f.write_fmt(format_args!(
                "NEW     {} ({})",
                new.path.display(),
                unix_mode::to_string(new.mode)
            )),
        }
    }
}

pub struct MissingReport {
    path: PathBuf,
    rpm: String,
}

pub struct ChangedReport {
    path: PathBuf,
    rpm: String,
    desc: String,
}

pub struct NewReport {
    path: PathBuf,
    mode: u32,
}

pub struct Reports {
    reports: Vec<Report>,
}

impl Reports {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    pub fn add_missing(&mut self, rpmdb: &RpmDb, file: &RpmFile) {
        self.reports.push(Report::Missing(MissingReport {
            path: file.path.clone(),
            rpm: rpmdb.rpm_to_string(file.rpm).to_string(),
        }))
    }

    pub fn add_change(&mut self, rpmdb: &RpmDb, file: &RpmFile, desc: String) {
        self.reports.push(Report::Changed(ChangedReport {
            path: file.path.clone(),
            rpm: rpmdb.rpm_to_string(file.rpm).to_string(),
            desc,
        }))
    }

    pub fn add_new(&mut self, file: PathBuf, mode: u32) {
        self.reports
            .push(Report::New(NewReport { path: file, mode }))
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
