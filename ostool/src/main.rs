use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use clap::*;

use log::info;
use ostool::{
    build,
    ctx::AppContext,
    run::{cargo::CargoRunner, qemu::RunQemuArgs, uboot::RunUbootArgs},
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
    /// Path to the build configuration file
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

    let mut ctx = AppContext {
        workdir,
        ..Default::default()
    };

    match cli.command {
        SubCommands::Build { config } => {
            let config = ctx.perpare_build_config(config).await?;
            match config.system {
                build::config::BuildSystem::Cargo(config) => {
                    let mut cargo = CargoRunner::new("build", false);
                    cargo.run(&mut ctx, config).await?;
                }
                build::config::BuildSystem::Custom(custom_cfg) => {
                    ctx.shell_run_cmd(&custom_cfg.build_cmd)?;
                }
            }
        }
        SubCommands::Run(args) => {
            let config = ctx.perpare_build_config(args.config).await?;
            match config.system {
                build::config::BuildSystem::Cargo(config) => {
                    let mut cargo = CargoRunner::new("run", true);
                    cargo.arg("--");

                    if config.to_bin {
                        cargo.arg("--to-bin");
                    }

                    match args.command {
                        RunSubCommands::Qemu(qemu_args) => {
                            cargo.arg("qemu");
                            if let Some(cfg) = qemu_args.qemu_config {
                                cargo.arg("--config");
                                cargo.arg(cfg.display().to_string());
                            }
                            ctx.debug = qemu_args.debug;

                            if qemu_args.dtb_dump {
                                cargo.arg("--dtb-dump");
                            }
                        }
                        RunSubCommands::Uboot(uboot_args) => {
                            cargo.arg("uboot");
                            if let Some(cfg) = uboot_args.uboot_config {
                                cargo.arg("--config");
                                cargo.arg(cfg.display().to_string());
                            }
                        }
                    }
                    cargo.run(&mut ctx, config).await?;
                }
                build::config::BuildSystem::Custom(custom_cfg) => {
                    ctx.shell_run_cmd(&custom_cfg.build_cmd)?;
                    ctx.set_elf_path(custom_cfg.elf_path.into()).await;
                    info!(
                        "ELF {:?}: {}",
                        ctx.arch,
                        ctx.elf_path.as_ref().unwrap().display()
                    );

                    if custom_cfg.to_bin {
                        ctx.objcopy_output_bin()?;
                    }

                    match args.command {
                        RunSubCommands::Qemu(qemu_args) => {
                            ostool::run::qemu::run_qemu(
                                ctx,
                                RunQemuArgs {
                                    qemu_config: qemu_args.qemu_config,
                                    dtb_dump: qemu_args.dtb_dump,
                                    show_output: true,
                                },
                            )
                            .await?;
                        }
                        RunSubCommands::Uboot(uboot_args) => {
                            ostool::run::uboot::run_uboot(
                                ctx,
                                RunUbootArgs {
                                    config: uboot_args.uboot_config,
                                    show_output: true,
                                },
                            )
                            .await?;
                        }
                    }
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
