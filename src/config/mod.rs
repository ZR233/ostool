use compile::Compile;
use qemu::Qemu;
use serde::{Deserialize, Serialize};

pub mod compile;
pub mod qemu;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub compile: Compile,
    pub qemu: Qemu,
}
