//! # mkimage
//!
//! A Rust implementation of U-Boot's mkimage tool for creating bootable images.
//!
//! This crate provides functionality to create various types of U-Boot compatible images
//! including legacy images, FIT images, and multi-file images.
//!
//! ## Example
//!
//! ```rust
//! use mkimage::{ImageBuilder, ImageType, Arch, OsType};
//!
//! let mut builder = ImageBuilder::new();
//! builder
//!     .image_type(ImageType::Kernel)
//!     .arch(Arch::Aarch64)
//!     .os_type(OsType::Linux)
//!     .load_address(0x80000)
//!     .entry_point(0x80000)
//!     .name("Test Kernel")
//!     .data(&kernel_data)?;
//!
//! let image = builder.build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod image_header;
pub mod image_types;
pub mod crc;
pub mod builder;
pub mod error;
pub mod cli;
// 新增模块
pub mod fit;
pub mod compression;

// Re-export main types for convenience
pub use image_header::ImageHeader;
pub use image_types::*;
pub use builder::ImageBuilder;
pub use error::{MkImageError, Result};
// 新增导出
pub use fit::{FitImage, FitComponent, FitComponentType, FitConfiguration, FitImageBuilder};
pub use compression::traits::CompressionInterface;
pub use crc::calculate_crc32;

/// Current version of the mkimage implementation
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Magic number for U-Boot legacy images
pub const IH_MAGIC: u32 = 0x27051956;

/// Default header size for legacy images
pub const IH_HEADER_SIZE: usize = 60;