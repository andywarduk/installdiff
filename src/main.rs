use clap::{Parser, Subcommand};
use new::check_new;
use rpmdb::load_rpm_database;
use std::error::Error;
use verify::verify;

mod new;
mod rpmdb;
mod verify;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Print debugging messages
    #[arg(short = 'd', long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Checks the rpm database against the files on the filesystem
    Check(Check),
    /// Prints a list of files in the RPM database
    List,
}

#[derive(Parser)]
struct Check {
    /// Don't check for changed files
    #[arg(short = 'c', long)]
    nochanged: bool,

    /// Don't check for missing files
    #[arg(short = 'm', long)]
    nomissing: bool,

    /// Don't check for new files
    #[arg(short = 'n', long)]
    nonew: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let rpmdb = load_rpm_database(cli.debug)?;

            for file in &rpmdb.files {
                println!(
                    "{}  (from {})",
                    file.path.display(),
                    rpmdb.rpm_to_string(file.rpm)
                )
            }
        }
        Commands::Check(check) => {
            let rpmdb = load_rpm_database(cli.debug)?;

            if !check.nochanged || !check.nomissing {
                verify(&rpmdb, !check.nochanged, !check.nomissing);
            }

            if !check.nonew {
                check_new(&rpmdb);
            }
        }
    }

    Ok(())
}
