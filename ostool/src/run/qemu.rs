use anyhow::anyhow;
use jkconfig::data::app_data::default_schema_by_init;
use object::Object;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{QemuArgs, ctx::AppContext, utils::ShellRunner};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct QemuConfig {
    pub arch: Option<String>,
    pub args: Vec<String>,
}

pub async fn run_qemu(ctx: AppContext, args: &QemuArgs) -> anyhow::Result<()> {
    // Build logic will be implemented here
    let config_path = match args.qemu_config.clone() {
        Some(path) => path,
        None => ctx.workdir.join(".qemu.toml"),
    };

    let schema_path = default_schema_by_init(&config_path);

    let schema = schemars::schema_for!(QemuConfig);
    let schema_json = serde_json::to_value(&schema)?;
    let schema_content = serde_json::to_string_pretty(&schema_json)?;
    fs::write(&schema_path, schema_content).await?;

    // 初始化AppData
    // let app_data = AppData::new(Some(&config_path), Some(schema_path))?;

    let config = if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)
            .await
            .map_err(|_| anyhow!("can not open config file: {}", config_path.display()))?;
        let config: QemuConfig = toml::from_str(&config_content)?;
        config
    } else {
        let mut config = QemuConfig::default();
        config.args.push("-nographic".to_string());
        if let Some(arch) = ctx.arch {
            match arch {
                object::Architecture::Aarch64 => {
                    config.args.push("-cpu".to_string());
                    config.args.push("cortex-a53".to_string());
                }
                _ => {}
            }
        }
        fs::write(&config_path, toml::to_string_pretty(&config)?).await?;
        config
    };

    let mut runner = QemuRunner {
        ctx,
        config,
        args: vec![],
        dtbdump: args.dtb_dump,
    };
    runner.run().await?;
    Ok(())
}

struct QemuRunner {
    ctx: AppContext,
    config: QemuConfig,
    args: Vec<String>,
    dtbdump: bool,
}

impl QemuRunner {
    async fn run(&mut self) -> anyhow::Result<()> {
        let arch = self.detect_arch()?;

        let mut machine = "virt".to_string();

        for arg in &self.config.args {
            if arg == "-machine" || arg == "-M" {
                machine = arg.clone();
                continue;
            }
            self.args.push(arg.clone());
        }

        if self.dtbdump {
            let _ = fs::remove_file("target/qemu.dtb").await;
            machine = format!("{},dumpdtb=target/qemu.dtb", machine);
        }

        let mut cmd = self.ctx.command(&format!("qemu-system-{arch}"));
        for arg in &self.config.args {
            cmd.arg(arg);
        }

        cmd.arg("-machine").arg(machine);

        if self.ctx.debug {
            cmd.arg("-s").arg("-S");
        }

        if let Some(bin_path) = &self.ctx.bin_path {
            cmd.arg("-kernel").arg(bin_path);
        } else if let Some(elf_path) = &self.ctx.elf_path {
            cmd.arg("-kernel").arg(elf_path);
        }

        cmd.run()?;

        // QEMU execution logic goes here
        Ok(())
    }

    fn detect_arch(&self) -> anyhow::Result<String> {
        if let Some(arch) = &self.config.arch {
            return Ok(arch.clone());
        }

        if let Some(arch) = &self.ctx.arch {
            return Ok(format!("{:?}", arch).to_lowercase());
        }

        Err(anyhow!(
            "Please specify `arch` in QEMU config or provide a valid ELF file."
        ))
    }
}
