use clap::{Parser, Subcommand};
use new::check_new;
use packageman::PackageDb;
use report::Reports;
use std::error::Error;
use verify::verify;

mod new;
mod packageman;
mod report;
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
            let packagedb = PackageDb::load_rpmdb(cli.debug)?;

            for file in &packagedb.files {
                println!(
                    "{} (package {})",
                    file.path.display(),
                    packagedb.package_to_string(file.package)
                )
            }
        }
        Commands::Check(check) => {
            let packagedb = PackageDb::load_rpmdb(cli.debug)?;

            let mut reports = Reports::new();

            if !check.nochanged || !check.nomissing {
                verify(&packagedb, &check, &mut reports);
            }

            if !check.nonew {
                check_new(&packagedb, &mut reports);
            }

            reports.sort();

            reports.print();
        }
    }

    Ok(())
}
