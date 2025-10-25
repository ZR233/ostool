use std::{env::current_dir, path::PathBuf, thread::sleep, time::Duration};

use anyhow::Result;
use clap::*;
use colored::Colorize;

mod utils;

mod build;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    workdir: Option<PathBuf>,
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    Build {
        /// Path to the build configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    Run,
    CargoRun,
    Defconfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let workdir = match cli.workdir {
        Some(dir) => dir,
        None => current_dir()?,
    };

    match cli.command {
        SubCommands::Build { config } => {
            println!("Building in directory: {}", workdir.display());
            build::run_build(&workdir, config).await?;
        }
        SubCommands::Run => {
            println!("Running in directory: {}", workdir.display());
            // Run logic goes here
        }
        SubCommands::CargoRun => {
            println!("Cargo running in directory: {}", workdir.display());
            // Cargo run logic goes here
        }
        SubCommands::Defconfig => {
            println!(
                "Generating default config in directory: {}",
                workdir.display()
            );
            // Defconfig logic goes here
        }
    }

    Ok(())
}
