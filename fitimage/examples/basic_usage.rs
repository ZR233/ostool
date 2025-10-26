//! Basic usage example for mkimage library
//!
//! Demonstrates how to create a FIT image with kernel and device tree.

use mkimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample kernel data
    let kernel_data = b"Hello, this is a sample kernel!";

    // Create some sample device tree data
    let fdt_data = b"This is sample device tree data";

    // Configure FIT image
    let config = FitImageConfig::new("Sample FIT Image")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_data.to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000)
        )
        .with_fdt(
            ComponentConfig::new("fdt", fdt_data.to_vec())
                .with_load_address(0x82000000)
        )
        .with_kernel_compression(false);

    // Build FIT image
    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)?;

    println!("FIT image created successfully!");
    println!("Size: {} bytes", fit_data.len());
    println!("First 16 bytes: {:02x?}", &fit_data[..16.min(fit_data.len())]);

    // Verify device tree magic
    if fit_data.len() >= 4 && &fit_data[0..4] == b"\xd0\x0d\xfe\xed" {
        println!("✅ Valid device tree magic number found");
    } else {
        println!("❌ Invalid device tree magic number");
    }

    Ok(())
}