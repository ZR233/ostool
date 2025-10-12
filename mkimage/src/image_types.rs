//! Image type definitions and constants

use std::fmt;
use std::str::FromStr;
use crate::error::{MkImageError, Result};

/// U-Boot image types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageType {
    /// Invalid/unknown image type
    Invalid = 0,
    /// Standalone program
    Standalone = 1,
    /// OS kernel image
    Kernel = 2,
    /// RAM disk image
    Ramdisk = 3,
    /// Multi-file image
    Multi = 4,
    /// Firmware image
    Firmware = 5,
    /// Script file
    Script = 6,
    /// Filesystem image
    Filesystem = 7,
    /// Flat device tree
    FlattenedDeviceTree = 8,
    /// Kernel image with arguments
    KernelWithArgs = 9,
    /// RAM disk image with arguments
    RamdiskWithArgs = 10,
    /// Firmware with arguments
    FirmwareWithArgs = 11,
    /// Script with arguments
    ScriptWithArgs = 12,
    /// Filesystem with arguments
    FilesystemWithArgs = 13,
}

/// Architecture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    Invalid = 0,
    Alpha = 1,
    Arm = 2,
    I386 = 3,
    Ia64 = 4,
    Mips = 5,
    Mips64 = 6,
    Powerpc = 7,
    S390 = 8,
    Sh = 9,
    Sparc = 10,
    Sparc64 = 11,
    M68k = 12,
    Nios = 13,
    Microblaze = 14,
    Nios2 = 15,
    Blackfin = 16,
    Avr32 = 17,
    St200 = 18,
    Sandbox = 19,
    Nds32 = 20,
    Openrisc = 21,
    Arm64 = 22,
    Arc = 23,
    X86_64 = 24,
    Xtensa = 25,
    Riscv = 26,
}

/// Compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compression {
    None = 0,
    Gzip = 1,
    Bzip2 = 2,
    Lzma = 3,
    Lzo = 4,
    Lz4 = 5,
}

/// Operating system types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsType {
    Invalid = 0,
    Openbsd = 1,
    Netbsd = 2,
    Freebsd = 3,
    Bsd4_4 = 4,
    Linux = 5,
    SvR4 = 6,
    Esix = 7,
    Solaris = 8,
    Irix = 9,
    Sco = 10,
    Dell = 11,
    Ncr = 12,
    Lynxos = 13,
    Vxworks = 14,
    Psos = 15,
    Qnx = 16,
    Uboot = 17,
    Rtems = 18,
    Artos = 19,
    Unity = 20,
    Integrity = 21,
    Ose = 22,
    Plan9 = 23,
    Openrtos = 24,
}

impl FromStr for ImageType {
    type Err = MkImageError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "standalone" => Ok(Self::Standalone),
            "kernel" => Ok(Self::Kernel),
            "ramdisk" => Ok(Self::Ramdisk),
            "multi" => Ok(Self::Multi),
            "firmware" => Ok(Self::Firmware),
            "script" => Ok(Self::Script),
            "filesystem" => Ok(Self::Filesystem),
            "flatdt" | "fdt" => Ok(Self::FlattenedDeviceTree),
            "kernel_noload" => Ok(Self::KernelWithArgs),
            "ramdisk_noload" => Ok(Self::RamdiskWithArgs),
            "firmware_noload" => Ok(Self::FirmwareWithArgs),
            "script_noload" => Ok(Self::ScriptWithArgs),
            "filesystem_noload" => Ok(Self::FilesystemWithArgs),
            _ => Err(MkImageError::unsupported_image_type(s)),
        }
    }
}

