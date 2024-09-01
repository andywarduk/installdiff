use std::error::Error;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::process::Command;

pub fn get_rpm_list() -> Result<Vec<OsString>, Box<dyn Error>> {
    // Run rpm -ql to get list of installed packages
    let output = Command::new("rpm").arg("-qa").output()?;

    // Successful?
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{}", stderr);
        Err(format!("rpm package query returned {}", output.status))?
    }

    // Return list of rpms
    let rpms = output
        .stdout
        .split(|c| *c == 0x0a)
        .filter(|line| !line.is_empty())
        .map(|line| OsString::from_vec(line.to_vec()))
        .collect::<Vec<_>>();

    Ok(rpms)
}
