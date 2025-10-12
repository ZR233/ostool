//! U-Boot image header structures and serialization

use crate::IH_MAGIC;
use crate::error::{MkImageError, Result};
use crate::image_types::{ImageType, Arch, Compression, OsType};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Utc};
use std::io::{Read, Write};

/// Maximum length of image name
pub const IH_NMLEN: usize = 32;

/// U-Boot legacy image header size (from U-Boot source)
pub const IH_HEADER_SIZE: usize = 60;

/// U-Boot legacy image header structure
///
/// This structure represents the standard U-Boot legacy image header format.
/// It follows the exact layout used by U-Boot for compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageHeader {
    /// Magic number (must be IH_MAGIC)
    pub magic: u32,
    /// Timestamp when the image was created
    pub time: u32,
    /// Size of the image data
    pub size: u32,
    /// Load address of the image
    pub load: u32,
    /// Entry point address
    pub ep: u32,
    /// CRC32 checksum of the image data
    pub dcrc: u32,
    /// Operating system type
    pub os: OsType,
    /// Architecture type
    pub arch: Arch,
    /// Image type
    pub type_: ImageType,
    /// Compression type
    pub comp: Compression,
    /// Image name (null-terminated)
    pub name: String,
}

impl Default for ImageHeader {
    fn default() -> Self {
        Self {
            magic: IH_MAGIC,
            time: Utc::now().timestamp() as u32,
            size: 0,
            load: 0,
            ep: 0,
            dcrc: 0,
            os: OsType::default(),
            arch: Arch::default(),
            type_: ImageType::default(),
            comp: Compression::default(),
            name: String::new(),
        }
    }
}

