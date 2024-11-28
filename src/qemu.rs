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

use crate::{
    project::{Arch, Project},
    shell::Shell,
    QemuArgs,
};

struct QemuParams {
    kernel: PathBuf,
    machine: String,
    cmd: Command,
}

pub struct Qemu {}

impl Qemu {
    pub fn run(project: &mut Project, cli: QemuArgs, is_check_test: bool) {
        let cmd = project.shell(project.arch.unwrap().qemu_program());

        let mut params = QemuParams {
            kernel: PathBuf::new(),
            machine: "virt".to_string(),
            cmd,
        };

        if matches!(project.arch, Some(Arch::X86_64)) {
            params.machine = "q35".to_string();
            params.kernel = project.elf_path.clone().unwrap();
        }

        if let Some(m) = project.config_ref().qemu.machine.as_ref() {
            params.machine = m.to_string();
        }

        if cli.dtb {
            let _ = fs::remove_file("target/qemu.dtb");
            params.machine = format!("{},dumpdtb=target/qemu.dtb", params.machine);
        }

        let bin_path = project.bin_path.as_ref().unwrap();
        params.kernel = fs::canonicalize(bin_path).unwrap();

        #[cfg(target_os = "windows")]
        Self::cmd_windows_env(project, &mut params);

        if !project.config_ref().qemu.graphic {
            params.cmd.arg("-nographic");
        }
        params.cmd.args(["-machine", &params.machine]);

        let more_args = project
            .config_ref()
            .qemu
            .args
            .split(" ")
            .map(|o| o.trim())
            .filter(|o| !o.is_empty())
            .collect::<Vec<_>>();

        if !more_args.is_empty() {
            params.cmd.args(more_args);
        }

        if cli.debug {
            params.cmd.args(["-s", "-S"]);
        }

        if let Some(cpu) = &project.config_ref().qemu.cpu {
            params.cmd.arg("-cpu");
            params.cmd.arg(cpu);
        }
        params.cmd.arg("-kernel");
        params.cmd.arg(&params.kernel);

        if is_check_test {
            let is_ok = Arc::new(AtomicBool::new(false));
            let is_ok2 = is_ok.clone();
            params
                .cmd
                .exec_with_lines(project.is_print_cmd, move |line| {
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
            params.cmd.exec(project.is_print_cmd).unwrap();
        }
    }

    fn cmd_windows_env(project: &mut Project, params: &mut QemuParams) {
        let env = params.cmd.get_envs().collect::<HashMap<_, _>>();
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

        params.cmd.env("PATH", path);
    }
}
