use check::{check, CheckArgs};
use clap::{ArgAction, Parser, Subcommand};
use packageman::{PackageDb, PackageMgr};
use regex::{escape, Regex};
use std::{borrow::Cow, error::Error};

mod check;
mod packageman;

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
    /// Prints a list of packages in the package manager database
    Packages,
    /// Prints a list of files in the package manager database
    Files,
}

#[derive(Parser, Clone, Default)]
struct Check {
    /// Don't report changed files
    #[arg(short = 'c', long)]
    no_changed: bool,

    /// Don't report missing files
    #[arg(short = 'm', long)]
    no_missing: bool,

    /// Don't report new files
    #[arg(short = 'n', long)]
    no_new: bool,

    /// Verify file checksums
    #[arg(short = 's', long)]
    checksum: bool,

    /// Ignore directory
    #[clap(short = 'i', long)]
    pub ignore_dir: Vec<String>,

    /// Ignore file
    #[clap(short = 'I', long)]
    pub ignore_file: Vec<String>,

    /// Ignore files matching regular expression
    #[clap(short = 'r', long, value_parser = validate_regex)]
    pub ignore_regex: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Get or default command
    let command = match &cli.command {
        Some(c) => Cow::Borrowed(c),
        None => Cow::Owned(Commands::Check(Check::default())),
    };

    match command.as_ref() {
        Commands::Packages => {
            // List installed packages

            // Load package database
            let packagedb = load_packages(&cli)?;

            // Sort in to name:arch order
            let mut packages = packagedb
                .packages()
                .map(|p| p.name_arch())
                .collect::<Vec<_>>();

            packages.sort();

            // Print package list
            for package in packages {
                println!("{}", package)
            }
        }
        Commands::Files => {
            // List installed package files

            // Load package database
            let packagedb = load_packages(&cli)?;

            // Print files
            for file in packagedb.files() {
                println!(
                    "{} (package {})",
                    file.path().display(),
                    packagedb.package_to_string(*file.package(), false)
                )
            }
        }
        Commands::Check(checkargs) => {
            // Check packages

            // Load package database
            let packagedb = load_packages(&cli)?;

            // Build ignore regular expression list
            let ignores = checkargs
                .ignore_regex
                .iter()
                .cloned()
                .chain(
                    checkargs
                        .ignore_dir
                        .iter()
                        .map(|dir| format!("^{}($|/.*)", escape(dir))),
                )
                .chain(
                    checkargs
                        .ignore_file
                        .iter()
                        .map(|file| format!("^{}$", escape(file))),
                )
                .collect::<Vec<_>>();

            // Report differences
            check(
                &packagedb,
                CheckArgs {
                    changed: !checkargs.no_changed,
                    missing: !checkargs.no_missing,
                    new: !checkargs.no_new,
                    checksum: checkargs.checksum,
                    ignores,
                    debug: cli.debug,
                },
            );
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

fn validate_regex(s: &str) -> Result<String, regex::Error> {
    Regex::new(s).map(|_| s.to_string())
}