impl fmt::Display for ImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Invalid => "invalid",
            Self::Standalone => "standalone",
            Self::Kernel => "kernel",
            Self::Ramdisk => "ramdisk",
            Self::Multi => "multi",
            Self::Firmware => "firmware",
            Self::Script => "script",
            Self::Filesystem => "filesystem",
            Self::FlattenedDeviceTree => "flatdt",
            Self::KernelWithArgs => "kernel_noload",
            Self::RamdiskWithArgs => "ramdisk_noload",
            Self::FirmwareWithArgs => "firmware_noload",
            Self::ScriptWithArgs => "script_noload",
            Self::FilesystemWithArgs => "filesystem_noload",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Arch {
    type Err = MkImageError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "alpha" => Ok(Self::Alpha),
            "arm" => Ok(Self::Arm),
            "i386" | "x86" => Ok(Self::I386),
            "ia64" => Ok(Self::Ia64),
            "mips" => Ok(Self::Mips),
            "mips64" => Ok(Self::Mips64),
            "ppc" | "powerpc" => Ok(Self::Powerpc),
            "s390" => Ok(Self::S390),
            "sh" => Ok(Self::Sh),
            "sparc" => Ok(Self::Sparc),
            "sparc64" => Ok(Self::Sparc64),
            "m68k" => Ok(Self::M68k),
            "nios" => Ok(Self::Nios),
            "microblaze" => Ok(Self::Microblaze),
            "nios2" => Ok(Self::Nios2),
            "blackfin" => Ok(Self::Blackfin),
            "avr32" => Ok(Self::Avr32),
            "st200" => Ok(Self::St200),
            "sandbox" => Ok(Self::Sandbox),
            "nds32" => Ok(Self::Nds32),
            "openrisc" => Ok(Self::Openrisc),
            "arm64" | "aarch64" => Ok(Self::Arm64),
            "arc" => Ok(Self::Arc),
            "x86_64" | "amd64" => Ok(Self::X86_64),
            "xtensa" => Ok(Self::Xtensa),
            "riscv" => Ok(Self::Riscv),
            _ => Err(MkImageError::unsupported_arch(s)),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Invalid => "invalid",
            Self::Alpha => "alpha",
            Self::Arm => "arm",
            Self::I386 => "i386",
            Self::Ia64 => "ia64",
            Self::Mips => "mips",
            Self::Mips64 => "mips64",
            Self::Powerpc => "powerpc",
            Self::S390 => "s390",
            Self::Sh => "sh",
            Self::Sparc => "sparc",
            Self::Sparc64 => "sparc64",
            Self::M68k => "m68k",
            Self::Nios => "nios",
            Self::Microblaze => "microblaze",
            Self::Nios2 => "nios2",
            Self::Blackfin => "blackfin",
            Self::Avr32 => "avr32",
            Self::St200 => "st200",
            Self::Sandbox => "sandbox",
            Self::Nds32 => "nds32",
            Self::Openrisc => "openrisc",
            Self::Arm64 => "arm64",
            Self::Arc => "arc",
            Self::X86_64 => "x86_64",
            Self::Xtensa => "xtensa",
            Self::Riscv => "riscv",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Compression {
    type Err = MkImageError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "gzip" | "gz" => Ok(Self::Gzip),
            "bzip2" | "bz2" => Ok(Self::Bzip2),
            "lzma" => Ok(Self::Lzma),
            "lzo" => Ok(Self::Lzo),
            "lz4" => Ok(Self::Lz4),
            _ => Err(MkImageError::unsupported_compression(s)),
        }
    }
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::None => "none",
            Self::Gzip => "gzip",
            Self::Bzip2 => "bzip2",
            Self::Lzma => "lzma",
            Self::Lzo => "lzo",
            Self::Lz4 => "lz4",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for OsType {
    type Err = MkImageError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "openbsd" => Ok(Self::Openbsd),
            "netbsd" => Ok(Self::Netbsd),
            "freebsd" => Ok(Self::Freebsd),
            "bsd4.4" | "bsd44" => Ok(Self::Bsd4_4),
            "linux" => Ok(Self::Linux),
            "svr4" => Ok(Self::SvR4),
            "esix" => Ok(Self::Esix),
            "solaris" => Ok(Self::Solaris),
            "irix" => Ok(Self::Irix),
            "sco" => Ok(Self::Sco),
            "dell" => Ok(Self::Dell),
            "ncr" => Ok(Self::Ncr),
            "lynxos" => Ok(Self::Lynxos),
            "vxworks" => Ok(Self::Vxworks),
            "psos" => Ok(Self::Psos),
            "qnx" => Ok(Self::Qnx),
            "uboot" => Ok(Self::Uboot),
            "rtems" => Ok(Self::Rtems),
            "artos" => Ok(Self::Artos),
            "unity" => Ok(Self::Unity),
            "integrity" => Ok(Self::Integrity),
            "ose" => Ok(Self::Ose),
            "plan9" => Ok(Self::Plan9),
            "openrtos" => Ok(Self::Openrtos),
            _ => Err(MkImageError::unsupported_image_type(s)),
        }
    }
}

impl fmt::Display for OsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Invalid => "invalid",
            Self::Openbsd => "openbsd",
            Self::Netbsd => "netbsd",
            Self::Freebsd => "freebsd",
            Self::Bsd4_4 => "bsd4.4",
            Self::Linux => "linux",
            Self::SvR4 => "svr4",
            Self::Esix => "esix",
            Self::Solaris => "solaris",
            Self::Irix => "irix",
            Self::Sco => "sco",
            Self::Dell => "dell",
            Self::Ncr => "ncr",
            Self::Lynxos => "lynxos",
            Self::Vxworks => "vxworks",
            Self::Psos => "psos",
            Self::Qnx => "qnx",
            Self::Uboot => "uboot",
            Self::Rtems => "rtems",
            Self::Artos => "artos",
            Self::Unity => "unity",
            Self::Integrity => "integrity",
            Self::Ose => "ose",
            Self::Plan9 => "plan9",
            Self::Openrtos => "openrtos",
        };
        write!(f, "{}", name)
    }
}

// Constants for default values
impl Default for ImageType {
    fn default() -> Self {
        Self::Kernel
    }
}

impl Default for Arch {
    fn default() -> Self {
        Self::I386
    }
}

impl Default for Compression {
    fn default() -> Self {
        Self::None
    }
}

impl Default for OsType {
    fn default() -> Self {
        Self::Linux
    }
}