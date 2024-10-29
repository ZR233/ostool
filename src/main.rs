use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::*;
use compile::Compile;
use project::Project;
use qemu::Qemu;
use test::CargoTest;

mod compile;
mod config;
mod os;
mod project;
mod qemu;
mod shell;
mod test;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    workdir: Option<String>,
    #[arg(short, long)]
    config: Option<String>,
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Build,
    Qemu(QemuArgs),
    Uboot,
    CargoTest(TestArgs),
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
    let mut workdir = PathBuf::from(cli.workdir.unwrap_or("./".to_string()));
    workdir = fs::canonicalize(&workdir)
        .unwrap_or_else(|_| panic!("Workdir not found: {}", workdir.display()));

    let mut project = Project::new(workdir, cli.config)?;

    project.prepere_deps();
    match cli.command {
        SubCommands::Build => {
            Compile::run(&mut project, false);
        }
        SubCommands::Qemu(args) => {
            Compile::run(&mut project, args.debug);
            Qemu::run(&mut project, args);
        }
        SubCommands::Uboot => {}
        SubCommands::CargoTest(args) => {
            project.is_print_cmd = false;
            CargoTest::run(&mut project, args.elf);
            Qemu::run(
                &mut project,
                QemuArgs {
                    debug: false,
                    dtb: false,
                },
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }

    #[test]
    fn it_works2() {
        assert_eq!(0, 1);
    }
}
