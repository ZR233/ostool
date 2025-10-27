//! FIT (Flattened Image Tree) 模块
//!
//! 实现U-Boot FIT image格式的创建和处理功能

pub mod builder;
pub mod config;
pub mod dt_builder;
pub mod fdt_header;
pub mod fdt_tokens;
pub mod standard_dt_builder;
pub mod string_table;

// 重新导出主要类型
pub use builder::FitImageBuilder;
pub use config::{ComponentConfig, FitImageConfig};
pub use dt_builder::DeviceTreeBuilder;
pub use fdt_header::{FdtHeader, MemReserveEntry, FDT_MAGIC, FDT_VERSION, FDT_LAST_COMP_VERSION};
pub use fdt_tokens::{FdtToken, FdtTokenUtils, FDT_STRUCT_ALIGN};
pub use standard_dt_builder::StandardFdtBuilder;
pub use string_table::StringTable;
