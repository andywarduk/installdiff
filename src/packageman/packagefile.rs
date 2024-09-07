use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PackageFile {
    path: PathBuf,
    package: Option<usize>,
    size: Option<usize>,
    mode: Option<u32>,
    chksum: Option<Vec<u8>>,
    time: Option<i64>,
}

impl PackageFile {
    pub fn new(
        path: PathBuf,
        package: Option<usize>,
        size: Option<usize>,
        mode: Option<u32>,
        chksum: Option<Vec<u8>>,
        time: Option<i64>,
    ) -> Self {
        Self {
            path,
            package,
            size,
            mode,
            chksum,
            time,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn package(&self) -> &Option<usize> {
        &self.package
    }

    pub fn size(&self) -> &Option<usize> {
        &self.size
    }

    pub fn mode(&self) -> &Option<u32> {
        &self.mode
    }

    pub fn chksum(&self) -> &Option<Vec<u8>> {
        &self.chksum
    }

    pub fn time(&self) -> &Option<i64> {
        &self.time
    }
}
