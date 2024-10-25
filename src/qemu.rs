use std::fs;

use crate::{project::Project, shell::Shell, QemuArgs};

pub struct Qemu {}

impl Qemu {
    pub fn run(project: &mut Project, cli: QemuArgs) {
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

        let mut cmd = project.shell(project.arch.qemu_arch());
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

        cmd.args(more_args);

        if cli.debug {
            cmd.args(["-s", "-S"]);
        }

        if let Some(cpu) = &project.config.qemu.cpu {
            cmd.arg("-cpu");
            cmd.arg(cpu);
        }
        cmd.arg("-kernel");
        cmd.arg(&bin_path);
        cmd.exec().unwrap();
    }
}
