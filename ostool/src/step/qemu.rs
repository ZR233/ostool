use std::{
    fs,
    process::{Command, exit},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use colored::Colorize;

use crate::{QemuArgs, project::Project, shell::Shell};

use super::Step;

pub struct Qemu {
    args: QemuArgs,
    is_check_test: bool,
    machine: String,
    cmd: Command,
}

impl Qemu {
    pub fn new_boxed(cli: QemuArgs, is_check_test: bool) -> Box<dyn Step> {
        Box::new(Self {
            args: cli,
            is_check_test,
            machine: "virt".to_string(),
            cmd: Command::new("ls"),
        })
    }
}

impl Step for Qemu {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        self.cmd = project.shell(project.arch.unwrap().qemu_program());

        let mut config = project.config_ref().qemu.clone();
        config.set_default_by_arch(project.arch.unwrap());

        if let Some(m) = config.machine.as_ref() {
            self.machine = m.to_string();
        }

        if self.args.dtb {
            let _ = fs::remove_file("target/qemu.dtb");
            self.machine = format!("{},dumpdtb=target/qemu.dtb", self.machine);
        }

        if !config.graphic {
            self.cmd.arg("-nographic");
        }
        self.cmd.args(["-machine", &self.machine]);

        let more_args = config
            .args
            .split(" ")
            .map(|o| o.trim())
            .filter(|o| !o.is_empty())
            .collect::<Vec<_>>();

        if !more_args.is_empty() {
            self.cmd.args(more_args);
        }

        if self.args.debug {
            self.cmd.args(["-s", "-S"]);
        }

        if let Some(cpu) = &config.cpu {
            self.cmd.arg("-cpu");
            self.cmd.arg(cpu);
        }
        self.cmd.arg("-kernel");
        self.cmd.arg(project.kernel.as_ref().unwrap());

        if self.is_check_test {
            let test_success = Arc::new(Mutex::new(false));
            let test_success_clone = test_success.clone();

            self.cmd
                .exec_with_lines(project.is_print_cmd, move |line, _child| {
                    if line.contains("All tests passed") {
                        println!("{}", "Test passed!".green());
                        {
                            let mut finished = test_success_clone.lock().unwrap();
                            *finished = true;
                        }

                        // 启动一个线程，1秒后强制退出
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(1));
                            exit(0);
                        });
                    }
                    if line.contains("Test failed") {
                        println!("{}", "Test failed!".red());
                        {
                            let mut finished = test_success_clone.lock().unwrap();
                            *finished = true;
                        }

                        // 启动一个线程，1秒后强制退出
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(1));
                            exit(1);
                        });
                    }
                    Ok(())
                })
                .unwrap();

            if *test_success.lock().unwrap() {
                exit(0);
            } else {
                exit(1);
            }
        } else {
            self.cmd
                .exec(project.is_print_cmd)
                .expect("run qemu failed!");
        }
        Ok(())
    }
}
