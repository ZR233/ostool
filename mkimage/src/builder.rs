//! Image builder for creating U-Boot compatible images

use crate::{ImageHeader, IH_HEADER_SIZE};
use crate::crc::calculate_crc32;
use crate::error::{MkImageError, Result};
use crate::image_types::{ImageType, Arch, Compression, OsType};
use std::io::Write;

/// Maximum data size for a legacy image (1GB)
pub const MAX_DATA_SIZE: u64 = 1024 * 1024 * 1024;

/// Builder for creating U-Boot images
///
/// This struct provides a fluent interface for configuring and creating
/// U-Boot compatible images.
#[derive(Debug, Clone)]
pub struct ImageBuilder {
    header: ImageHeader,
    data: Vec<u8>,
}

impl ImageBuilder {
    /// Create a new image builder with default values
    pub fn new() -> Self {
        Self {
            header: ImageHeader::default(),
            data: Vec::new(),
        }
    }

    /// Create a new image builder with the given name
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut builder = Self::new();
        builder.header.set_name(name);
        builder
    }

    /// Set the image type
    pub fn image_type(mut self, type_: ImageType) -> Self {
        self.header.type_ = type_;
        self
    }

    /// Set the target architecture
    pub fn arch(mut self, arch: Arch) -> Self {
        self.header.arch = arch;
        self
    }

    /// Set the operating system type
    pub fn os_type(mut self, os: OsType) -> Self {
        self.header.os = os;
        self
    }

    /// Set the compression type
    pub fn compression(mut self, comp: Compression) -> Self {
        self.header.comp = comp;
        self
    }

    /// Set the load address
    pub fn load_address(mut self, addr: u32) -> Self {
        self.header.load = addr;
        self
    }

    /// Set the entry point address
    pub fn entry_point(mut self, addr: u32) -> Self {
        self.header.ep = addr;
        self
    }

    /// Set the image name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.header.set_name(name);
        self
    }

    /// Set the image data (method that can return error)
    pub fn set_data(&mut self, data: &[u8]) -> Result<()> {
        if data.len() as u64 > MAX_DATA_SIZE {
            return Err(MkImageError::DataTooLarge {
                size: data.len() as u64,
                max: MAX_DATA_SIZE,
            });
        }

        self.data = data.to_vec();
        self.header.size = data.len() as u32;
        self.header.dcrc = calculate_crc32(data);
        Ok(())
    }

    /// Set the image data
    pub fn data(mut self, data: &[u8]) -> Result<Self> {
        self.set_data(data)?;
        Ok(self)
    }

    /// Set data from a file
    pub fn data_from_file<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self> {
        let data = std::fs::read(path)?;
        self.set_data(&data)?;
        Ok(self)
    }

    /// Get the current image header
    pub fn header(&self) -> &ImageHeader {
        &self.header
    }

    /// Get the current image data
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Set data from a reader
    pub fn data_from_reader<R: std::io::Read>(mut self, reader: &mut R) -> Result<Self> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        self.set_data(&buffer)?;
        Ok(self)
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<()> {
        self.header.validate()?;

        if self.header.size != self.get_data().len() as u32 {
            return Err(MkImageError::invalid_image_data(format!(
                "Header size ({}) doesn't match actual data size ({})",
                self.header.size,
                self.get_data().len()
            )));
        }

        if self.header.dcrc != calculate_crc32(self.get_data()) {
            return Err(MkImageError::crc_mismatch(
                self.header.dcrc,
                calculate_crc32(self.get_data()),
            ));
        }

        // Validate load and entry point addresses for certain image types
        match self.header.type_ {
            ImageType::Kernel | ImageType::Standalone | ImageType::Firmware => {
                if self.header.load == 0 {
                    return Err(MkImageError::InvalidLoadAddress {
                        address: self.header.load as u64,
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Build the complete image
    ///
    /// Returns the image data including header and payload.
    pub fn build(&self) -> Result<Vec<u8>> {
        self.validate()?;

        let mut image = Vec::with_capacity(IH_HEADER_SIZE + self.data.len());

        // Write header
        self.header.write_to(&mut image)?;

        // Write data
        image.write_all(&self.data)?;

        Ok(image)
    }

    /// Build the image and write it to a file
    pub fn build_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let image_data = self.build()?;
        std::fs::write(path, image_data)?;
        Ok(())
    }

    /// Build the image and write it to a writer
    pub fn build_to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.validate()?;

        // Write header
        self.header.write_to(writer)?;

        // Write data
        writer.write_all(&self.data)?;

        Ok(())
    }

    /// Build the image with CRC32 of the entire image (header + data)
    ///
    /// This is useful for creating images that can be verified as a whole.
    pub fn build_with_total_crc(&self) -> Result<Vec<u8>> {
        let image_data = self.build()?;
        let total_crc = calculate_crc32(&image_data);

        let mut result = image_data;
        result.extend_from_slice(&total_crc.to_le_bytes());

        Ok(result)
    }

    /// Create a builder from an existing header
    pub fn from_header(header: ImageHeader) -> Self {
        Self {
            header,
            data: Vec::new(),
        }
    }

    /// Create a builder from an existing image
    pub fn from_image(image_data: &[u8]) -> Result<Self> {
        if image_data.len() < IH_HEADER_SIZE {
            return Err(MkImageError::invalid_image_data(format!(
                "Image data too short: {} bytes (expected at least {})",
                image_data.len(),
                IH_HEADER_SIZE
            )));
        }

        let header = ImageHeader::from_bytes(&image_data[..IH_HEADER_SIZE])?;

        let data_start = IH_HEADER_SIZE;
        let data_end = data_start + header.size as usize;

        if image_data.len() < data_end {
            return Err(MkImageError::invalid_image_data(format!(
                "Image data incomplete: expected {} bytes, got {}",
                data_end,
                image_data.len()
            )));
        }

        let data = image_data[data_start..data_end].to_vec();

        let mut builder = Self::from_header(header);
        builder.data = data;

        Ok(builder)
    }

    /// Print image information
    pub fn print_info(&self) {
        println!("{}", self.header.summary());
        if !self.data.is_empty() {
            println!("Data CRC32: 0x{:08x}", self.header.dcrc);
        }
    }
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for creating common image types
impl ImageBuilder {
    /// Create a kernel image
    pub fn kernel() -> Self {
        Self::new()
            .image_type(ImageType::Kernel)
            .os_type(OsType::Linux)
    }

    /// Create a RAM disk image
    pub fn ramdisk() -> Self {
        Self::new()
            .image_type(ImageType::Ramdisk)
            .os_type(OsType::Linux)
    }

    /// Create a device tree image
    pub fn device_tree() -> Self {
        Self::new()
            .image_type(ImageType::FlattenedDeviceTree)
            .os_type(OsType::Linux)
    }

    /// Create a script image
    pub fn script() -> Self {
        Self::new()
            .image_type(ImageType::Script)
            .os_type(OsType::Linux)
    }

    /// Create a multi-file image
    pub fn multi_file() -> Self {
        Self::new()
            .image_type(ImageType::Multi)
            .os_type(OsType::Linux)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_types::Arch;

    #[test]
    fn test_builder_default() {
        let builder = ImageBuilder::new();
        // Default builder should be valid for non-kernel types
        assert_eq!(builder.header().type_, ImageType::default());
        // We'll use a script type which doesn't need load address
        let script_builder = ImageBuilder::script();
        assert!(script_builder.validate().is_ok());
    }

    #[test]
    fn test_builder_with_data() {
        let data = b"Hello, World!";
        let builder = ImageBuilder::script()  // Use script type to avoid load address requirement
            .data(data)
            .unwrap()
            .name("Test");

        assert!(builder.validate().is_ok());
        assert_eq!(builder.get_data(), data);
        assert_eq!(builder.header().size, data.len() as u32);
    }

    #[test]
    fn test_builder_kernel() {
        let data = b"kernel_data";
        let builder = ImageBuilder::kernel()
            .arch(Arch::Arm64)
            .load_address(0x80000)
            .entry_point(0x80000)
            .data(data)
            .unwrap()
            .name("Test Kernel");

        assert!(builder.validate().is_ok());
        assert_eq!(builder.header().type_, ImageType::Kernel);
        assert_eq!(builder.header().arch, Arch::Arm64);
    }

    #[test]
    fn test_builder_build() {
        let data = b"test data";
        let image = ImageBuilder::script()  // Use script type
            .data(data)
            .unwrap()
            .name("Test")
            .build()
            .unwrap();

        assert_eq!(image.len(), IH_HEADER_SIZE + data.len());

        // Verify we can parse it back
        let parsed = ImageBuilder::from_image(&image).unwrap();
        assert_eq!(parsed.get_data(), data);
        assert_eq!(parsed.header().name, "Test");
    }

    #[test]
    fn test_builder_invalid_load_address() {
        let builder = ImageBuilder::kernel()
            .data(b"test")
            .unwrap();

        // Kernel should have non-zero load address
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_builder_data_too_large() {
        let data = vec![0u8; (MAX_DATA_SIZE + 1) as usize];
        let mut builder = ImageBuilder::new();
        let result = builder.set_data(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_from_image() {
        let data = b"original data";
        let original = ImageBuilder::script()  // Use script type to avoid load address requirement
            .arch(Arch::Arm64)
            .name("Original")
            .data(data)
            .unwrap();

        let image_data = original.build().unwrap();
        let parsed = ImageBuilder::from_image(&image_data).unwrap();

        assert_eq!(parsed.get_data(), data);
        assert_eq!(parsed.header().name, "Original");
        assert_eq!(parsed.header().arch, Arch::Arm64);
        assert_eq!(parsed.header().type_, ImageType::Script);
    }

    #[test]
    fn test_builder_build_with_total_crc() {
        let data = b"test data";
        let builder = ImageBuilder::script()  // Use script type
            .data(data)
            .unwrap()
            .name("Test");

        let image_with_crc = builder.build_with_total_crc().unwrap();
        assert_eq!(image_with_crc.len(), IH_HEADER_SIZE + data.len() + 4);

        // Extract total CRC
        let total_crc = u32::from_le_bytes([
            image_with_crc[image_with_crc.len() - 4],
            image_with_crc[image_with_crc.len() - 3],
            image_with_crc[image_with_crc.len() - 2],
            image_with_crc[image_with_crc.len() - 1],
        ]);

        let expected_crc = calculate_crc32(&image_with_crc[..image_with_crc.len() - 4]);
        assert_eq!(total_crc, expected_crc);
    }

    #[test]
    fn test_convenience_constructors() {
        assert_eq!(ImageBuilder::kernel().header().type_, ImageType::Kernel);
        assert_eq!(ImageBuilder::ramdisk().header().type_, ImageType::Ramdisk);
        assert_eq!(ImageBuilder::device_tree().header().type_, ImageType::FlattenedDeviceTree);
        assert_eq!(ImageBuilder::script().header().type_, ImageType::Script);
        assert_eq!(ImageBuilder::multi_file().header().type_, ImageType::Multi);
    }
}