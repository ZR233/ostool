use std::{
    alloc::System,
    fs::{self, File},
    path::{Path, PathBuf},
    time::{Instant, SystemTime},
};

use anyhow::bail;

use crate::data::menu::MenuRoot;

pub struct AppData {
    pub root: MenuRoot,
    pub current_key: String,
    pub needs_save: bool,
    pub config: PathBuf,
}

const DEFAULT_CONFIG_PATH: &str = ".project.toml";

fn default_schema_by_init(config: &Path) -> PathBuf {
    let binding = config.file_name().unwrap().to_string_lossy();
    let mut name_split = binding.split(".").collect::<Vec<_>>();
    if name_split.len() > 1 {
        name_split.pop();
    }

    let name = format!("{}-schema.json", name_split.join("."));

    if let Some(parent) = config.parent() {
        parent.join(name)
    } else {
        PathBuf::from(name)
    }
}

impl AppData {
    pub fn new(
        config: Option<impl AsRef<Path>>,
        schema: Option<impl AsRef<Path>>,
    ) -> anyhow::Result<Self> {
        let mut init_value_path = PathBuf::from(DEFAULT_CONFIG_PATH);
        if let Some(cfg) = config {
            init_value_path = cfg.as_ref().to_path_buf();
        }

        let schema_path = if let Some(sch) = schema {
            sch.as_ref().to_path_buf()
        } else {
            default_schema_by_init(&init_value_path)
        };

        if !schema_path.exists() {
            bail!("Schema file does not exist: {}", schema_path.display());
        }

        let schema_content = fs::read_to_string(&schema_path)?;
        let schema_json: serde_json::Value = serde_json::from_str(&schema_content)?;

        let mut root = MenuRoot::try_from(&schema_json)?;

        if init_value_path.exists() {
            let init_content = fs::read_to_string(&init_value_path)?;
            if !init_content.trim().is_empty() {
                let ext = init_value_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let init_json: serde_json::Value = match ext {
                    "json" => serde_json::from_str(&init_content)?,
                    "toml" => {
                        let v: toml::Value = toml::from_str(&init_content)?;
                        serde_json::to_value(v)?
                    }
                    _ => {
                        bail!("Unsupported config file extension: {ext:?}");
                    }
                };
                root.update_by_value(&init_json)?;
            }
        }

        Ok(AppData {
            root,
            current_key: String::new(),
            needs_save: false,
            config: init_value_path,
        })
    }

    pub fn on_exit(&mut self) -> anyhow::Result<()> {
        if self.needs_save && self.config.exists() {
            let ext = self
                .config
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let bk = format!(
                "bk-{:?}.{ext}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs()
            );

            let backup_path = self.config.with_extension(bk);
            fs::copy(&self.config, &backup_path)?;

            let json_value = self.root.as_json();

            let s = match ext {
                "toml" | "tml" => toml::to_string_pretty(&json_value)?,
                "json" => serde_json::to_string_pretty(&json_value)?,
                _ => {
                    bail!("Unsupported config file extension: {}", ext);
                }
            };
            fs::write(&self.config, s)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_default() {
        let name = "config.toml";
        let expected_schema_name = "config-schema.json";
        let schema_path = default_schema_by_init(Path::new(name));
        assert_eq!(schema_path, PathBuf::from(expected_schema_name));
    }
}
