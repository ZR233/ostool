use std::{env::current_dir, path::PathBuf, thread::sleep, time::Duration};

use anyhow::Result;
use clap::*;
use colored::Colorize;

use crate::ctx::AppContext;

mod build;
mod ctx;
mod utils;

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

    let ctx = AppContext {
        workdir,
        debug: false,
    };

    match cli.command {
        SubCommands::Build { config } => {
            build::run_build(ctx, config).await?;
        }
        SubCommands::Run => {
            // Run logic goes here
        }
        SubCommands::CargoRun => {
            // Cargo run logic goes here
        }
        SubCommands::Defconfig => {

            // Defconfig logic goes here
        }
    }

    Ok(())
}
