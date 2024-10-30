use std::{
    fs,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use colored::Colorize;

use crate::{project::Project, shell::Shell, QemuArgs};

pub struct Qemu {}

impl Qemu {
    pub fn run(project: &mut Project, cli: QemuArgs, is_check_test: bool) {
        let mut machine = "virt".to_string();

        if let Some(m) = project.config.qemu.machine.as_ref() {
            machine = m.to_string();
        }

        if cli.dtb {
            let _ = fs::remove_file("target/qemu.dtb");
            machine = format!("{},dumpdtb=target/qemu.dtb", machine);
        }

        let bin_path = project.bin_path.as_ref().unwrap();
        let bin_path = fs::canonicalize(bin_path).unwrap();

        let mut cmd = project.shell(project.arch.qemu_program());
        if !project.config.qemu.graphic {
            cmd.arg("-nographic");
        }
        cmd.args(["-machine", &machine]);

        let more_args = project
            .config
            .qemu
            .args
            .split(" ")
            .map(|o| o.trim())
            .filter(|o| !o.is_empty())
            .collect::<Vec<_>>();

        if !more_args.is_empty() {
            cmd.args(more_args);
        }

        if cli.debug {
            cmd.args(["-s", "-S"]);
        }

        if let Some(cpu) = &project.config.qemu.cpu {
            cmd.arg("-cpu");
            cmd.arg(cpu);
        }
        cmd.arg("-kernel");
        cmd.arg(&bin_path);

        if is_check_test {
            let is_ok = Arc::new(AtomicBool::new(false));
            let is_ok2 = is_ok.clone();
            cmd.exec_with_lines(project.is_print_cmd, move |line| {
                if line.contains("All tests passed") {
                    is_ok2.store(true, Ordering::SeqCst);
                }
                Ok(())
            })
            .unwrap();
            if !is_ok.load(Ordering::SeqCst) {
                println!("{}", "Test failed!".red());
                exit(1);
            }
        } else {
            cmd.exec(project.is_print_cmd).unwrap();
        }
    }
}
