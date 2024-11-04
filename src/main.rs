use std::path::PathBuf;

use anyhow::Result;
use clap::*;
use compile::Compile;
use project::Project;
use qemu::Qemu;
use test::CargoTest;
use uboot::Uboot;

mod compile;
mod config;
mod os;
mod project;
mod qemu;
mod shell;
mod test;
mod uboot;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    workdir: Option<String>,
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Build,
    Run(RunArgs),
    CargoTest(TestArgs),
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
}

#[derive(Args, Debug, Default)]
struct TestArgs {
    elf: String,
    mode: Option<String>,
    #[arg(long)]
    show_output: bool,
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
    let workdir = PathBuf::from(cli.workdir.unwrap_or("./".to_string()));

    let mut project = Project::new(workdir);
    project.prepere_deps();
    match cli.command {
        SubCommands::Build => {
            project.config_with_file().unwrap();
            Compile::run(&mut project, false);
        }

        SubCommands::CargoTest(args) => {
            project.is_print_cmd = false;
            CargoTest::run(&mut project, args.elf);
            Qemu::run(
                &mut project,
                QemuArgs {
                    debug: false,
                    dtb: false,
                },
                true,
            );
        }
        SubCommands::Run(run_args) => {
            project.config_with_file().unwrap();
            match run_args.command {
                RunSubCommands::Qemu(args) => {
                    Compile::run(&mut project, args.debug);
                    Qemu::run(&mut project, args, false);
                }
                RunSubCommands::Uboot => {
                    Compile::run(&mut project, false);

                    let config = project.config.as_mut().unwrap();
                    if config.uboot.is_none() {
                        config.uboot = Some(uboot::UbootConfig::config_by_select());
                        project.save_config();
                    }

                    Uboot::run(&mut project);
                }
            };
        }
    }

    Ok(())
}
