use std::{env::current_dir, path::PathBuf, thread::sleep, time::Duration};

use anyhow::Result;
use clap::*;
use colored::Colorize;
use project::Project;
use step::{TestPrepare, Compile, Qemu, Step, Tftp, Uboot, UbootConfig};
use cmd::{Cli, SubCommands, RunArgs, RunSubCommands, TestArgs, QemuArgs};

mod cmd;
mod config;
mod env;
mod project;
mod shell;
mod step;
mod ui;

fn main() -> Result<()> {
    env_logger::builder()
        .format_module_path(false)
        .filter_level(log::LevelFilter::Info)
        .init();

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
        SubCommands::Test(args) => {
            // Handle cargo test compatibility
            let elf = if args.elf.is_some() {
                args.elf
            } else if !args.trailing.is_empty() {
                // Extract ELF from cargo test arguments if provided
                extract_elf_from_cargo_args(&args.trailing)
            } else {
                None
            };

            // Load test configuration using unified loader
            project.test_config(elf.clone(), args.board).unwrap();

            // Use unified test preparation
            project.is_print_cmd = false;
            steps.push(TestPrepare::new_boxed(elf.clone(), args.uboot, elf.is_some()));

            if let Some(_elf_path) = elf {
                // Cargo test mode - specific execution handled by TestPrepare
                if args.uboot {
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
            } else {
                // Standard test mode
                let use_uboot = args.uboot || args.mode.as_ref().map_or(false, |m| m == "uboot");

                steps.push(Compile::new_boxed(args.no_run));
                if use_uboot {
                    let config = project.config.as_mut().unwrap();
                    if config.uboot.is_none() {
                        config.uboot = Some(UbootConfig::config_by_select());
                        project.save_config();
                    }
                    steps.push(Uboot::new_boxed(false));
                } else {
                    steps.push(Qemu::new_boxed(
                        QemuArgs {
                            debug: args.no_run,
                            dtb: false,
                        },
                        false,
                    ));
                }
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

/// Extract ELF path from cargo test arguments
fn extract_elf_from_cargo_args(args: &[String]) -> Option<String> {
    // Look for patterns like --test <name> or test binary paths
    for (i, arg) in args.iter().enumerate() {
        if arg == "--test" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        // Handle direct binary paths
        if arg.ends_with(".elf") || arg.contains("target/") {
            return Some(arg.clone());
        }
    }
    None
}
