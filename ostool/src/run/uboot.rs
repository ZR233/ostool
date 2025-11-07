use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::Context;
use byte_unit::Byte;
use colored::Colorize;
use fitimage::{ComponentConfig, FitImageBuilder, FitImageConfig};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use jkconfig::data::app_data::default_schema_by_init;
use log::{info, warn};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::fs;
use uboot_shell::UbootShell;

use crate::{ctx::AppContext, run::tftp, sterm::SerialTerm, utils::replace_env_placeholders};

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
    pub baud_rate: String,
    pub dtb_file: Option<String>,
    /// Kernel load address
    /// if not specified, use U-Boot env variable 'loadaddr'
    pub kernel_load_addr: Option<String>,
    /// TFTP boot configuration
    pub net: Option<Net>,
    /// Board reset command
    /// shell command to reset the board
    pub board_reset_cmd: Option<String>,
    /// Board power off command
    /// shell command to power off the board
    pub board_power_off_cmd: Option<String>,
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

pub async fn run_uboot(ctx: AppContext, args: RunUbootArgs) -> anyhow::Result<()> {
    // Build logic will be implemented here
    let config_path = match args.config.clone() {
        Some(path) => path,
        None => ctx.workspace_folder.join(".uboot.toml"),
    };

    let schema_path = default_schema_by_init(&config_path);

    let schema = schemars::schema_for!(UbootConfig);
    let schema_json = serde_json::to_value(&schema)?;
    let schema_content = serde_json::to_string_pretty(&schema_json)?;
    fs::write(&schema_path, schema_content).await?;

    // 初始化AppData
    // let app_data = AppData::new(Some(&config_path), Some(schema_path))?;

    let config = if config_path.exists() {
        let mut config_content = fs::read_to_string(&config_path)
            .await
            .map_err(|_| anyhow!("can not open config file: {}", config_path.display()))?;

        config_content = replace_env_placeholders(&config_content)?;

        let config: UbootConfig = toml::from_str(&config_content)?;
        config
    } else {
        let config = UbootConfig {
            serial: "/dev/ttyUSB0".to_string(),
            baud_rate: "115200".into(),
            ..Default::default()
        };

        fs::write(&config_path, toml::to_string_pretty(&config)?).await?;
        config
    };

    let baud_rate = config
        .baud_rate
        .parse::<u32>()
        .with_context(|| anyhow!("baud_rate is not valid int"))?;

    let mut runner = Runner {
        ctx,
        config,
        baud_rate,
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
    baud_rate: u32,
}

impl Runner {
    /// 生成压缩的 FIT image 包含 kernel 和 FDT
    ///
    /// # 参数
    /// - `kernel_path`: kernel 文件路径
    /// - `dtb_path`: DTB 文件路径（可选）
    /// - `kernel_load_addr`: kernel 加载地址
    ///
    /// # 返回值
    /// 返回生成的 FIT image 文件路径
    async fn generate_fit_image(
        &self,
        kernel_path: &Path,
        dtb_path: Option<&Path>,
        kernel_load_addr: u64,
        kernel_entry_addr: u64,
        fdt_load_addr: Option<u64>,
        _ramfs_load_addr: Option<u64>,
    ) -> anyhow::Result<PathBuf> {
        info!("Making FIT image...");
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
            "kernel: {} (size: {:.2})",
            kernel_path.display(),
            Byte::from(kernel_data.len())
        );

        let arch = match self.ctx.arch.as_ref().unwrap() {
            object::Architecture::Aarch64 => "arm64",
            object::Architecture::Arm => "arm",
            object::Architecture::LoongArch64 => "loongarch64",
            _ => todo!(),
        };

        // 创建配置，与 test.its 文件中的参数一致
        let mut config = FitImageConfig::new("Various kernels, ramdisks and FDT blobs")
            .with_kernel(
                ComponentConfig::new("kernel", kernel_data)
                    .with_description("This kernel")
                    .with_type("kernel")
                    .with_arch(arch)
                    .with_os("linux")
                    .with_compression(true)
                    .with_load_address(kernel_load_addr)
                    .with_entry_point(kernel_entry_addr),
            );
        let mut fdt_name = None;

        // 处理 DTB 文件
        if let Some(dtb_path) = dtb_path {
            match fs::read(dtb_path).await {
                Ok(data) => {
                    info!(
                        "已读取 DTB 文件: {} (大小: {:.2})",
                        dtb_path.display(),
                        Byte::from(data.len())
                    );
                    fdt_name = Some("fdt");

                    // Can not compress DTB, U-Boot will not accept it
                    let mut fdt_config = ComponentConfig::new("fdt", data.clone())
                        .with_description("This fdt")
                        .with_type("flat_dt")
                        .with_arch(arch);

                    if let Some(addr) = fdt_load_addr {
                        fdt_config = fdt_config.with_load_address(addr);
                    }

                    config = config.with_fdt(fdt_config);
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
        }

        config = config
            .with_default_config("config-ostool")
            .with_configuration(
                "config-ostool",
                "ostool configuration",
                Some("kernel"),
                fdt_name,
                None::<String>,
            );

        // 使用新的 mkimage API 构建 FIT image
        let mut builder = FitImageBuilder::new();
        let fit_data = builder
            .build(config)
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

        info!("FIT image ok: {}", output_path.display());
        Ok(output_path)
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        let res = self._run().await;
        if let Some(ref cmd) = self.config.board_power_off_cmd
            && !cmd.trim().is_empty()
        {
            info!("Executing board power off command: {}", cmd);
            let _ = self.ctx.shell_run_cmd(cmd);
            info!("Board powered off");
        }
        res
    }

    async fn _run(&mut self) -> anyhow::Result<()> {
        self.preper_regex()?;
        self.ctx.objcopy_output_bin()?;

        let kernel = self.ctx.bin_path.as_ref().ok_or(anyhow!("bin not exist"))?;

        info!("Starting U-Boot runner...");

        info!("kernel from: {}", kernel.display());

        let ip_string = self.detect_tftp_ip();

        if let Some(ip) = ip_string.as_ref() {
            info!("TFTP server IP: {}", ip);
            tftp::run_tftp_server(&self.ctx)?;
        }

        info!(
            "Opening serial port: {} @ {}",
            self.config.serial, self.baud_rate
        );

        let rx = serialport::new(&self.config.serial, self.baud_rate as _)
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

        if let Some(cmd) = self.config.board_reset_cmd.clone()
            && !cmd.trim().is_empty()
        {
            info!("Executing board reset command: {}", cmd);
            self.ctx.shell_run_cmd(&cmd)?;
        }

        let mut uboot = handle.join().unwrap()?;

        if let Some(ref ip) = ip_string
            && let Ok(output) = uboot.cmd("net list")
        {
            let device_list = output.strip_prefix("net list").unwrap_or(&output).trim();

            if device_list.is_empty() {
                let _ = uboot.cmd("bootdev hunt ethernet");
            }

            info!("Board network ok");

            if let Some(ref board_ip) = self.config.net.as_ref().unwrap().board_ip {
                uboot.set_env("ipaddr", board_ip)?;
            } else {
                uboot.cmd("dhcp")?;
            }

            uboot.set_env("serverip", ip.clone())?;
        }

        let mut fdt_load_addr = None;
        let mut ramfs_load_addr = None;

        if let Ok(addr) = uboot.env_int("fdt_addr_r") {
            fdt_load_addr = Some(addr as u64);
        }

        if let Ok(addr) = uboot.env_int("ramdisk_addr_r") {
            ramfs_load_addr = Some(addr as u64);
        }

        let loadaddr = if let Ok(addr) = uboot.env_int("loadaddr") {
            info!("Found $loadaddr: {addr:#x}");
            addr as u64
        } else if let Ok(addr) = uboot.env_int("kernel_comp_addr_r") {
            uboot.set_env("loadaddr", format!("{addr:#x}"))?;
            info!("Set $loadaddr to kernel_comp_addr_r: {addr:#x}");
            addr as u64
        } else {
            let addr = uboot.env_int("kernel_addr_c")? as u64;
            uboot.set_env("loadaddr", format!("{addr:#x}"))?;
            info!("Set $loadaddr to kernel_addr_c: {addr:#x}");
            addr
        };

        let kernel_entry = if let Some(entry) = self.config.kernel_load_addr_int() {
            info!("Using configured kernel load address: {entry:#x}");
            entry
        } else {
            uboot
                .env_int("kernel_addr_r")
                .expect("kernel_addr_r not found") as u64
        };

        info!("fitimage loadaddr: {loadaddr:#x}");
        info!("kernel entry: {kernel_entry:#x}");
        let dtb = self.config.dtb_file.clone();
        if let Some(ref dtb_file) = dtb {
            info!("Using DTB from: {}", dtb_file);
        }

        let dtb_path = dtb.as_ref().map(Path::new);
        let fitimage = self
            .generate_fit_image(
                kernel,
                dtb_path,
                kernel_entry,
                kernel_entry,
                fdt_load_addr,
                ramfs_load_addr,
            )
            .await?;

        if self.config.net.is_some() {
            info!("TFTP upload FIT image to board...");
            let filename = fitimage.file_name().unwrap().to_str().unwrap();

            let tftp_cmd = format!("tftp {filename}");
            uboot.cmd(&tftp_cmd)?;
            uboot.cmd_without_reply("bootm")?;
        } else {
            info!("No TFTP config, using loady to upload FIT image...");
            Self::uboot_loady(&mut uboot, loadaddr as usize, fitimage);
            uboot.cmd_without_reply("bootm")?;
        }

        let tx = uboot.tx.take().unwrap();
        let rx = uboot.rx.take().unwrap();

        drop(uboot);

        println!("{}", "Interacting with U-Boot shell...".green());

        let success_regex = self.success_regex.clone();
        let fail_regex = self.fail_regex.clone();

        let res = Arc::new(Mutex::<Option<anyhow::Result<()>>>::new(None));
        let res_clone = res.clone();
        let mut shell = SerialTerm::new(tx, rx, move |h, line| {
            for regex in success_regex.iter() {
                if regex.is_match(line) {
                    println!("{}", "\r\n=== SUCCESS PATTERN MATCHED ===".green());
                    h.stop();
                    let mut res_lock = res_clone.lock().unwrap();
                    *res_lock = Some(Ok(()));
                    return;
                }
            }

            for regex in fail_regex.iter() {
                if regex.is_match(line) {
                    println!("{}", "\r\n=== FAIL PATTERN MATCHED ===".red());
                    h.stop();
                    let mut res_lock = res_clone.lock().unwrap();
                    *res_lock = Some(Err(anyhow!("Fail pattern matched: {}", line)));
                    return;
                }
            }
        });
        shell.run().await?;
        {
            let mut res_lock = res.lock().unwrap();
            if let Some(result) = res_lock.take() {
                result?;
            }
        }
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

    fn detect_tftp_ip(&self) -> Option<String> {
        let net = self.config.net.as_ref()?;

        let mut ip_string = String::new();

        let interfaces = NetworkInterface::show().unwrap();
        for interface in interfaces.iter() {
            if interface.name == net.interface {
                let addr_list: Vec<Addr> = interface.addr.to_vec();
                for one in addr_list {
                    if let Addr::V4(v4_if_addr) = one {
                        ip_string = v4_if_addr.ip.to_string();
                    }
                }
            }
        }

        if ip_string.trim().is_empty() {
            return None;
        }

        info!("TFTP : {}", ip_string);

        Some(ip_string)
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
