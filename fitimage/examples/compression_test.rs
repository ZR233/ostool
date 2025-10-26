//! Compression test example for mkimage library
//!
//! Demonstrates FIT image creation with gzip compression.

use mkimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create larger sample kernel data to see compression effects
    let kernel_data = "Hello, this is a sample kernel! ".repeat(100);
    let kernel_bytes = kernel_data.as_bytes();

    println!("Original kernel data size: {} bytes", kernel_bytes.len());

    // Configure FIT image with compression
    let config_compressed = FitImageConfig::new("Compressed FIT Image")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_bytes.to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000)
        )
        .with_kernel_compression(true);

    // Build compressed FIT image
    let mut builder = FitImageBuilder::new();
    let fit_data_compressed = builder.build(config_compressed)?;

    println!("Compressed FIT image size: {} bytes", fit_data_compressed.len());

    // Configure FIT image without compression
    let config_uncompressed = FitImageConfig::new("Uncompressed FIT Image")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_bytes.to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000)
        )
        .with_kernel_compression(false);

    // Build uncompressed FIT image
    let fit_data_uncompressed = builder.build(config_uncompressed)?;

    println!("Uncompressed FIT image size: {} bytes", fit_data_uncompressed.len());

    // Compare sizes
    if fit_data_compressed.len() < fit_data_uncompressed.len() {
        let savings = fit_data_uncompressed.len() - fit_data_compressed.len();
        println!("✅ Compression saved {} bytes ({}% reduction)",
                savings,
                (savings * 100) / fit_data_uncompressed.len());
    } else {
        println!("⚠️  Compression did not reduce size (data too small or not compressible)");
    }

    // Verify both have valid device tree magic
    for (name, data) in [("compressed", &fit_data_compressed), ("uncompressed", &fit_data_uncompressed)] {
        if data.len() >= 4 && &data[0..4] == b"\xd0\x0d\xfe\xed" {
            println!("✅ {} FIT image has valid device tree magic", name);
        } else {
            println!("❌ {} FIT image has invalid device tree magic", name);
        }
    }

    Ok(())
}