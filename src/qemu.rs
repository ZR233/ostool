use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
    process::{exit, Command},
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

        if let Some(m) = project.config_ref().qemu.machine.as_ref() {
            machine = m.to_string();
        }

        if cli.dtb {
            let _ = fs::remove_file("target/qemu.dtb");
            machine = format!("{},dumpdtb=target/qemu.dtb", machine);
        }

        let bin_path = project.bin_path.as_ref().unwrap();
        let bin_path = fs::canonicalize(bin_path).unwrap();

        let mut cmd = project.shell(project.arch.unwrap().qemu_program());

        #[cfg(target_os = "windows")]
        Self::cmd_windows_env(&mut cmd);

        if !project.config_ref().qemu.graphic {
            cmd.arg("-nographic");
        }
        cmd.args(["-machine", &machine]);

        let more_args = project
            .config_ref()
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

        if let Some(cpu) = &project.config_ref().qemu.cpu {
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

    fn cmd_windows_env(cmd: &mut Command) {
        let env = cmd.get_envs().collect::<HashMap<_, _>>();
        let mut mysys2_root = PathBuf::from("C:\\msys64");
        if let Some(p) = std::env::var_os("MSYS2_ROOT") {
            mysys2_root = PathBuf::from(p);
        }

        let mut path = env
            .get(OsStr::new("PATH"))
            .unwrap_or(&None)
            .unwrap_or_default()
            .to_os_string();
        if !path.is_empty() {
            path.push(";");
        }

        let ucrt64 = mysys2_root.join("ucrt64/bin");

        if ucrt64.join("qemu-system-x86_64.exe").exists() {
            path.push(OsString::from(ucrt64));
        }

        let mingw64 = mysys2_root.join("mingw64/bin");

        if mingw64.join("qemu-system-x86_64.exe").exists() {
            path.push(mingw64);
        }

        cmd.env("PATH", path);
    }
}
