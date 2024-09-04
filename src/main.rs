use clap::{ArgAction, Parser, Subcommand};
use new::check_new;
use packageman::{PackageDb, PackageMgr};
use report::Report;
use std::{borrow::Cow, error::Error};
use verify::verify;

mod new;
mod packageman;
mod report;
mod verify;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Package manager
    #[arg(short = 'p', long)]
    manager: Option<PackageMgr>,

    /// Print debugging messages
    #[arg(short = 'd', long, action = ArgAction::Count)]
    debug: u8,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Checks the package manager database against the files on the filesystem
    Check(Check),
    /// Prints a list of files in the package manager database
    List,
}

#[derive(Parser, Clone, Default)]
struct Check {
    /// Don't report changed files
    #[arg(short = 'c', long)]
    nochanged: bool,

    /// Don't report missing files
    #[arg(short = 'm', long)]
    nomissing: bool,

    /// Don't report new files
    #[arg(short = 'n', long)]
    nonew: bool,

    /// Check file checksums
    #[arg(short = 's', long)]
    checksum: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Get or default command
    let command = match &cli.command {
        Some(c) => Cow::Borrowed(c),
        None => Cow::Owned(Commands::Check(Check::default())),
    };

    match command.as_ref() {
        Commands::List => {
            // List installed package files

            // Load package database
            let packagedb = load_packages(&cli)?;

            // Print files
            for file in packagedb.files() {
                println!(
                    "{} (package {})",
                    file.path.display(),
                    packagedb.package_to_string(file.package)
                )
            }
        }
        Commands::Check(check) => {
            // Check packages

            // Load package database
            let packagedb = load_packages(&cli)?;

            // Create report
            let mut report = Report::new();

            // Verify package database files
            if !check.nochanged || !check.nomissing {
                verify(&packagedb, check, &mut report);
            }

            // Check for new files
            if !check.nonew {
                check_new(&packagedb, &mut report);
            }

            // Sort report in to file order
            report.sort();

            // Print the report
            report.print();
        }
    }

    Ok(())
}

fn load_packages(cli: &Cli) -> Result<PackageDb, Box<dyn Error>> {
    let mgr = match &cli.manager {
        Some(mgr) => mgr.clone(),
        None => PackageDb::detect_mgr()?,
    };

    PackageDb::load(mgr, cli.debug)
}
