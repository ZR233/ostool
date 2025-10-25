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
    pub build_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Cargo {
    /// target triple
    pub target: String,
    /// package name
    pub package: String,
    /// features to enable
    pub features: Vec<String>,
    /// log level feature
    pub log: Option<LogLevel>,
    /// other cargo args
    pub args: Vec<String>,
    /// shell commands before build
    pub pre_build_cmds: Vec<String>,
    /// shell commands after build
    pub post_build_cmds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
