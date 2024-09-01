use std::error::Error;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::process::Command;

pub fn get_rpm_list(debug: bool) -> Result<Vec<OsString>, Box<dyn Error>> {
    if debug {
        eprintln!("Getting RPM list");
    }

    // Run rpm -qa to get list of installed packages
    let output = Command::new("rpm").arg("-qa").output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        Err(format!("rpm package query returned {}", output.status))?
    }

    // Return list of rpms
    let rpms = output
        .stdout
        .split(|c| *c == 0x0a)
        .filter(|line| !line.is_empty())
        .map(|line| OsString::from_vec(line.to_vec()))
        .collect::<Vec<_>>();

    if debug {
        eprintln!("{} RPMs found", rpms.len());
    }

    Ok(rpms)
}
