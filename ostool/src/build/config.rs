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
    pub target: String,
    pub package: String,
    pub features: Vec<String>,
    pub args: Vec<String>,
    /// 构建前执行的命令列表
    pub pre_build_cmd: Vec<String>,
    /// 构建后执行的命令列表
    pub post_build_cmd: Vec<String>,
}
