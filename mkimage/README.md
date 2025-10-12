# mkimage

A Rust implementation of U-Boot's mkimage tool for creating bootable images.

## Overview

This crate provides functionality to create various types of U-Boot compatible images including legacy images, FIT images, and multi-file images. It aims to be a drop-in replacement for the original C implementation with a modern, safe Rust interface.

## Features

- **Legacy Image Support**: Full support for U-Boot legacy image format
- **Multiple Image Types**: Kernel, ramdisk, firmware, script, device tree, etc.
- **Architecture Support**: ARM, ARM64, x86, x86_64, RISC-V, and more
- **Compression Types**: None, gzip, bzip2, lzma, lzo, lz4
- **CLI Tool**: Command-line interface compatible with original mkimage
- **Library API**: Programmatic interface for embedding in other tools
- **CRC32 Verification**: Automatic checksum calculation and verification
- **Safe Implementation**: Memory-safe Rust implementation

## Installation

### As a Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
mkimage = "0.1.0"
```

### As a CLI Tool

```bash
cargo install mkimage
```

## Usage

### Command Line Interface

The CLI follows the same interface as the original mkimage:

```bash
# Create a kernel image
mkimage create -T kernel -A arm64 -n "Linux Kernel" -d zImage -o uImage

# Create a ramdisk image
mkimage create -T ramdisk -A arm64 -n "Initramfs" -d initramfs.cpio.gz -o uramdisk

# List image information
mkimage list uImage

# Verify an image
mkimage verify uImage
```

### Library API

```rust
use mkimage::{ImageBuilder, ImageType, Arch, OsType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a kernel image
    let kernel_data = std::fs::read("zImage")?;

    let builder = ImageBuilder::kernel()
        .arch(Arch::Arm64)
        .load_address(0x80000)
        .entry_point(0x80000)
        .name("Linux Kernel")
        .data(&kernel_data)?;

    let image = builder.build()?;
    std::fs::write("uImage", image)?;

    Ok(())
}
```

## Supported Image Types

- **Kernel**: OS kernel images
- **Ramdisk**: RAM disk images
- **Firmware**: Firmware/bootloader images
- **Script**: U-Boot script images
- **Filesystem**: Filesystem images
- **Flattened Device Tree**: Device tree blobs
- **Multi**: Multi-file images
- **Standalone**: Standalone programs

## Supported Architectures

- ARM, ARM64 (AArch64)
- x86, x86_64
- MIPS, MIPS64
- PowerPC
- RISC-V
- SPARC, SPARC64
- And many more...

## Testing

Run the test suite:

```bash
cargo test
```

Run integration tests:

```bash
cargo test --test integration_tests
cargo test --test cli_tests
```

## Compatibility

This implementation aims to be compatible with the original U-Boot mkimage tool. Images created with this tool should be bootable by U-Boot and vice versa.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## References

- [U-Boot mkimage documentation](https://www.denx.de/wiki/U-Boot/mkimage)
- [U-Boot image format specification](https://www.denx.de/wiki/U-Boot/Documentation/ImageFormat)