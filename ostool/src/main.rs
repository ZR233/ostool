use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use clap::*;

use ostool::{
    build,
    ctx::AppContext,
    run::{self, qemu::RunQemuArgs, uboot::RunUbootArgs},
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
}

#[derive(Args, Debug)]
struct RunArgs {
    /// Path to the build configuration file, default to `.config.toml`
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: RunSubCommands,
}

#[derive(Subcommand, Debug)]
enum RunSubCommands {
    Qemu(QemuArgs),
    Uboot(UbootArgs),
}

#[derive(Args, Debug, Default)]
struct QemuArgs {
    /// Path to the qemu configuration file, default to '.qemu.toml'
    #[arg(short, long)]
    qemu_config: Option<PathBuf>,
    #[arg(short, long)]
    debug: bool,
    /// Dump DTB file
    #[arg(long)]
    dtb_dump: bool,
}

#[derive(Args, Debug)]
struct UbootArgs {
    /// Path to the uboot configuration file, default to '.uboot.toml'
    #[arg(short, long)]
    uboot_config: Option<PathBuf>,
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
            let ctx = build::run_build(ctx, args.config.clone()).await?;
            match args.command {
                RunSubCommands::Qemu(args) => {
                    run::qemu::run_qemu(ctx, args.into()).await?;
                }
                RunSubCommands::Uboot(args) => {
                    run::uboot::run_uboot(ctx, args.into()).await?;
                }
            }
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

impl From<UbootArgs> for RunUbootArgs {
    fn from(value: UbootArgs) -> Self {
        RunUbootArgs {
            config: value.uboot_config,
            show_output: true,
        }
    }
}
