use check::{check, CheckArgs};
use clap::{ArgAction, Parser, Subcommand};
use packageman::{PackageDb, PackageMgr};
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

    /// Ignore files matching pattern
    #[clap(short = 'i', long)]
    pub ignore: Vec<String>,
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

            for package in packagedb.packages() {
                println!("{}", package.name_arch())
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

            // Report differences
            check(
                &packagedb,
                CheckArgs {
                    changed: !checkargs.nochanged,
                    missing: !checkargs.nomissing,
                    new: !checkargs.nonew,
                    checksum: checkargs.checksum,
                    ignores: &checkargs.ignore,
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
