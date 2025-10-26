//! FIT (Flattened Image Tree) 模块
//!
//! 实现U-Boot FIT image格式的创建和处理功能

pub mod builder;
pub mod config;
pub mod dt_builder;

// 重新导出主要类型
pub use builder::FitImageBuilder;
pub use config::{ComponentConfig, FitImageConfig};
pub use dt_builder::DeviceTreeBuilder;
