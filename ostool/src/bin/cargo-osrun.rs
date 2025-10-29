use std::{env, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use log::{LevelFilter, debug};
use ostool::{
    ctx::AppContext,
    run::{
        qemu,
        uboot::{self, RunUbootArgs},
    },
};

#[derive(Debug, Parser, Clone)]
struct RunnerArgs {
    program: PathBuf,

    /// Path to the binary to run on the device
    elf: PathBuf,

    /// Test name
    test_name: Option<String>,

    /// Objcopy elf to binary before running
    #[arg(long("to-bin"))]
    to_bin: bool,

    #[arg(short)]
    /// Enable verbose output
    verbose: bool,

    #[arg(short)]
    /// Enable quiet output (no output except errors)
    quiet: bool,

    /// Path to the runner configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(long("show-output"))]
    show_output: bool,

    #[arg(long)]
    no_run: bool,

    /// Sub-commands
    #[command(subcommand)]
    command: Option<SubCommands>,

    /// Dump DTB file
    #[arg(long)]
    dtb_dump: bool,

    #[arg(allow_hyphen_values = true)]
    /// Arguments to be run
    runner_args: Vec<String>,
}

#[derive(Debug, Subcommand, Clone)]
enum SubCommands {
    Uboot(CliUboot),
}

#[derive(Debug, Parser, Clone)]
struct CliUboot {
    #[arg(allow_hyphen_values = true)]
    runner_args: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_module_path(false)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = RunnerArgs::parse();

    debug!("Parsed arguments: {:?}", args);

    if env::var("CARGO").is_err() {
        eprintln!("This binary may only be called via `cargo ndk-runner`.");
        exit(1);
    }

    let workdir = env::var("CARGO_MANIFEST_DIR")?.into();

    let mut app = AppContext {
        workdir,
        ..Default::default()
    };

    app.set_elf_path(args.elf).await;

    app.debug = args.no_run;
    if args.to_bin {
        app.objcopy_output_bin()?;
    }

    match args.command {
        Some(SubCommands::Uboot(_)) => {
            uboot::run_uboot(
                app,
                RunUbootArgs {
                    config: args.config,
                    show_output: args.show_output,
                },
            )
            .await?;
        }
        None => {
            qemu::run_qemu(
                app,
                qemu::RunQemuArgs {
                    qemu_config: args.config,
                    dtb_dump: args.dtb_dump,
                    show_output: args.show_output,
                },
            )
            .await?;
        }
    }

    Ok(())
}
