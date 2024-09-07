use std::{error::Error, process::Command};

use dpkgquery::dpkg_query;

use super::LoadResult;

mod dpkgcsums;
mod dpkgquery;

pub fn load_apt(debug: u8) -> Result<LoadResult, Box<dyn Error>> {
    let (packages, files) = dpkg_query(debug)?;

    // Default ignores for apt systems
    let ignores = vec![
        "^/var/lib/apt($|/.*)".into(),
        "^/var/lib/dpkg($|/.*)".into(),
    ];

    Ok((packages, files, ignores))
}

pub fn apt_available() -> bool {
    match Command::new("dpkg-query").arg("--version").output() {
        Ok(output) => output.status.success(),
        _ => false,
    }
}
