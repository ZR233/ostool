//! # mkimage
//!
//! A Rust library for creating U-Boot compatible FIT (Flattened Image Tree) images.
//!
//! This crate provides functionality to create FIT images that are fully compatible with
//! U-Boot's FIT image format. It supports kernel, device tree (FDT), and ramdisk components
//! with optional gzip compression.
//!
//! ## Example
//!
//! ```rust
//! use mkimage::{FitImageBuilder, FitImageConfig, ComponentConfig};
//!
//! let config = FitImageConfig {
//!     description: "FIT Image".to_string(),
//!     kernel: Some(ComponentConfig {
//!         name: "kernel".to_string(),
//!         data: kernel_data,
//!         load_address: Some(0x80080000),
//!         entry_point: Some(0x80080000),
//!     }),
//!     fdt: None,
//!     ramdisk: None,
//!     compress_kernel: true,
//! };
//!
//! let fit_data = FitImageBuilder::new().build(config)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod fit;
pub mod compression;
pub mod crc;
pub mod error;

// Re-export main types for convenience
pub use fit::{FitImageBuilder, FitImageConfig, ComponentConfig};
pub use compression::traits::CompressionInterface;
pub use crc::calculate_crc32;
pub use error::{MkImageError, Result};

/// Current version of the mkimage implementation
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// FIT image magic number
pub const FIT_MAGIC: &[u8] = b"FIT";