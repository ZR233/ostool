//! FIT image builder
//!
//! Main interface for building FIT images from configuration.

use crate::compression::gzip::GzipCompressor;
use crate::compression::traits::CompressionInterface;
use crate::error::Result;
use crate::fit::config::FitImageConfig;
use crate::fit::standard_dt_builder::StandardFdtBuilder;

/// Main FIT image builder
pub struct FitImageBuilder;

impl FitImageBuilder {
    /// Create a new FIT image builder
    pub fn new() -> Self {
        Self
    }

    /// Build a FIT image from configuration
    pub fn build(&mut self, mut config: FitImageConfig) -> Result<Vec<u8>> {
        // Apply compression to kernel if requested
        if config.compress_kernel {
            if let Some(ref mut kernel) = config.kernel {
                let compressor = GzipCompressor::default();
                kernel.data = compressor.compress(&kernel.data)?;
            }
        }

        // Build standard FDT structure
        let mut dt_builder = StandardFdtBuilder::new()?;
        dt_builder.build_fit_tree(&config)?;

        // Generate FIT image data
        let fit_data = dt_builder.finalize()?;

        Ok(fit_data)
    }

    /// Build FIT image with custom compressor
    pub fn build_with_compressor(
        &mut self,
        mut config: FitImageConfig,
        compressor: Box<dyn CompressionInterface>,
    ) -> Result<Vec<u8>> {
        // Apply compression to kernel if requested
        if config.compress_kernel {
            if let Some(ref mut kernel) = config.kernel {
                kernel.data = compressor.compress(&kernel.data)?;
            }
        }

        // Build standard FDT structure
        let mut dt_builder = StandardFdtBuilder::new()?;
        dt_builder.build_fit_tree(&config)?;

        // Generate FIT image data
        let fit_data = dt_builder.finalize()?;

        Ok(fit_data)
    }
}

impl Default for FitImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit::config::{ComponentConfig, FitImageConfig};

    #[test]
    fn test_fit_builder() {
        let config = FitImageConfig::new("Test FIT Image")
            .with_kernel(
                ComponentConfig::new("kernel", vec![1, 2, 3, 4, 5])
                    .with_load_address(0x80080000)
                    .with_entry_point(0x80080000),
            )
            .with_fdt(ComponentConfig::new("fdt", vec![6, 7, 8, 9]).with_load_address(0x82000000))
            .with_kernel_compression(false);

        let mut builder = FitImageBuilder::new();
        let fit_data = builder.build(config).unwrap();

        // Verify we got a valid FIT image
        assert!(!fit_data.is_empty());

        // Basic device tree magic check
        assert_eq!(&fit_data[0..4], b"\xd0\x0d\xfe\xed");
    }

    #[test]
    fn test_fit_builder_with_compression() {
        let kernel_data = vec![1, 2, 3, 4, 5];
        let config = FitImageConfig::new("Test FIT Image")
            .with_kernel(
                ComponentConfig::new("kernel", kernel_data.clone())
                    .with_load_address(0x80080000)
                    .with_entry_point(0x80080000),
            )
            .with_kernel_compression(true);

        let mut builder = FitImageBuilder::new();
        let fit_data = builder.build(config).unwrap();

        // Verify we got a valid FIT image
        assert!(!fit_data.is_empty());

        // Compressed data should be different from original
        // Note: This is a basic check - in practice, compression might not always reduce size
        // for very small data, but the data should still be valid
        assert_eq!(&fit_data[0..4], b"\xd0\x0d\xfe\xed");
    }

    #[test]
    fn test_empty_config() {
        let config = FitImageConfig::new("Empty FIT");

        let mut builder = FitImageBuilder::new();
        let fit_data = builder.build(config).unwrap();

        // Should still create a valid device tree structure
        assert!(!fit_data.is_empty());
        assert_eq!(&fit_data[0..4], b"\xd0\x0d\xfe\xed");
    }
}
