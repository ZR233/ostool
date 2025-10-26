use std::{path::PathBuf, thread, time::Duration};

use crossterm::terminal::{disable_raw_mode, is_raw_mode_enabled};
use jkconfig::data::app_data::default_schema_by_init;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use tokio::{fs, spawn, task::spawn_blocking};
use uboot_shell::UbootShell;

use crate::ctx::AppContext;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct UbootConfig {
    /// Serial console device
    /// e.g., /dev/ttyUSB0 on linux, COM3 on Windows
    pub serial: String,
    pub baud_rate: i64,
    pub dtb_file: String,
    /// Kernel load address
    /// if not specified, use U-Boot env variable 'loadaddr'
    pub kernel_load_addr: Option<String>,
    /// TFTP boot configuration
    pub net: Option<Net>,
    /// U-Boot reset command
    /// shell command to reset the board
    pub reset_cmd: Option<String>,
    pub success_regex: Vec<String>,
    pub fail_regex: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Net {
    pub interface: String,
    pub board_ip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunUbootArgs {
    pub config: Option<PathBuf>,
    pub show_output: bool,
}

pub async fn run_qemu(ctx: AppContext, args: RunUbootArgs) -> anyhow::Result<()> {
    // Build logic will be implemented here
    let config_path = match args.config.clone() {
        Some(path) => path,
        None => ctx.workdir.join(".uboot.toml"),
    };

    let schema_path = default_schema_by_init(&config_path);

    let schema = schemars::schema_for!(UbootConfig);
    let schema_json = serde_json::to_value(&schema)?;
    let schema_content = serde_json::to_string_pretty(&schema_json)?;
    fs::write(&schema_path, schema_content).await?;

    // 初始化AppData
    // let app_data = AppData::new(Some(&config_path), Some(schema_path))?;

    let config = if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)
            .await
            .map_err(|_| anyhow!("can not open config file: {}", config_path.display()))?;
        let config: UbootConfig = toml::from_str(&config_content)?;
        config
    } else {
        let config = UbootConfig {
            serial: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
            ..Default::default()
        };

        fs::write(&config_path, toml::to_string_pretty(&config)?).await?;
        config
    };

    let mut runner = Runner {
        ctx,
        config,
        success_regex: vec![],
        fail_regex: vec![],
    };
    runner.run().await?;
    Ok(())
}

struct Runner {
    ctx: AppContext,
    config: UbootConfig,
    success_regex: Vec<regex::Regex>,
    fail_regex: Vec<regex::Regex>,
}

impl Runner {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.preper_regex()?;
        self.ctx.objcopy_output_bin()?;

        let kernel = self.ctx.bin_path.as_ref().ok_or(anyhow!("bin not exist"))?;

        info!("Starting U-Boot runner...");
        info!("Loading kernel from: {}", kernel.display());

        let rx = serialport::new(&self.config.serial, self.config.baud_rate as _)
            .timeout(Duration::from_millis(200))
            .open()
            .map_err(|e| anyhow!("Failed to open serial port: {e}"))?;
        let tx = rx
            .try_clone()
            .map_err(|e| anyhow!("Failed to clone serial port: {e}"))?;

        println!("Waiting for board on power or reset...");
        let handle: thread::JoinHandle<anyhow::Result<UbootShell>> = thread::spawn(move || {
            let uboot = UbootShell::new(tx, rx)?;
            Ok(uboot)
        });

        if let Some(cmd) = self.config.reset_cmd.clone() {
            info!("Executing board reset command: {}", cmd);
            self.ctx.shell_run_cmd(&cmd)?;
        }

        let mut uboot = handle.join().unwrap()?;

        drop(uboot);

        println!("Interacting with U-Boot shell...");
        let rx = serialport::new(&self.config.serial, self.config.baud_rate as _)
            .timeout(Duration::from_millis(200))
            .open()
            .map_err(|e| anyhow!("Failed to open serial port: {e}"))?;
        let tx = rx
            .try_clone()
            .map_err(|e| anyhow!("Failed to clone serial port: {e}"))?;
        let mut shell = SerialShell::new(tx, rx);

        Ok(())
    }

    fn preper_regex(&mut self) -> anyhow::Result<()> {
        // Prepare regex patterns if needed
        // Compile success regex patterns
        for pattern in self.config.success_regex.iter() {
            // Compile and store the regex
            let regex =
                regex::Regex::new(pattern).map_err(|e| anyhow!("success regex error: {e}"))?;
            self.success_regex.push(regex);
        }

        // Compile fail regex patterns
        for pattern in self.config.fail_regex.iter() {
            // Compile and store the regex
            let regex = regex::Regex::new(pattern).map_err(|e| anyhow!("fail regex error: {e}"))?;
            self.fail_regex.push(regex);
        }

        Ok(())
    }
}

struct SerialShell {
    tx: Box<dyn SerialPort>,
    rx: Box<dyn SerialPort>,
}

impl SerialShell {
    fn new(tx: Box<dyn SerialPort>, rx: Box<dyn SerialPort>) -> Self {
        SerialShell { tx, rx }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        // Implement serial shell interaction logic here

        Ok(())
    }
}

impl Drop for SerialShell {
    fn drop(&mut self) {}
}
