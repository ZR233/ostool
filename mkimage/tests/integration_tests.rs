//! Integration tests for mkimage

use mkimage::{ImageBuilder, ImageType, Arch, OsType, Compression, IH_MAGIC, IH_HEADER_SIZE};
use mkimage::crc::calculate_crc32;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

/// Test basic image creation and parsing
#[test]
fn test_basic_image_creation() {
    let test_data = b"Hello, U-Boot!";

    let builder = ImageBuilder::new()
        .image_type(ImageType::Kernel)
        .arch(Arch::Arm64)
        .os_type(OsType::Linux)
        .load_address(0x80000)
        .entry_point(0x80000)
        .name("Test Kernel")
        .data(test_data)
        .unwrap();

    let image_data = builder.build().unwrap();

    // Verify image structure
    assert_eq!(image_data.len(), IH_HEADER_SIZE + test_data.len());

    // Parse it back
    let parsed = ImageBuilder::from_image(&image_data).unwrap();

    assert_eq!(parsed.get_data(), test_data);
    assert_eq!(parsed.header().name, "Test Kernel");
    assert_eq!(parsed.header().type_, ImageType::Kernel);
    assert_eq!(parsed.header().arch, Arch::Arm64);
    assert_eq!(parsed.header().os, OsType::Linux);
    assert_eq!(parsed.header().load, 0x80000);
    assert_eq!(parsed.header().ep, 0x80000);
    assert_eq!(parsed.header().size, test_data.len() as u32);
    assert_eq!(parsed.header().dcrc, calculate_crc32(test_data));
}

/// Test different image types
#[test]
fn test_different_image_types() {
    let test_data = b"test data";

    let image_types = vec![
        ImageType::Standalone,
        ImageType::Kernel,
        ImageType::Ramdisk,
        ImageType::Multi,
        ImageType::Firmware,
        ImageType::Script,
        ImageType::Filesystem,
        ImageType::FlattenedDeviceTree,
    ];

    for img_type in image_types {
        let builder = ImageBuilder::new()
            .image_type(img_type)
            .name(&format!("Test {:?}", img_type))
            .data(test_data)
            .unwrap();

        let image_data = builder.build().unwrap();
        let parsed = ImageBuilder::from_image(&image_data).unwrap();

        assert_eq!(parsed.header().type_, img_type);
        assert_eq!(parsed.get_data(), test_data);
    }
}

/// Test different architectures
#[test]
fn test_different_architectures() {
    let test_data = b"test data";

    let archs = vec![
        Arch::Arm,
        Arch::Arm64,
        Arch::I386,
        Arch::X86_64,
        Arch::Mips,
        Arch::Powerpc,
        Arch::Riscv,
    ];

    for arch in archs {
        let builder = ImageBuilder::new()
            .image_type(ImageType::Kernel)
            .arch(arch)
            .name(&format!("Test {:?}", arch))
            .data(test_data)
            .unwrap();

        let image_data = builder.build().unwrap();
        let parsed = ImageBuilder::from_image(&image_data).unwrap();

        assert_eq!(parsed.header().arch, arch);
        assert_eq!(parsed.get_data(), test_data);
    }
}

/// Test file I/O operations
#[test]
fn test_file_operations() {
    let test_data = b"File test data";

    // Create temporary input file
    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(test_data).unwrap();
    input_file.flush().unwrap();

    // Create image from file
    let builder = ImageBuilder::new()
        .name("File Test")
        .data_from_file(input_file.path())
        .unwrap();

    let image_data = builder.build().unwrap();

    // Create temporary output file
    let output_file = NamedTempFile::new().unwrap();

    // Write image to file
    builder.build_to_file(output_file.path()).unwrap();

    // Read it back
    let file_data = fs::read(output_file.path()).unwrap();
    assert_eq!(file_data, image_data);

    // Parse from file
    let file_builder = ImageBuilder::from_image(&file_data).unwrap();
    assert_eq!(file_builder.get_data(), test_data);
    assert_eq!(file_builder.header().name, "File Test");
}

/// Test image verification
#[test]
fn test_image_verification() {
    let test_data = b"Verification test data";

    let builder = ImageBuilder::new()
        .name("Verification Test")
        .data(test_data)
        .unwrap();

    let image_data = builder.build().unwrap();

    // Verify we can parse it without errors
    let parsed = ImageBuilder::from_image(&image_data).unwrap();
    parsed.validate().unwrap();

    // Corrupt the data and verify it fails
    let mut corrupted_data = image_data.clone();
    let corrupt_pos = IH_HEADER_SIZE + 1; // Corrupt first byte of data
    if corrupt_pos < corrupted_data.len() {
        corrupted_data[corrupt_pos] ^= 0xFF;

        let result = ImageBuilder::from_image(&corrupted_data);
        assert!(result.is_err());
    }
}

/// Test total CRC functionality
#[test]
fn test_total_crc() {
    let test_data = b"Total CRC test data";

    let builder = ImageBuilder::new()
        .name("Total CRC Test")
        .data(test_data)
        .unwrap();

    let image_with_crc = builder.build_with_total_crc().unwrap();

    // Should be 4 bytes longer than regular image
    let regular_image = builder.build().unwrap();
    assert_eq!(image_with_crc.len(), regular_image.len() + 4);

    // Extract and verify total CRC
    let total_crc = u32::from_le_bytes([
        image_with_crc[image_with_crc.len() - 4],
        image_with_crc[image_with_crc.len() - 3],
        image_with_crc[image_with_crc.len() - 2],
        image_with_crc[image_with_crc.len() - 1],
    ]);

    let expected_crc = calculate_crc32(&image_with_crc[..image_with_crc.len() - 4]);
    assert_eq!(total_crc, expected_crc);
}

