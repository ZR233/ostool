use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Qemu {
    pub machine: Option<String>,
    pub cpu: Option<String>,
    pub graphic: bool,
    pub args: String,
}
