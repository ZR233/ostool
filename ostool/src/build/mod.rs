use std::path::PathBuf;

use anyhow::anyhow;
use jkconfig::data::app_data::default_schema_by_init;
use tokio::fs;

use crate::{ctx::AppContext, utils::ShellRunner};

pub mod config;

pub async fn run_build(
    ctx: AppContext,
    config_path: Option<PathBuf>,
) -> anyhow::Result<AppContext> {
    // Build logic will be implemented here
    let config_path = match config_path {
        Some(path) => path,
        None => ctx.workdir.join(".config.toml"),
    };

    let schema_path = default_schema_by_init(&config_path);

    let schema = schemars::schema_for!(config::BuildConfig);
    let schema_json = serde_json::to_value(&schema)?;
    let schema_content = serde_json::to_string_pretty(&schema_json)?;
    fs::write(&schema_path, schema_content).await?;

    // 初始化AppData
    // let app_data = AppData::new(Some(&config_path), Some(schema_path))?;

    let config_content = fs::read_to_string(&config_path)
        .await
        .map_err(|_| anyhow!("can not open config file: {}", config_path.display()))?;

    let build_config: config::BuildConfig = toml::from_str(&config_content)?;

    println!("Build configuration: {:?}", build_config);

    match build_config.system {
        config::BuildSystem::Custom(custom) => {
            let ctx = CtxCustom {
                ctx,
                config: custom,
            };
            ctx.run().await
        }
        config::BuildSystem::Cargo(cargo) => {
            let ctx = CtxCargo { ctx, config: cargo };
            ctx.run().await
        }
    }
}

struct CtxCustom {
    ctx: AppContext,
    config: config::Custom,
}

impl CtxCustom {
    async fn run(self) -> anyhow::Result<AppContext> {
        self.ctx.shell_run_cmd(&self.config.build_cmd)?;
        Ok(self.ctx)
    }
}

struct CtxCargo {
    ctx: AppContext,
    config: config::Cargo,
}

impl CtxCargo {
    async fn run(mut self) -> anyhow::Result<AppContext> {
        for cmd in &self.config.pre_build_cmds {
            self.ctx.shell_run_cmd(cmd)?;
        }

        let mut features = self.config.features.clone();
        if let Some(log_level) = &self.log_level_feature() {
            features.push(log_level.to_string());
        }

        let mut cmd = self.ctx.command("cargo");
        cmd.arg("build");

        if let Some(extra_config_path) = self.cargo_extra_config().await? {
            cmd.arg("--config");
            cmd.arg(extra_config_path);
        }

        cmd.arg("-p");
        cmd.arg(&self.config.package);
        cmd.arg("--target");
        cmd.arg(&self.config.target);
        cmd.arg("-Z");
        cmd.arg("unstable-options");
        if !features.is_empty() {
            cmd.arg("--features");
            cmd.arg(features.join(","));
        }
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        if !self.ctx.debug {
            cmd.arg("--release");
        }

        cmd.run()?;

        let elf_path = self
            .ctx
            .workdir
            .join("target")
            .join(&self.config.target)
            .join(if self.ctx.debug { "debug" } else { "release" })
            .join(&self.config.package);

        self.ctx.set_elf_path(elf_path.clone()).await;

        if self.config.to_bin {
            self.ctx.objcopy_output_bin()?;
        }

        for cmd in &self.config.post_build_cmds {
            self.ctx.shell_run_cmd(cmd)?;
        }

        Ok(self.ctx)
    }

    fn log_level_feature(&self) -> Option<String> {
        let level = self.config.log.clone()?;

        let meta = self.ctx.metadata().ok()?;
        let pkg = meta
            .packages
            .iter()
            .find(|p| p.name == self.config.package)?;
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
                if self.ctx.debug { "" } else { "release_" },
                format!("{:?}", level).to_lowercase()
            ))
        } else {
            None
        }
    }

    async fn cargo_extra_config(&self) -> anyhow::Result<Option<String>> {
        let s = match self.config.extra_config.as_ref() {
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
            Ok(Some(s.clone()))
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

    async fn download_config_to_temp(&self, url: &str) -> anyhow::Result<String> {
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

        Ok(target_path.to_string_lossy().to_string())
    }
}
