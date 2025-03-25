use serde::{Deserialize, Serialize};

use crate::project::Arch;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Qemu {
    pub machine: Option<String>,
    pub cpu: Option<String>,
    pub graphic: bool,
    pub args: String,
}

impl Qemu {
    pub fn new_default(arch: Arch) -> Self {
        match arch {
            Arch::Aarch64 => Self {
                machine: Some("virt".to_string()),
                cpu: Some("cortex-a57".to_string()),
                graphic: false,
                args: "".to_string(),
            },
            Arch::Riscv64 => Self {
                machine: Some("virt".to_string()),
                cpu: Some("rv64".to_string()),
                graphic: false,
                args: "".to_string(),
            },
            Arch::X86_64 => Self {
                machine: Some("virt".to_string()),
                cpu: Some("qemu64".to_string()),
                graphic: false,
                args: "".to_string(),
            },
        }
    }
}
