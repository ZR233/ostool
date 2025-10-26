use std::{env, path::PathBuf, process::exit};

use clap::Parser;
use ostool::{ctx::AppContext, run::qemu};

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

    #[arg(long("show-output"))]
    show_output: bool,

    #[arg(long)]
    no_run: bool,

    /// Path to the qemu configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Dump DTB file
    #[arg(long)]
    dtb_dump: bool,

    #[arg(allow_hyphen_values = true)]
    /// Arguments to be run
    runner_args: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var("CARGO").is_err() {
        eprintln!("This binary may only be called via `cargo ndk-runner`.");
        exit(1);
    }

    let args = RunnerArgs::parse();

    let workdir = env::var("CARGO_MANIFEST_DIR")?.into();

    let mut app = AppContext {
        workdir,
        debug: args.no_run,
        ..Default::default()
    };

    app.set_elf_path(args.elf).await;

    qemu::run_qemu(
        app,
        qemu::RunQemuArgs {
            qemu_config: args.config,
            dtb_dump: args.dtb_dump,
        },
    )
    .await?;

    Ok(())
}
