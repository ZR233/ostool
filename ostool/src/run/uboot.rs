use std::{
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use byte_unit::Byte;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use jkconfig::data::app_data::default_schema_by_init;
use log::{info, warn};
use mkimage::{fit::builder::convenience, image_types::Compression};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use tokio::fs;
use uboot_shell::UbootShell;

use crate::{ctx::AppContext, sterm::SerialTerm};

/// FIT image 生成相关的错误消息常量
mod errors {
    pub const KERNEL_READ_ERROR: &str = "读取 kernel 文件失败";
    pub const DTB_READ_ERROR: &str = "读取 DTB 文件失败";
    pub const FIT_BUILD_ERROR: &str = "构建 FIT image 失败";
    pub const FIT_SAVE_ERROR: &str = "保存 FIT image 失败";
    pub const DIR_ERROR: &str = "无法获取 kernel 文件目录";
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct UbootConfig {
    /// Serial console device
    /// e.g., /dev/ttyUSB0 on linux, COM3 on Windows
    pub serial: String,
    pub baud_rate: i64,
    pub dtb_file: Option<String>,
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

impl UbootConfig {
    pub fn kernel_load_addr_int(&self) -> Option<u64> {
        self.kernel_load_addr.as_ref().and_then(|addr_str| {
            if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
                u64::from_str_radix(&addr_str[2..], 16).ok()
            } else {
                addr_str.parse::<u64>().ok()
            }
        })
    }
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
    /// 生成压缩的 FIT image 包含 kernel 和 FDT
    ///
    /// # 参数
    /// - `kernel_path`: kernel 文件路径
    /// - `dtb_path`: DTB 文件路径（可选）
    /// - `output_dir`: 输出目录
    /// - `kernel_load_addr`: kernel 加载地址
    ///
    /// # 返回值
    /// 返回生成的 FIT image 文件路径
    async fn generate_fit_image(
        &self,
        kernel_path: &Path,
        dtb_path: Option<&Path>,
        kernel_load_addr: u64,
    ) -> anyhow::Result<PathBuf> {
        info!("begin gen FIT image...");
        // 生成压缩的 FIT image
        let output_dir = kernel_path
            .parent()
            .and_then(|p| p.to_str())
            .ok_or(anyhow!(errors::DIR_ERROR))?;

        // 读取 kernel 数据
        let kernel_data = fs::read(kernel_path).await.map_err(|e| {
            anyhow!(
                "{} {}: {}",
                errors::KERNEL_READ_ERROR,
                kernel_path.display(),
                e
            )
        })?;

        info!(
            "已读取 kernel 文件: {} (大小: {:.2})",
            kernel_path.display(),
            Byte::from(kernel_data.len())
        );

        // 处理 DTB 文件
        let fdt_result = if let Some(dtb_path) = dtb_path {
            match fs::read(dtb_path).await {
                Ok(data) => {
                    info!(
                        "已读取 DTB 文件: {} (大小: {:.2})",
                        dtb_path.display(),
                        Byte::from(data.len())
                    );
                    Some(data)
                }
                Err(e) => {
                    return Err(anyhow!(
                        "{} {}: {}",
                        errors::DTB_READ_ERROR,
                        dtb_path.display(),
                        e
                    ));
                }
            }
        } else {
            warn!("未指定 DTB 文件，将生成仅包含 kernel 的 FIT image");
            None
        };

        // 计算 FDT 加载地址（kernel 加载地址 + 32MB，标准偏移）
        let (fdt_data, fdt_load_addr) = if let Some(data) = fdt_result {
            (data, kernel_load_addr + 0x02000000)
        } else {
            (vec![], 0) // 未使用时不关心
        };

        // 使用 mkimage 创建压缩的 FIT
        let fit_builder = convenience::compressed_kernel_fdt(
            kernel_data,
            fdt_data,
            kernel_load_addr,
            kernel_load_addr, // entry = loadaddr
            fdt_load_addr,
            Compression::Gzip,
        );

        // 构建 FIT 数据
        let fit_data = fit_builder
            .build()
            .map_err(|e| anyhow!("{}: {}", errors::FIT_BUILD_ERROR, e))?;

        // 保存到文件
        let output_path = Path::new(output_dir).join("image.fit");
        fs::write(&output_path, fit_data).await.map_err(|e| {
            anyhow!(
                "{} {}: {}",
                errors::FIT_SAVE_ERROR,
                output_path.display(),
                e
            )
        })?;

        info!("FIT image 生成成功: {}", output_path.display());
        Ok(output_path)
    }

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

        if self.config.net.is_some()
            && let Ok(output) = uboot.cmd("net list")
        {
            let device_list = output.strip_prefix("net list").unwrap_or(&output).trim();

            if device_list.is_empty() {
                let _ = uboot.cmd_without_reply("bootdev hunt ethernet");
            }
        }

        let loadaddr = self.config.kernel_load_addr_int().unwrap_or_else(|| {
            uboot
                .env_int("loadaddr")
                .map(|o| o as u64)
                .unwrap_or_else(|_e| {
                    info!("$loadaddr not found");

                    let loadaddr = uboot
                        .env_int("kernel_addr_r")
                        .expect("kernel_addr_r not found");
                    uboot.set_env("loadaddr", format!("{loadaddr:#x}")).unwrap();
                    info!("$loadaddr set to {:#x} (kernel_addr_r)", loadaddr);
                    loadaddr as u64
                })
        });

        info!("kernel load addr: {loadaddr:#x}");
        let dtb = self.config.dtb_file.clone();
        if let Some(ref dtb_file) = dtb {
            info!("Using DTB from: {}", dtb_file);
        }

        let dtb_path = dtb.as_ref().map(Path::new);
        let fitimage = self.generate_fit_image(kernel, dtb_path, loadaddr).await?;

        Self::uboot_loady(&mut uboot, loadaddr as usize, fitimage);

        drop(uboot);

        println!("Interacting with U-Boot shell...");
        let rx = serialport::new(&self.config.serial, self.config.baud_rate as _)
            .timeout(Duration::from_millis(200))
            .open()
            .map_err(|e| anyhow!("Failed to open serial port: {e}"))?;
        let tx = rx
            .try_clone()
            .map_err(|e| anyhow!("Failed to clone serial port: {e}"))?;

        let mut shell = SerialTerm::new(tx, rx);
        shell.run().await?;

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

    fn uboot_loady(uboot: &mut UbootShell, addr: usize, file: impl Into<PathBuf>) {
        println!("{}", "\r\nsend file".green());

        let pb = ProgressBar::new(100);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn core::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        let res = uboot
            .loady(addr, file, |x, a| {
                pb.set_length(a as _);
                pb.set_position(x as _);
            })
            .unwrap();

        pb.finish_with_message("upload done");

        println!("{}", res);
        println!("send ok");
    }
}
