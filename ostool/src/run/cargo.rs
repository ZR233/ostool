use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crossterm::style::Stylize;

use crate::{build::config::Cargo, ctx::AppContext};

pub enum CargoRunnerKind {
    Qemu {
        qemu_config: Option<PathBuf>,
        debug: bool,
        dtb_dump: bool,
    },
    Uboot {
        uboot_config: Option<PathBuf>,
    },
}

impl AppContext {
    pub async fn cargo_run(
        &mut self,
        config: &Cargo,
        runner: &CargoRunnerKind,
    ) -> anyhow::Result<()> {
        let build_config_path = self.build_config_path.clone().unwrap();

        let mut cargo = CargoRunner::new("run", true, &build_config_path);
        for (k, v) in &config.env {
            cargo.env(k, v);
        }

        cargo.arg("--");

        if config.to_bin {
            cargo.arg("--to-bin");
        }

        match runner {
            CargoRunnerKind::Qemu {
                qemu_config,
                debug,
                dtb_dump,
            } => {
                if let Some(cfg) = qemu_config {
                    cargo.arg("--config");
                    cargo.arg(cfg.display().to_string());
                }
                self.debug = *debug;

                if *dtb_dump {
                    cargo.arg("--dtb-dump");
                }
                cargo.arg("qemu");
            }
            CargoRunnerKind::Uboot { uboot_config } => {
                if let Some(cfg) = uboot_config {
                    cargo.arg("--config");
                    cargo.arg(cfg.display().to_string());
                }
                cargo.arg("uboot");
            }
        }
        cargo.run(self, config).await
    }
}

pub struct CargoRunner {
    pub cmd: String,
    pub args: Vec<String>,

    skip_objcopy: bool,

    envs: HashMap<String, String>,

    config_path: PathBuf,
}

impl CargoRunner {
    pub fn new(cmd: &str, skip_objcopy: bool, build_config_path: &Path) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: vec![],
            skip_objcopy,
            envs: HashMap::new(),
            config_path: build_config_path.to_path_buf(),
        }
    }

    pub fn arg(&mut self, arg: impl Into<String>) {
        self.args.push(arg.into());
    }

    pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.envs.insert(key.into(), value.into());
    }

    pub async fn run(&mut self, ctx: &mut AppContext, config: &Cargo) -> anyhow::Result<()> {
        for cmd in &config.pre_build_cmds {
            ctx.shell_run_cmd(cmd)?;
        }

        let mut features = config.features.self_features.clone();
        if let Some(log_level) = &self.log_level_feature(ctx, &config) {
            features.push(log_level.to_string());
        }

        let mut cmd = ctx.command("cargo");
        for (k, v) in &self.envs {
            println!("{}", format!("{k}={v}").cyan());
            cmd.env(k, v);
        }

        cmd.arg(&self.cmd);

        if let Some(extra_config_path) = self.cargo_extra_config(config).await? {
            cmd.arg("--config");
            cmd.arg(extra_config_path);
        }

        cmd.arg("-p");
        cmd.arg(&config.package);
        cmd.arg("--target");
        cmd.arg(&config.target);
        cmd.arg("-Z");
        cmd.arg("unstable-options");
        if !features.is_empty() {
            cmd.arg("--features");
            cmd.arg(features.join(","));
        }
        for arg in &config.args {
            cmd.arg(arg);
        }
        if !ctx.debug {
            cmd.arg("--release");
        }

        for arg in &self.args {
            cmd.arg(arg);
        }

        cmd.run()?;

        let elf_path = ctx
            .manifest_dir
            .join("target")
            .join(&config.target)
            .join(if ctx.debug { "debug" } else { "release" })
            .join(&config.package);

        ctx.set_elf_path(elf_path.clone()).await;

        if config.to_bin && !self.skip_objcopy {
            ctx.objcopy_output_bin()?;
        }

        for cmd in &config.post_build_cmds {
            ctx.shell_run_cmd(cmd)?;
        }

        Ok(())
    }

    fn log_level_feature(&self, ctx: &AppContext, config: &Cargo) -> Option<String> {
        let level = config.log.clone()?;

        let meta = ctx.metadata().ok()?;
        let pkg = meta.packages.iter().find(|p| p.name == config.package)?;
        let mut has_log = false;
        for dep in &pkg.dependencies {
            if dep.name == "log" {
                has_log = true;
                break;
            }
        }
        if has_log {
            Some(format!(
                "log/{}max_level_{}",
                if ctx.debug { "" } else { "release_" },
                format!("{:?}", level).to_lowercase()
            ))
        } else {
            None
        }
    }

    async fn cargo_extra_config(&self, config: &Cargo) -> anyhow::Result<Option<PathBuf>> {
        let s = match config.extra_config.as_ref() {
            Some(s) => s,
            None => return Ok(None),
        };

        // Check if it's a URL (starts with http:// or https://)
        if s.starts_with("http://") || s.starts_with("https://") {
            // Convert GitHub URL to raw content URL if needed
            let download_url = Self::convert_to_raw_url(s);

            // Download to temp directory
            match self.download_config_to_temp(&download_url).await {
                Ok(path) => Ok(Some(path)),
                Err(e) => {
                    eprintln!("Failed to download config from {}: {}", s, e);
                    Err(e)
                }
            }
        } else {
            // It's a local path, return as is
            let extra = Path::new(s);
            if extra.is_relative() {
                let combined = self.config_path.parent().unwrap().join(extra);
                Ok(Some(combined))
            } else {
                Ok(Some(s.into()))
            }
        }
    }

    /// Convert GitHub URL to raw content URL
    /// Supports:
    /// - https://github.com/user/repo/blob/branch/path/file -> https://raw.githubusercontent.com/user/repo/branch/path/file
    /// - https://raw.githubusercontent.com/... (already raw, no change)
    /// - Other URLs: no change
    fn convert_to_raw_url(url: &str) -> String {
        // Already a raw URL
        if url.contains("raw.githubusercontent.com") || url.contains("raw.github.com") {
            return url.to_string();
        }

        // Convert github.com/user/repo/blob/... to raw.githubusercontent.com/user/repo/...
        if url.contains("github.com") && url.contains("/blob/") {
            let converted = url
                .replace("github.com", "raw.githubusercontent.com")
                .replace("/blob/", "/");
            println!("Converting GitHub URL to raw: {} -> {}", url, converted);
            return converted;
        }

        // Not a GitHub URL or already in correct format
        url.to_string()
    }

    async fn download_config_to_temp(&self, url: &str) -> anyhow::Result<PathBuf> {
        use std::time::SystemTime;

        println!("Downloading cargo config from: {}", url);

        // Get system temp directory
        let temp_dir = std::env::temp_dir();

        // Generate filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Extract filename from URL or use default
        let url_path = url.split('/').next_back().unwrap_or("config.toml");
        let filename = format!("cargo_config_{}_{}", timestamp, url_path);
        let target_path = temp_dir.join(filename);

        // Create reqwest client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        // Build request with User-Agent for GitHub
        let mut request = client.get(url);

        if url.contains("github.com") || url.contains("githubusercontent.com") {
            // GitHub requires User-Agent
            request = request.header("User-Agent", "ostool-cargo-downloader");
        }

        // Download the file
        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to download from {}: {}", url, e))?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error {}: {}", response.status(), url));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        // Write to temp file
        tokio::fs::write(&target_path, content)
            .await
            .map_err(|e| anyhow!("Failed to write to temp file: {}", e))?;

        println!("Config downloaded to: {}", target_path.display());

        Ok(target_path)
    }
}
