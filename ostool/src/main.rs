use std::{env::current_dir, path::PathBuf, thread::sleep, time::Duration};

use anyhow::Result;
use clap::*;
use colored::Colorize;
use project::Project;
use step::{CargoTestPrepare, Compile, Qemu, Step, Tftp, Uboot, UbootConfig};

mod cmd;
mod config;
mod env;
mod os;
mod project;
mod shell;
mod step;
mod ui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    workdir: Option<String>,
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    Build,
    Run(RunArgs),
    CargoTest(TestArgs),
    Defconfig(cmd::defconfig::Cmd),
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
struct TestArgs {
    elf: String,
    mode: Option<String>,
    #[arg(long)]
    show_output: bool,
    #[arg(long)]
    uboot: bool,
    #[arg(long)]
    no_run: bool,
}

#[derive(Args, Debug, Default)]
struct QemuArgs {
    #[arg(short, long)]
    debug: bool,
    #[arg(long)]
    dtb: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let workdir = cli
        .workdir
        .map(PathBuf::from)
        .unwrap_or(current_dir().unwrap());

    env::prepere_deps();

    let mut keep_run = false;

    let mut project = Project::new(workdir);
    project.prepere_deps();

    let mut steps: Vec<Box<dyn Step>> = vec![];

    match cli.command {
        SubCommands::Build => {
            project.config_with_file().unwrap();
            steps.push(Compile::new_boxed(false));
        }
        SubCommands::CargoTest(args) => {
            project.is_print_cmd = false;

            steps.push(CargoTestPrepare::new_boxed(args.elf, args.uboot));
            if args.uboot {
                steps.push(Tftp::new_boxed());
                steps.push(Uboot::new_boxed(true));
            } else {
                steps.push(Qemu::new_boxed(
                    QemuArgs {
                        debug: args.no_run,
                        dtb: false,
                    },
                    true,
                ));
            }
        }
        SubCommands::Run(run_args) => {
            project.config_with_file().unwrap();

            match run_args.command {
                RunSubCommands::Qemu(args) => {
                    steps.push(Compile::new_boxed(args.debug));
                    steps.push(Qemu::new_boxed(args, false));
                }
                RunSubCommands::Uboot => {
                    steps.push(Compile::new_boxed(false));
                    steps.push(Tftp::new_boxed());

                    let config = project.config.as_mut().unwrap();
                    if config.uboot.is_none() {
                        config.uboot = Some(UbootConfig::config_by_select());
                        project.save_config();
                    }

                    steps.push(Uboot::new_boxed(false));
                }
                RunSubCommands::Tftp => {
                    steps.push(Compile::new_boxed(false));
                    steps.push(Tftp::new_boxed());
                    keep_run = true;
                }
            };
        }
        SubCommands::Defconfig(cmd) => {
            cmd.run(&mut project);
        }
    }

    for step in &mut steps {
        if let Err(skip) = step.run(&mut project) {
            println!("{}", format!("warn: {}", skip).yellow());
        }
    }

    if keep_run {
        loop {
            sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}
