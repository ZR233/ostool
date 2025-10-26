//! Image type definitions and constants

use std::fmt;

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
    /// Flattened Image Tree (FIT)
    FlattenedImageTree = 14,
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
    Arc = 23,
    Arm64 = 24,
    X86_64 = 25,
    Xtensa = 26,
    Riscv = 27,
}

/// Compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    Rtems = 17,
    Artos = 19,
    Unity = 20,
    Integrity = 21,
    Ose = 22,
    Plan9 = 23,
    Openrtos = 24,
    Uboot = 25,
}

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

// Display trait implementations
impl fmt::Display for ImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid => write!(f, "Invalid"),
            Self::Standalone => write!(f, "Standalone"),
            Self::Kernel => write!(f, "Kernel"),
            Self::Ramdisk => write!(f, "Ramdisk"),
            Self::Multi => write!(f, "Multi"),
            Self::Firmware => write!(f, "Firmware"),
            Self::Script => write!(f, "Script"),
            Self::Filesystem => write!(f, "Filesystem"),
            Self::FlattenedDeviceTree => write!(f, "Flattened Device Tree"),
            Self::KernelWithArgs => write!(f, "Kernel with Args"),
            Self::RamdiskWithArgs => write!(f, "Ramdisk with Args"),
            Self::FirmwareWithArgs => write!(f, "Firmware with Args"),
            Self::ScriptWithArgs => write!(f, "Script with Args"),
            Self::FilesystemWithArgs => write!(f, "Filesystem with Args"),
            Self::FlattenedImageTree => write!(f, "Flattened Image Tree"),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid => write!(f, "Invalid"),
            Self::Alpha => write!(f, "Alpha"),
            Self::Arm => write!(f, "ARM"),
            Self::I386 => write!(f, "i386"),
            Self::Ia64 => write!(f, "IA64"),
            Self::Mips => write!(f, "MIPS"),
            Self::Mips64 => write!(f, "MIPS64"),
            Self::Powerpc => write!(f, "PowerPC"),
            Self::S390 => write!(f, "S390"),
            Self::Sh => write!(f, "SuperH"),
            Self::Sparc => write!(f, "SPARC"),
            Self::Sparc64 => write!(f, "SPARC64"),
            Self::M68k => write!(f, "68k"),
            Self::Nios => write!(f, "NIOS"),
            Self::Microblaze => write!(f, "MicroBlaze"),
            Self::Nios2 => write!(f, "NIOS II"),
            Self::Blackfin => write!(f, "Blackfin"),
            Self::Avr32 => write!(f, "AVR32"),
            Self::St200 => write!(f, "ST200"),
            Self::Sandbox => write!(f, "Sandbox"),
            Self::Nds32 => write!(f, "NDS32"),
            Self::Openrisc => write!(f, "OpenRISC"),
            Self::Arc => write!(f, "ARC"),
            Self::Arm64 => write!(f, "AArch64"),
            Self::X86_64 => write!(f, "x86_64"),
            Self::Xtensa => write!(f, "Xtensa"),
            Self::Riscv => write!(f, "RISC-V"),
        }
    }
}

impl fmt::Display for OsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid => write!(f, "Invalid"),
            Self::Openbsd => write!(f, "OpenBSD"),
            Self::Netbsd => write!(f, "NetBSD"),
            Self::Freebsd => write!(f, "FreeBSD"),
            Self::Bsd4_4 => write!(f, "BSD 4.4"),
            Self::Linux => write!(f, "Linux"),
            Self::SvR4 => write!(f, "SVR4"),
            Self::Esix => write!(f, "Esix"),
            Self::Solaris => write!(f, "Solaris"),
            Self::Irix => write!(f, "IRIX"),
            Self::Sco => write!(f, "SCO"),
            Self::Dell => write!(f, "Dell"),
            Self::Ncr => write!(f, "NCR"),
            Self::Lynxos => write!(f, "LynxOS"),
            Self::Vxworks => write!(f, "VxWorks"),
            Self::Psos => write!(f, "pSOS"),
            Self::Qnx => write!(f, "QNX"),
            Self::Rtems => write!(f, "RTEMS"),
            Self::Artos => write!(f, "ARTOS"),
            Self::Unity => write!(f, "Unity"),
            Self::Integrity => write!(f, "INTEGRITY"),
            Self::Ose => write!(f, "OSE"),
            Self::Plan9 => write!(f, "Plan 9"),
            Self::Openrtos => write!(f, "OpenRTOS"),
            Self::Uboot => write!(f, "U-Boot"),
        }
    }
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Gzip => write!(f, "Gzip"),
            Self::Bzip2 => write!(f, "Bzip2"),
            Self::Lzma => write!(f, "LZMA"),
            Self::Lzo => write!(f, "LZO"),
            Self::Lz4 => write!(f, "LZ4"),
        }
    }
}