impl ImageHeader {
    /// Create a new image header with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new image header with the given name
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut header = Self::default();
        header.set_name(name);
        header
    }

    /// Set the image name
    ///
    /// The name will be truncated if it exceeds the maximum length.
    pub fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        if name.len() > IH_NMLEN {
            self.name = name.chars().take(IH_NMLEN).collect();
        } else {
            self.name = name;
        }
    }

    /// Get the timestamp as a DateTime
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.time as i64, 0).unwrap_or_else(Utc::now)
    }

    /// Set the timestamp from a DateTime
    pub fn set_timestamp(&mut self, timestamp: DateTime<Utc>) {
        self.time = timestamp.timestamp() as u32;
    }

    /// Validate the header
    ///
    /// Checks if the magic number is correct and other fields are valid.
    pub fn validate(&self) -> Result<()> {
        if self.magic != IH_MAGIC {
            return Err(MkImageError::invalid_magic(IH_MAGIC, self.magic));
        }

        if self.name.len() > IH_NMLEN {
            return Err(MkImageError::NameTooLong {
                len: self.name.len(),
                max: IH_NMLEN,
            });
        }

        Ok(())
    }

    /// Serialize the header to bytes
    ///
    /// Returns a vector containing the serialized header data.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(IH_HEADER_SIZE);
        self.write_to(&mut buffer)?;
        Ok(buffer)
    }

    /// Write the header to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write magic number
        writer.write_u32::<LittleEndian>(self.magic)?;

        // Write timestamp
        writer.write_u32::<LittleEndian>(self.time)?;

        // Write size
        writer.write_u32::<LittleEndian>(self.size)?;

        // Write load address
        writer.write_u32::<LittleEndian>(self.load)?;

        // Write entry point
        writer.write_u32::<LittleEndian>(self.ep)?;

        // Write data CRC
        writer.write_u32::<LittleEndian>(self.dcrc)?;

        // Write OS type
        writer.write_u8(self.os as u8)?;

        // Write architecture
        writer.write_u8(self.arch as u8)?;

        // Write image type
        writer.write_u8(self.type_ as u8)?;

        // Write compression type
        writer.write_u8(self.comp as u8)?;

        // Write name (null-terminated, padded to IH_NMLEN)
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(IH_NMLEN);
        writer.write_all(&name_bytes[..name_len])?;

        // Pad with zeros and ensure null termination
        let padding_len = IH_NMLEN - name_len;
        if padding_len > 0 {
            writer.write_all(&vec![0u8; padding_len])?;
        }

        Ok(())
    }

    /// Deserialize the header from bytes
    ///
    /// Creates a new ImageHeader from the provided byte slice.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < IH_HEADER_SIZE {
            return Err(MkImageError::invalid_image_data(format!(
                "Header data too short: {} bytes (expected at least {})",
                data.len(),
                IH_HEADER_SIZE
            )));
        }

        let mut cursor = std::io::Cursor::new(data);

        let magic = cursor.read_u32::<LittleEndian>()?;
        let time = cursor.read_u32::<LittleEndian>()?;
        let size = cursor.read_u32::<LittleEndian>()?;
        let load = cursor.read_u32::<LittleEndian>()?;
        let ep = cursor.read_u32::<LittleEndian>()?;
        let dcrc = cursor.read_u32::<LittleEndian>()?;

        let os = OsType::try_from(cursor.read_u8()?)
            .map_err(|_| MkImageError::invalid_image_data("Invalid OS type"))?;
        let arch = Arch::try_from(cursor.read_u8()?)
            .map_err(|_| MkImageError::invalid_image_data("Invalid architecture"))?;
        let type_ = ImageType::try_from(cursor.read_u8()?)
            .map_err(|_| MkImageError::invalid_image_data("Invalid image type"))?;
        let comp = Compression::try_from(cursor.read_u8()?)
            .map_err(|_| MkImageError::invalid_image_data("Invalid compression type"))?;

        // Read name (up to IH_NMLEN bytes, null-terminated)
        let mut name_bytes = vec![0u8; IH_NMLEN];
        cursor.read_exact(&mut name_bytes)?;

        // Find null terminator
        let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(IH_NMLEN);
        let name = String::from_utf8_lossy(&name_bytes[..name_len]).into_owned();

        let header = Self {
            magic,
            time,
            size,
            load,
            ep,
            dcrc,
            os,
            arch,
            type_,
            comp,
            name,
        };

        header.validate()?;
        Ok(header)
    }

    /// Read the header from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut header_data = vec![0u8; IH_HEADER_SIZE];
        reader.read_exact(&mut header_data)?;
        Self::from_bytes(&header_data)
    }

    /// Calculate the total image size (header + data)
    pub fn total_size(&self) -> u32 {
        IH_HEADER_SIZE as u32 + self.size
    }

    /// Get a summary of the header information
    pub fn summary(&self) -> String {
        format!(
            "Image: {} ({})\n\
             Type: {} OS: {} Arch: {} Compression: {}\n\
             Load Address: 0x{:08x} Entry Point: 0x{:08x}\n\
             Size: {} bytes\n\
             Timestamp: {}",
            self.name,
            if self.magic == IH_MAGIC { "Legacy" } else { "Unknown" },
            self.type_,
            self.os,
            self.arch,
            self.comp,
            self.load,
            self.ep,
            self.size,
            self.timestamp().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

// Implement TryFrom for enum types
impl TryFrom<u8> for OsType {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Invalid),
            1 => Ok(Self::Openbsd),
            2 => Ok(Self::Netbsd),
            3 => Ok(Self::Freebsd),
            4 => Ok(Self::Bsd4_4),
            5 => Ok(Self::Linux),
            6 => Ok(Self::SvR4),
            7 => Ok(Self::Esix),
            8 => Ok(Self::Solaris),
            9 => Ok(Self::Irix),
            10 => Ok(Self::Sco),
            11 => Ok(Self::Dell),
            12 => Ok(Self::Ncr),
            13 => Ok(Self::Lynxos),
            14 => Ok(Self::Vxworks),
            15 => Ok(Self::Psos),
            16 => Ok(Self::Qnx),
            17 => Ok(Self::Uboot),
            18 => Ok(Self::Rtems),
            19 => Ok(Self::Artos),
            20 => Ok(Self::Unity),
            21 => Ok(Self::Integrity),
            22 => Ok(Self::Ose),
            23 => Ok(Self::Plan9),
            24 => Ok(Self::Openrtos),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Arch {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Invalid),
            1 => Ok(Self::Alpha),
            2 => Ok(Self::Arm),
            3 => Ok(Self::I386),
            4 => Ok(Self::Ia64),
            5 => Ok(Self::Mips),
            6 => Ok(Self::Mips64),
            7 => Ok(Self::Powerpc),
            8 => Ok(Self::S390),
            9 => Ok(Self::Sh),
            10 => Ok(Self::Sparc),
            11 => Ok(Self::Sparc64),
            12 => Ok(Self::M68k),
            13 => Ok(Self::Nios),
            14 => Ok(Self::Microblaze),
            15 => Ok(Self::Nios2),
            16 => Ok(Self::Blackfin),
            17 => Ok(Self::Avr32),
            18 => Ok(Self::St200),
            19 => Ok(Self::Sandbox),
            20 => Ok(Self::Nds32),
            21 => Ok(Self::Openrisc),
            22 => Ok(Self::Arm64),
            23 => Ok(Self::Arc),
            24 => Ok(Self::X86_64),
            25 => Ok(Self::Xtensa),
            26 => Ok(Self::Riscv),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for ImageType {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Invalid),
            1 => Ok(Self::Standalone),
            2 => Ok(Self::Kernel),
            3 => Ok(Self::Ramdisk),
            4 => Ok(Self::Multi),
            5 => Ok(Self::Firmware),
            6 => Ok(Self::Script),
            7 => Ok(Self::Filesystem),
            8 => Ok(Self::FlattenedDeviceTree),
            9 => Ok(Self::KernelWithArgs),
            10 => Ok(Self::RamdiskWithArgs),
            11 => Ok(Self::FirmwareWithArgs),
            12 => Ok(Self::ScriptWithArgs),
            13 => Ok(Self::FilesystemWithArgs),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Compression {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Gzip),
            2 => Ok(Self::Bzip2),
            3 => Ok(Self::Lzma),
            4 => Ok(Self::Lzo),
            5 => Ok(Self::Lz4),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_header_default() {
        let header = ImageHeader::default();
        assert_eq!(header.magic, IH_MAGIC);
        assert_eq!(header.size, 0);
        assert_eq!(header.load, 0);
        assert_eq!(header.ep, 0);
        assert_eq!(header.dcrc, 0);
        assert_eq!(header.name, "");
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_header_with_name() {
        let header = ImageHeader::with_name("Test Kernel");
        assert_eq!(header.name, "Test Kernel");
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_header_name_too_long() {
        let long_name = "A".repeat(IH_NMLEN + 10);
        let header = ImageHeader::with_name(long_name.clone());
        assert_eq!(header.name.len(), IH_NMLEN);
        assert_eq!(header.name, "A".repeat(IH_NMLEN));
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_header_serialization() {
        let mut header = ImageHeader::with_name("Test");
        header.size = 1024;
        header.load = 0x8000;
        header.ep = 0x8000;
        header.arch = Arch::Arm64;
        header.type_ = ImageType::Kernel;
        header.os = OsType::Linux;

        let bytes = header.to_bytes().unwrap();
        eprintln!("Header size: {}, Expected: {}", bytes.len(), IH_HEADER_SIZE);
        assert_eq!(bytes.len(), IH_HEADER_SIZE);

        let parsed = ImageHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.magic, parsed.magic);
        assert_eq!(header.time, parsed.time);
        assert_eq!(header.size, parsed.size);
        assert_eq!(header.load, parsed.load);
        assert_eq!(header.ep, parsed.ep);
        assert_eq!(header.dcrc, parsed.dcrc);
        assert_eq!(header.os, parsed.os);
        assert_eq!(header.arch, parsed.arch);
        assert_eq!(header.type_, parsed.type_);
        assert_eq!(header.comp, parsed.comp);
        assert_eq!(header.name, parsed.name);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut header = ImageHeader::default();
        header.magic = 0x12345678;
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_header_timestamp() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let mut header = ImageHeader::default();
        header.set_timestamp(timestamp);

        assert_eq!(header.timestamp(), timestamp);
    }

    #[test]
    fn test_header_summary() {
        let header = ImageHeader::with_name("Test Kernel");
        let summary = header.summary();
        assert!(summary.contains("Test Kernel"));
        assert!(summary.contains("Legacy"));
    }

    #[test]
    fn test_header_total_size() {
        let mut header = ImageHeader::default();
        header.size = 1024;
        assert_eq!(header.total_size(), IH_HEADER_SIZE as u32 + 1024);
    }

    #[test]
    fn test_enum_conversions() {
        // Test ImageType conversion
        let img_type: u8 = ImageType::Kernel as u8;
        let parsed = ImageType::try_from(img_type).unwrap();
        assert_eq!(parsed, ImageType::Kernel);

        // Test Arch conversion
        let arch: u8 = Arch::Arm64 as u8;
        let parsed = Arch::try_from(arch).unwrap();
        assert_eq!(parsed, Arch::Arm64);

        // Test OsType conversion
        let os: u8 = OsType::Linux as u8;
        let parsed = OsType::try_from(os).unwrap();
        assert_eq!(parsed, OsType::Linux);

        // Test Compression conversion
        let comp: u8 = Compression::Gzip as u8;
        let parsed = Compression::try_from(comp).unwrap();
        assert_eq!(parsed, Compression::Gzip);
    }
}