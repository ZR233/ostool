//! FIT (Flattened Image Tree) 模块
//!
//! 实现U-Boot FIT image格式的创建和处理功能

pub mod types;
pub mod builder;
pub mod serializer;

// 重新导出主要类型
pub use types::{FitImage, FitComponent, FitComponentType, FitConfiguration};
pub use builder::FitImageBuilder;