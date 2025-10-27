use std::{env, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use log::LevelFilter;
use ostool::{
    ctx::AppContext,
    run::{
        qemu,
        uboot::{self, RunUbootArgs},
    },
};

#[derive(Debug, Parser, Clone)]
struct RunnerArgs {
    #[arg(short)]
    /// Enable verbose output
    verbose: bool,

    #[arg(short)]
    /// Enable quiet output (no output except errors)
    quiet: bool,

    program: PathBuf,

    /// Path to the binary to run on the device
    elf: PathBuf,

    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug, Clone)]
enum SubCommands {
    Qemu {
        /// Path to the configuration file, default to '.qemu.toml'
        #[arg(short, long)]
        config: Option<PathBuf>,
        /// Dump DTB file
        #[arg(long)]
        dtb_dump: bool,

        #[arg(long("show-output"))]
        show_output: bool,

        #[arg(long)]
        no_run: bool,

        #[arg(allow_hyphen_values = true)]
        /// Arguments to be run
        runner_args: Vec<String>,
    },
    Uboot {
        /// Path to the configuration file, default to '.uboot.toml'
        #[arg(short, long)]
        config: Option<PathBuf>,

        #[arg(long("show-output"))]
        show_output: bool,

        #[arg(allow_hyphen_values = true)]
        /// Arguments to be run
        runner_args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_module_path(false)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    if env::var("CARGO").is_err() {
        eprintln!("This binary may only be called via `cargo ndk-runner`.");
        exit(1);
    }

    let args = RunnerArgs::parse();

    let workdir = env::var("CARGO_MANIFEST_DIR")?.into();

    let mut app = AppContext {
        workdir,
        ..Default::default()
    };

    app.set_elf_path(args.elf).await;

    match args.command {
        SubCommands::Qemu {
            config,
            dtb_dump,
            no_run,
            show_output,
            ..
        } => {
            app.debug = no_run;
            qemu::run_qemu(
                app,
                qemu::RunQemuArgs {
                    qemu_config: config,
                    dtb_dump,
                    show_output,
                },
            )
            .await?;
        }
        SubCommands::Uboot {
            config,
            show_output,
            ..
        } => {
            uboot::run_uboot(
                app,
                RunUbootArgs {
                    config,
                    show_output,
                },
            )
            .await?;
        }
    }

    Ok(())
}
