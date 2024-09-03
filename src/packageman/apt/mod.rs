use std::{error::Error, ffi::OsString, process::Command};

use dpkgquery::dpkg_query;

use super::PackageFile;

mod dpkgcsums;
mod dpkgquery;

pub fn load_apt(debug: u8) -> Result<(Vec<OsString>, Vec<PackageFile>), Box<dyn Error>> {
    let (packages, files) = dpkg_query(debug)?;

    Ok((packages, files))
}

pub fn apt_available() -> bool {
    match Command::new("dpkg-query").arg("--version").output() {
        Ok(output) => output.status.success(),
        _ => false,
    }
}
