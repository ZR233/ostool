use std::path::PathBuf;

use colored::Colorize;

use crate::{
    build::config::{Cargo, Custom},
    ctx::AppContext,
};

pub mod config;

impl AppContext {
    pub async fn build_with_config(&mut self, config: &config::BuildConfig) -> anyhow::Result<()> {
        match &config.system {
            config::BuildSystem::Custom(custom) => self.build_custrom(custom)?,
            config::BuildSystem::Cargo(cargo) => {
                self.build_cargo(cargo).await?;
            }
        }
        Ok(())
    }

    pub async fn build(&mut self, config_path: Option<PathBuf>) -> anyhow::Result<()> {
        let build_config = self.perpare_build_config(config_path).await?;
        println!("Build configuration: {:?}", build_config);
        self.build_with_config(&build_config).await
    }

    pub fn build_custrom(&mut self, config: &Custom) -> anyhow::Result<()> {
        self.shell_run_cmd(&config.build_cmd)?;
        Ok(())
    }

    pub async fn build_cargo(&mut self, config: &Cargo) -> anyhow::Result<()> {
        for cmd in &config.pre_build_cmds {
            self.shell_run_cmd(cmd)?;
        }

        let mut features = config.features.self_features.clone();
        if let Some(log_level) = &self.log_level_feature(config) {
            features.push(log_level.to_string());
        }

        let mut cmd = self.command("cargo");
        cmd.arg("build");

        for (k, v) in &config.env {
            cmd.env(k, v);
        }

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
        if !self.debug {
            cmd.arg("--release");
        }
        for (k, v) in cmd.get_envs() {
            println!("{}", format!("{k:?}={v:?}").cyan());
        }
        cmd.run()?;

        let elf_path = self
            .manifest_dir
            .join("target")
            .join(&config.target)
            .join(if self.debug { "debug" } else { "release" })
            .join(&config.package);

        self.set_elf_path(elf_path.clone()).await;

        if config.to_bin {
            self.objcopy_output_bin()?;
        }

        for cmd in &config.post_build_cmds {
            self.shell_run_cmd(cmd)?;
        }

        Ok(())
    }

    fn log_level_feature(&self, config: &Cargo) -> Option<String> {
        let level = config.log.clone()?;

        let meta = self.metadata().ok()?;
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
                if self.debug { "" } else { "release_" },
                format!("{:?}", level).to_lowercase()
            ))
        } else {
            None
        }
    }

    async fn cargo_extra_config(&self, config: &Cargo) -> anyhow::Result<Option<String>> {
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
