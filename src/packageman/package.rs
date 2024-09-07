use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
};

#[derive(Debug)]
pub struct Package {
    fullname: OsString,
    name: OsString,
    version: OsString,
    arch: Option<OsString>,
}

impl Package {
    pub fn new(
        fullname: OsString,
        name: OsString,
        version: OsString,
        arch: Option<OsString>,
    ) -> Self {
        Self {
            fullname,
            name,
            version,
            arch,
        }
    }

    pub fn fullname(&self) -> &OsStr {
        &self.fullname
    }

    pub fn fullnamestr(&self) -> Cow<str> {
        self.fullname.to_string_lossy()
    }

    pub fn namestr(&self) -> Cow<str> {
        self.name.to_string_lossy()
    }

    pub fn versionstr(&self) -> Cow<str> {
        self.version.to_string_lossy()
    }

    pub fn archstr(&self) -> Cow<str> {
        match &self.arch {
            Some(arch) => arch.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }

    pub fn name_arch(&self) -> String {
        match &self.arch {
            Some(arch) => format!("{}:{}", self.name.to_string_lossy(), arch.to_string_lossy()),
            None => format!("{}", self.name.to_string_lossy(),),
        }
    }

    pub fn name_ver_arch(&self) -> String {
        match &self.arch {
            Some(arch) => format!(
                "{}-{}:{}",
                self.name.to_string_lossy(),
                self.version.to_string_lossy(),
                arch.to_string_lossy()
            ),
            None => format!(
                "{}-{}",
                self.name.to_string_lossy(),
                self.version.to_string_lossy(),
            ),
        }
    }
}
