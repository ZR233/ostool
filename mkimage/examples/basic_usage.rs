//! Basic usage example for mkimage

use mkimage::{ImageBuilder, ImageType, Arch, OsType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a simple U-Boot image with mkimage...");

    // Create some test data
    let kernel_data = b"Hello, U-Boot! This is a test kernel image.";

    // Build a kernel image
    let builder = ImageBuilder::kernel()
        .arch(Arch::Arm64)
        .load_address(0x80000)
        .entry_point(0x80000)
        .name("Test Kernel")
        .data(kernel_data)?;

    // Generate the image
    let image = builder.build()?;
    std::fs::write("test_kernel.img", &image)?;

    println!("✓ Created test kernel image: test_kernel.img");
    println!("  Size: {} bytes", image.len());
    println!("  Header: 60 bytes");
    println!("  Data: {} bytes", kernel_data.len());

    // List the image information
    let parsed = ImageBuilder::from_image(&image)?;
    println!();
    println!("Image Information:");
    println!("{}", parsed.header().summary());

    // Verify the image
    parsed.validate()?;
    println!("✓ Image validation passed");

    Ok(())
}