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
}

#[derive(Subcommand)]
enum Commands {
    /// Checks the rpm database against the files on the filesystem
    Check {
        /// Skip the matched file verification stage
        #[arg(short = 'v', long)]
        noverify: bool,

        /// Skip the new file verification stage
        #[arg(short = 'n', long)]
        nonew: bool,
    },
    List,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let rpmdb = load_rpm_database()?;

            for file in &rpmdb.files {
                println!(
                    "{}  (from {})",
                    file.path.display(),
                    rpmdb.rpm_to_string(file.rpm)
                )
            }
        }
        Commands::Check { noverify, nonew } => {
            let rpmdb = load_rpm_database()?;

            if !noverify {
                verify(&rpmdb);
            }

            if !nonew {
                check_new(&rpmdb);
            }
        }
    }

    Ok(())
}