/// Test large data handling
#[test]
fn test_large_data() {
    // Create 1MB of test data
    let test_data = vec![0xAA; 1024 * 1024];

    let builder = ImageBuilder::new()
        .name("Large Data Test")
        .data(&test_data)
        .unwrap();

    let image_data = builder.build().unwrap();

    // Should be header + 1MB
    assert_eq!(image_data.len(), IH_HEADER_SIZE + test_data.len());

    // Verify parsing
    let parsed = ImageBuilder::from_image(&image_data).unwrap();
    assert_eq!(parsed.get_data(), test_data);
    assert_eq!(parsed.get_data().len(), 1024 * 1024);
}

/// Test empty data
#[test]
fn test_empty_data() {
    let builder = ImageBuilder::new()
        .name("Empty Data Test")
        .data(&[])
        .unwrap();

    let image_data = builder.build().unwrap();
    assert_eq!(image_data.len(), IH_HEADER_SIZE);

    let parsed = ImageBuilder::from_image(&image_data).unwrap();
    assert_eq!(parsed.get_data(), &[]);
    assert_eq!(parsed.header().size, 0);
}

/// Test name truncation
#[test]
fn test_name_truncation() {
    let long_name = "A".repeat(100); // Longer than IH_NMLEN (32)

    let builder = ImageBuilder::new()
        .name(&long_name)
        .data(b"test")
        .unwrap();

    assert_eq!(builder.header().name.len(), 32); // Should be truncated
    assert_eq!(builder.header().name, "A".repeat(32));

    let image_data = builder.build().unwrap();
    let parsed = ImageBuilder::from_image(&image_data).unwrap();
    assert_eq!(parsed.header().name, "A".repeat(32));
}

/// Test convenience constructors
#[test]
fn test_convenience_constructors() {
    let test_data = b"convenience test";

    // Test kernel constructor
    let kernel_builder = ImageBuilder::kernel()
        .data(test_data)
        .unwrap();

    assert_eq!(kernel_builder.header().type_, ImageType::Kernel);
    assert_eq!(kernel_builder.header().os, OsType::Linux);

    // Test ramdisk constructor
    let ramdisk_builder = ImageBuilder::ramdisk()
        .data(test_data)
        .unwrap();

    assert_eq!(ramdisk_builder.header().type_, ImageType::Ramdisk);
    assert_eq!(ramdisk_builder.header().os, OsType::Linux);

    // Test device tree constructor
    let dtb_builder = ImageBuilder::device_tree()
        .data(test_data)
        .unwrap();

    assert_eq!(dtb_builder.header().type_, ImageType::FlattenedDeviceTree);

    // Test script constructor
    let script_builder = ImageBuilder::script()
        .data(test_data)
        .unwrap();

    assert_eq!(script_builder.header().type_, ImageType::Script);
}

/// Test CRC32 consistency
#[test]
fn test_crc32_consistency() {
    let test_data = b"CRC32 consistency test";

    let builder1 = ImageBuilder::new()
        .name("Test 1")
        .data(test_data)
        .unwrap();

    let builder2 = ImageBuilder::new()
        .name("Test 2") // Different name
        .data(test_data) // Same data
        .unwrap();

    // Data CRC should be the same
    assert_eq!(builder1.header().dcrc, builder2.header().dcrc);

    // But the headers should be different due to different names
    assert_ne!(builder1.header(), builder2.header());
}

/// Test error conditions
#[test]
fn test_error_conditions() {
    // Test invalid magic number
    let mut invalid_header = vec![0u8; IH_HEADER_SIZE];
    // Wrong magic number
    invalid_header[0] = 0x12;
    invalid_header[1] = 0x34;
    invalid_header[2] = 0x56;
    invalid_header[3] = 0x78;

    let result = ImageBuilder::from_image(&invalid_header);
    assert!(result.is_err());

    // Test truncated image
    let mut header_data = vec![0u8; IH_HEADER_SIZE];
    // Set valid magic
    header_data[0] = (IH_MAGIC & 0xFF) as u8;
    header_data[1] = ((IH_MAGIC >> 8) & 0xFF) as u8;
    header_data[2] = ((IH_MAGIC >> 16) & 0xFF) as u8;
    header_data[3] = ((IH_MAGIC >> 24) & 0xFF) as u8;
    // Set size to 100 bytes
    header_data[16] = 100;
    header_data[17] = 0;
    header_data[18] = 0;
    header_data[19] = 0;

    // But only provide 50 bytes of data
    let truncated_image = [header_data, vec![0u8; 50]].concat();

    let result = ImageBuilder::from_image(&truncated_image);
    assert!(result.is_err());
}

/// Test round-trip compatibility
#[test]
fn test_round_trip_compatibility() {
    // Create an image with complex configuration
    let test_data = b"Round-trip compatibility test";

    let original = ImageBuilder::new()
        .image_type(ImageType::Kernel)
        .arch(Arch::Arm64)
        .os_type(OsType::Linux)
        .compression(Compression::None)
        .load_address(0x80000)
        .entry_point(0x80000)
        .name("Round-trip Test")
        .data(test_data)
        .unwrap();

    let image_data = original.build().unwrap();

    // Parse it multiple times to ensure consistency
    let parsed1 = ImageBuilder::from_image(&image_data).unwrap();
    let parsed2 = ImageBuilder::from_image(&image_data).unwrap();

    // Should be identical
    assert_eq!(parsed1.header(), parsed2.header());
    assert_eq!(parsed1.get_data(), parsed2.get_data());

    // Should match original
    assert_eq!(parsed1.header(), original.header());
    assert_eq!(parsed1.get_data(), original.get_data());

    // Rebuilding should produce identical data
    let rebuilt_image = parsed1.build().unwrap();
    assert_eq!(rebuilt_image, image_data);
}