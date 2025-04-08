use serde::{Deserialize, Serialize};

use crate::project::Arch;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Qemu {
    pub machine: Option<String>,
    pub cpu: Option<String>,
    pub graphic: bool,
    pub args: String,
}

impl Qemu {
    pub fn new_default(arch: Arch) -> Self {
        let mut s = Self::default();
        s.set_default_by_arch(arch);
        s
    }

    pub fn set_default_by_arch(&mut self, arch: Arch) {
        match arch {
            Arch::Aarch64 => {
                if self.cpu.is_none() {
                    self.cpu = Some("cortex-a57".to_string());
                }
                if self.machine.is_none() {
                    self.machine = Some("virt".to_string());
                }
            }
            Arch::X86_64 => {
                if self.machine.is_none() {
                    self.machine = Some("q35".to_string());
                }
            }
            Arch::Riscv64 => {
                if self.machine.is_none() {
                    self.machine = Some("virt".to_string());
                }
            }
        }
    }
}
