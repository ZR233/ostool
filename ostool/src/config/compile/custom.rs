use serde::{Deserialize, Serialize};

use crate::ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomBuild {
    pub shell: Vec<String>,
    pub elf: Option<String>,
    pub kernel: String,
}

impl CustomBuild {
    pub fn new_by_ui() -> Self {
        let shell = vec![ui::shell_input("请输入构建命令：")];

        let mut elf: String = ui::shell_input("请输入elf文件名（空为不需要，用于debug）：");
        elf = elf.trim().to_string();

        let kernel = ui::shell_input("请输入kernel文件名：");

        Self {
            shell,
            elf: if elf.is_empty() { None } else { Some(elf) },
            kernel,
        }
    }
}
