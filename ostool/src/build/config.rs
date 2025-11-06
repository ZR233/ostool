use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BuildConfig {
    pub system: BuildSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum BuildSystem {
    Custom(Custom),
    Cargo(Cargo),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Custom {
    /// shell command to build the kernel
    pub build_cmd: String,
    /// path to the built ELF file
    pub elf_path: String,
    /// whether to output as binary
    pub to_bin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Cargo {
    /// target triple
    pub target: String,
    /// package name
    pub package: String,
    /// features to enable
    pub features: Features,
    /// log level feature
    pub log: Option<LogLevel>,
    /// extra cargo .config.toml file
    /// can be url or local path
    pub extra_config: Option<String>,
    /// other cargo args
    pub args: Vec<String>,
    /// shell commands before build
    pub pre_build_cmds: Vec<String>,
    /// shell commands after build
    /// `KERNEL_ELF` env var is set to the built ELF path
    pub post_build_cmds: Vec<String>,
    /// whether to output as binary
    pub to_bin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Features {
    pub self_features: Vec<String>,
    pub depend_features: Vec<Depend>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Depend {
    pub name: String,
    pub d_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
