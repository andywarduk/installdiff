use crate::packageman::PackageDb;
use new::check_new;
use regex::Regex;
use report::Report;
use verify::verify;

mod new;
mod report;
mod verify;

pub struct CheckArgs {
    pub changed: bool,
    pub missing: bool,
    pub new: bool,
    pub checksum: bool,
    pub ignores: Vec<String>,
    pub debug: u8,
}

const GLOBAL_IGNORES: [&str; 6] = [
    "^/etc/pki/ca-trust/extracted($|/.*)",
    "^/tmp($|/.*)",
    "^/var/log($|/.*)",
    "^/var/cache($|/.*)",
    "^/var/tmp($|/.*)",
    ".*?/__pycache__($|/.*)",
];

pub fn check(packagedb: &PackageDb, args: CheckArgs) {
    // Compile ignores
    let mut ignores = Vec::new();

    let mut add_ignore = |ignore| {
        if let Ok(regex) = Regex::new(ignore) {
            ignores.push(regex);
        } else {
            eprintln!("ERROR: Failed to compile regex '{}'", ignore);
        }
    };

    for ignore in GLOBAL_IGNORES {
        add_ignore(ignore);
    }

    for ignore in packagedb.ignores() {
        add_ignore(ignore);
    }

    for ignore in &args.ignores {
        add_ignore(ignore);
    }

    // Create report
    let mut report = Report::new(ignores);

    // Verify package files
    if args.changed || args.missing {
        verify(packagedb, &args, &mut report);
    }

    // Check for new files
    if args.new {
        check_new(packagedb, &mut report);
    }

    // Sort report in to file order
    report.sort();

    // Print the report
    report.print(args.debug);
}
