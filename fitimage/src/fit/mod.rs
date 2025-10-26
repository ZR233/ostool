//! FIT (Flattened Image Tree) 模块
//!
//! 实现U-Boot FIT image格式的创建和处理功能

pub mod config;
pub mod builder;
pub mod dt_builder;

// 重新导出主要类型
pub use config::{FitImageConfig, ComponentConfig};
pub use builder::FitImageBuilder;
pub use dt_builder::DeviceTreeBuilder;