use clap::{Parser, Subcommand};
use new::check_new;
use report::Reports;
use rpmdb::RpmDb;
use std::error::Error;
use verify::verify;

mod new;
mod report;
mod rpmdb;
mod verify;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

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

#[derive(Parser, Default)]
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

    /// Don't check file digests
    #[arg(short = 'd', long)]
    nodigest: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let command = match cli.command {
        Some(c) => c,
        None => Commands::Check(Check::default()),
    };

    match command {
        Commands::List => {
            let rpmdb = RpmDb::load(cli.debug)?;

            for file in &rpmdb.files {
                println!(
                    "{} (package {})",
                    file.path.display(),
                    rpmdb.rpm_to_string(file.rpm)
                )
            }
        }
        Commands::Check(check) => {
            let rpmdb = RpmDb::load(cli.debug)?;

            let mut reports = Reports::new();

            if !check.nochanged || !check.nomissing {
                verify(&rpmdb, &check, &mut reports);
            }

            if !check.nonew {
                check_new(&rpmdb, &mut reports);
            }

            reports.sort();

            reports.print();
        }
    }

    Ok(())
}
