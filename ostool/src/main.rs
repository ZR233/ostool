use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use clap::*;

use ostool::{
    build,
    ctx::AppContext,
    run::{self, qemu::RunQemuArgs},
};

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
    Run(RunArgs),
    CargoRun,
}

#[derive(Args, Debug)]
struct RunArgs {
    #[command(subcommand)]
    command: RunSubCommands,
}

#[derive(Subcommand, Debug)]
enum RunSubCommands {
    Qemu(QemuArgs),
    Uboot,
    Tftp,
}

#[derive(Args, Debug, Default)]
struct QemuArgs {
    #[arg(short, long)]
    build_config: Option<PathBuf>,
    #[arg(short, long)]
    qemu_config: Option<PathBuf>,
    #[arg(short, long)]
    debug: bool,
    /// Dump DTB file
    #[arg(long)]
    dtb_dump: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .format_module_path(false)
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let workdir = match cli.workdir {
        Some(dir) => dir,
        None => current_dir()?,
    };

    let ctx = AppContext {
        workdir,
        ..Default::default()
    };

    match cli.command {
        SubCommands::Build { config } => {
            build::run_build(ctx, config).await?;
        }
        SubCommands::Run(args) => {
            // match args.command {
            //     RunSubCommands::Qemu(args) => {
            //         let ctx = build::run_build(ctx, args.build_config.clone()).await?;
            //         run::qemu::run_qemu(ctx, args.into()).await?;
            //     }
            //     RunSubCommands::Uboot => todo!(),
            //     RunSubCommands::Tftp => todo!(),
            // }

            // Run logic goes here
        }
        SubCommands::CargoRun => {
            // Cargo run logic goes here
        }
    }

    Ok(())
}

impl From<QemuArgs> for RunQemuArgs {
    fn from(value: QemuArgs) -> Self {
        RunQemuArgs {
            qemu_config: value.qemu_config,
            dtb_dump: value.dtb_dump,
            show_output: true,
        }
    }
}
