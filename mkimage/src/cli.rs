//! Command line interface for mkimage

use crate::{ImageBuilder, VERSION};
use crate::error::{MkImageError, Result};
use crate::image_types::{ImageType, Arch, Compression, OsType};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Command line arguments for mkimage
#[derive(Parser, Debug)]
#[command(name = "mkimage")]
#[command(version = VERSION)]
#[command(about = "A Rust implementation of U-Boot's mkimage tool", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode - only output errors
    #[arg(short, long)]
    pub quiet: bool,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a U-Boot image
    Create(CreateArgs),
    /// List information about an existing image
    List(ListArgs),
    /// Verify an image's CRC32
    Verify(VerifyArgs),
}

/// Arguments for creating an image
#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// Image type (kernel, ramdisk, etc.)
    #[arg(short = 'T', long, value_enum, default_value = "kernel")]
    pub image_type: ImageTypeArg,

    /// Architecture (arm, arm64, x86_64, etc.)
    #[arg(short = 'A', long, value_enum, default_value = "i386")]
    pub arch: ArchArg,

    /// Operating system type
    #[arg(short = 'O', long, value_enum, default_value = "linux")]
    pub os: OsTypeArg,

    /// Compression type (none, gzip, bzip2, etc.)
    #[arg(short = 'C', long, value_enum, default_value = "none")]
    pub compression: CompressionArg,

    /// Load address (hexadecimal)
    #[arg(short = 'a', long, value_parser = parse_hex_u32)]
    pub load_address: Option<u32>,

    /// Entry point address (hexadecimal)
    #[arg(short = 'e', long, value_parser = parse_hex_u32)]
    pub entry_point: Option<u32>,

    /// Image name
    #[arg(short = 'n', long, default_value = "Image")]
    pub name: String,

    /// Input data file
    #[arg(short = 'd', long)]
    pub data_file: Option<PathBuf>,

    /// Output image file
    #[arg(short, long, default_value = "image.bin")]
    pub output: PathBuf,

    /// Add CRC32 of entire image at the end
    #[arg(long)]
    pub add_total_crc: bool,

    /// Print image information after creation
    #[arg(long)]
    pub print_info: bool,
}

/// Arguments for listing image information
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Image file to examine
    pub image_file: PathBuf,

    /// Print in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Arguments for verifying an image
#[derive(Parser, Debug)]
pub struct VerifyArgs {
    /// Image file to verify
    pub image_file: PathBuf,

    /// Verify total CRC (if present)
    #[arg(long)]
    pub total_crc: bool,
}

/// Custom argument types for better CLI experience

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum ImageTypeArg {
    Invalid,
    Standalone,
    Kernel,
    Ramdisk,
    Multi,
    Firmware,
    Script,
    Filesystem,
    FlattenedDeviceTree,
    KernelWithArgs,
    RamdiskWithArgs,
    FirmwareWithArgs,
    ScriptWithArgs,
    FilesystemWithArgs,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum ArchArg {
    Invalid,
    Alpha,
    Arm,
    I386,
    Ia64,
    Mips,
    Mips64,
    Powerpc,
    S390,
    Sh,
    Sparc,
    Sparc64,
    M68k,
    Nios,
    Microblaze,
    Nios2,
    Blackfin,
    Avr32,
    St200,
    Sandbox,
    Nds32,
    Openrisc,
    Arm64,
    Arc,
    X86_64,
    Xtensa,
    Riscv,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum CompressionArg {
    None,
    Gzip,
    Bzip2,
    Lzma,
    Lzo,
    Lz4,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OsTypeArg {
    Invalid,
    Openbsd,
    Netbsd,
    Freebsd,
    Bsd44,
    Linux,
    SvR4,
    Esix,
    Solaris,
    Irix,
    Sco,
    Dell,
    Ncr,
    Lynxos,
    Vxworks,
    Psos,
    Qnx,
    Uboot,
    Rtems,
    Artos,
    Unity,
    Integrity,
    Ose,
    Plan9,
    Openrtos,
}

// Conversion implementations
impl From<ImageTypeArg> for ImageType {
    fn from(arg: ImageTypeArg) -> Self {
        match arg {
            ImageTypeArg::Invalid => Self::Invalid,
            ImageTypeArg::Standalone => Self::Standalone,
            ImageTypeArg::Kernel => Self::Kernel,
            ImageTypeArg::Ramdisk => Self::Ramdisk,
            ImageTypeArg::Multi => Self::Multi,
            ImageTypeArg::Firmware => Self::Firmware,
            ImageTypeArg::Script => Self::Script,
            ImageTypeArg::Filesystem => Self::Filesystem,
            ImageTypeArg::FlattenedDeviceTree => Self::FlattenedDeviceTree,
            ImageTypeArg::KernelWithArgs => Self::KernelWithArgs,
            ImageTypeArg::RamdiskWithArgs => Self::RamdiskWithArgs,
            ImageTypeArg::FirmwareWithArgs => Self::FirmwareWithArgs,
            ImageTypeArg::ScriptWithArgs => Self::ScriptWithArgs,
            ImageTypeArg::FilesystemWithArgs => Self::FilesystemWithArgs,
        }
    }
}

impl From<ArchArg> for Arch {
    fn from(arg: ArchArg) -> Self {
        match arg {
            ArchArg::Invalid => Self::Invalid,
            ArchArg::Alpha => Self::Alpha,
            ArchArg::Arm => Self::Arm,
            ArchArg::I386 => Self::I386,
            ArchArg::Ia64 => Self::Ia64,
            ArchArg::Mips => Self::Mips,
            ArchArg::Mips64 => Self::Mips64,
            ArchArg::Powerpc => Self::Powerpc,
            ArchArg::S390 => Self::S390,
            ArchArg::Sh => Self::Sh,
            ArchArg::Sparc => Self::Sparc,
            ArchArg::Sparc64 => Self::Sparc64,
            ArchArg::M68k => Self::M68k,
            ArchArg::Nios => Self::Nios,
            ArchArg::Microblaze => Self::Microblaze,
            ArchArg::Nios2 => Self::Nios2,
            ArchArg::Blackfin => Self::Blackfin,
            ArchArg::Avr32 => Self::Avr32,
            ArchArg::St200 => Self::St200,
            ArchArg::Sandbox => Self::Sandbox,
            ArchArg::Nds32 => Self::Nds32,
            ArchArg::Openrisc => Self::Openrisc,
            ArchArg::Arm64 => Self::Arm64,
            ArchArg::Arc => Self::Arc,
            ArchArg::X86_64 => Self::X86_64,
            ArchArg::Xtensa => Self::Xtensa,
            ArchArg::Riscv => Self::Riscv,
        }
    }
}

impl From<CompressionArg> for Compression {
    fn from(arg: CompressionArg) -> Self {
        match arg {
            CompressionArg::None => Self::None,
            CompressionArg::Gzip => Self::Gzip,
            CompressionArg::Bzip2 => Self::Bzip2,
            CompressionArg::Lzma => Self::Lzma,
            CompressionArg::Lzo => Self::Lzo,
            CompressionArg::Lz4 => Self::Lz4,
        }
    }
}

impl From<OsTypeArg> for OsType {
    fn from(arg: OsTypeArg) -> Self {
        match arg {
            OsTypeArg::Invalid => Self::Invalid,
            OsTypeArg::Openbsd => Self::Openbsd,
            OsTypeArg::Netbsd => Self::Netbsd,
            OsTypeArg::Freebsd => Self::Freebsd,
            OsTypeArg::Bsd44 => Self::Bsd4_4,
            OsTypeArg::Linux => Self::Linux,
            OsTypeArg::SvR4 => Self::SvR4,
            OsTypeArg::Esix => Self::Esix,
            OsTypeArg::Solaris => Self::Solaris,
            OsTypeArg::Irix => Self::Irix,
            OsTypeArg::Sco => Self::Sco,
            OsTypeArg::Dell => Self::Dell,
            OsTypeArg::Ncr => Self::Ncr,
            OsTypeArg::Lynxos => Self::Lynxos,
            OsTypeArg::Vxworks => Self::Vxworks,
            OsTypeArg::Psos => Self::Psos,
            OsTypeArg::Qnx => Self::Qnx,
            OsTypeArg::Uboot => Self::Uboot,
            OsTypeArg::Rtems => Self::Rtems,
            OsTypeArg::Artos => Self::Artos,
            OsTypeArg::Unity => Self::Unity,
            OsTypeArg::Integrity => Self::Integrity,
            OsTypeArg::Ose => Self::Ose,
            OsTypeArg::Plan9 => Self::Plan9,
            OsTypeArg::Openrtos => Self::Openrtos,
        }
    }
}

/// Parse hexadecimal string to u32
fn parse_hex_u32(s: &str) -> std::result::Result<u32, std::num::ParseIntError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16)
    } else {
        s.parse::<u32>()
    }
}

/// Main CLI handler
pub fn run_cli(args: Args) -> Result<()> {
    let verbose = args.verbose && !args.quiet;
    let quiet = args.quiet;

    match args.command.unwrap_or_else(|| Commands::Create(CreateArgs {
        image_type: ImageTypeArg::Kernel,
        arch: ArchArg::I386,
        os: OsTypeArg::Linux,
        compression: CompressionArg::None,
        load_address: None,
        entry_point: None,
        name: "Image".to_string(),
        data_file: None,
        output: PathBuf::from("image.bin"),
        add_total_crc: false,
        print_info: false,
    })) {
        Commands::Create(create_args) => {
            handle_create(create_args, verbose, quiet)
        }
        Commands::List(list_args) => {
            handle_list(list_args, verbose, quiet)
        }
        Commands::Verify(verify_args) => {
            handle_verify(verify_args, verbose, quiet)
        }
    }
}

fn handle_create(args: CreateArgs, verbose: bool, quiet: bool) -> Result<()> {
    if verbose {
        eprintln!("Creating U-Boot image...");
    }

    // Start with basic builder
    let mut builder = ImageBuilder::new()
        .image_type(args.image_type.clone().into())
        .arch(args.arch.into())
        .os_type(args.os.into())
        .compression(args.compression.into())
        .name(&args.name);

    // Set load address for certain image types if not provided
    let load_address = args.load_address.unwrap_or_else(|| {
        match args.image_type {
            ImageTypeArg::Kernel => 0x80000,     // Common kernel load address
            ImageTypeArg::Ramdisk => 0x4000000,  // Common ramdisk load address
            _ => 0,
        }
    });

    if load_address != 0 || matches!(args.image_type, ImageTypeArg::Kernel | ImageTypeArg::Firmware | ImageTypeArg::Standalone) {
        builder = builder.load_address(load_address);
    }

    // Set entry point if provided or same as load address for certain types
    if let Some(ep) = args.entry_point {
        builder = builder.entry_point(ep);
    } else if matches!(args.image_type, ImageTypeArg::Kernel | ImageTypeArg::Firmware | ImageTypeArg::Standalone) {
        builder = builder.entry_point(load_address);
    }

    // Load data
    let builder = if let Some(data_file) = args.data_file {
        if verbose {
            eprintln!("Loading data from: {}", data_file.display());
        }
        builder.data_from_file(data_file)?
    } else {
        if !quiet {
            eprintln!("Warning: No data file specified, creating image with no payload");
        }
        builder.data(&[])?
    };

    // Validate before building
    builder.validate()?;

    // Build image
    let image_data = if args.add_total_crc {
        if verbose {
            eprintln!("Adding total CRC32 to image");
        }
        builder.build_with_total_crc()?
    } else {
        builder.build()?
    };

    // Write to file
    if verbose {
        eprintln!("Writing image to: {}", args.output.display());
    }
    std::fs::write(&args.output, image_data)?;

    if !quiet {
        eprintln!("Image created successfully: {}", args.output.display());
        eprintln!("Image size: {} bytes", std::fs::metadata(&args.output)?.len());
    }

    // Print info if requested
    if args.print_info && !quiet {
        println!();
        builder.print_info();
    }

    Ok(())
}

fn handle_list(args: ListArgs, verbose: bool, _quiet: bool) -> Result<()> {
    if verbose {
        eprintln!("Reading image: {}", args.image_file.display());
    }

    let image_data = std::fs::read(&args.image_file)?;
    let builder = ImageBuilder::from_image(&image_data)?;

    if args.json {
        // JSON output would require serde dependency, for now use custom format
        println!("{{
  \"name\": \"{}\",
  \"type\": \"{}\",
  \"arch\": \"{}\",
  \"os\": \"{}\",
  \"compression\": \"{}\",
  \"load_address\": \"0x{:08x}\",
  \"entry_point\": \"0x{:08x}\",
  \"size\": {},
  \"data_crc\": \"0x{:08x}\",
  \"timestamp\": \"{}\"
}}",
            builder.header().name,
            builder.header().type_,
            builder.header().arch,
            builder.header().os,
            builder.header().comp,
            builder.header().load,
            builder.header().ep,
            builder.header().size,
            builder.header().dcrc,
            builder.header().timestamp().format("%Y-%m-%d %H:%M:%S UTC")
        );
    } else {
        builder.print_info();
    }

    Ok(())
}

fn handle_verify(args: VerifyArgs, verbose: bool, quiet: bool) -> Result<()> {
    if verbose {
        eprintln!("Verifying image: {}", args.image_file.display());
    }

    let image_data = std::fs::read(&args.image_file)?;

    // Verify data CRC
    let builder = ImageBuilder::from_image(&image_data)?;

    if !quiet {
        eprintln!("Image data CRC32: 0x{:08x} - OK", builder.header().dcrc);
    }

    // Verify total CRC if requested and present
    if args.total_crc && image_data.len() > 4 {
        let total_size = builder.header().total_size() as usize;
        if image_data.len() >= total_size + 4 {
            let expected_total_crc = u32::from_le_bytes([
                image_data[total_size],
                image_data[total_size + 1],
                image_data[total_size + 2],
                image_data[total_size + 3],
            ]);

            let calculated_total_crc = crate::crc::calculate_crc32(&image_data[..total_size]);

            if expected_total_crc == calculated_total_crc {
                if !quiet {
                    eprintln!("Total CRC32: 0x{:08x} - OK", expected_total_crc);
                }
            } else {
                eprintln!("Total CRC32 mismatch: expected 0x{:08x}, calculated 0x{:08x}",
                         expected_total_crc, calculated_total_crc);
                return Err(MkImageError::crc_mismatch(expected_total_crc, calculated_total_crc));
            }
        }
    }

    if !quiet {
        eprintln!("Image verification successful");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_hex_u32() {
        assert_eq!(parse_hex_u32("0x1000").unwrap(), 4096);
        assert_eq!(parse_hex_u32("0X1000").unwrap(), 4096);
        assert_eq!(parse_hex_u32("1000").unwrap(), 1000);
    }

    #[test]
    fn test_args_parsing() {
        let args = Args::try_parse_from(&["mkimage", "create", "-T", "kernel", "-A", "arm64", "-n", "Test"]).unwrap();

        if let Commands::Create(create_args) = args.command.unwrap() {
            assert_eq!(create_args.name, "Test");
            assert_eq!(create_args.image_type, ImageTypeArg::Kernel);
            assert_eq!(create_args.arch, ArchArg::Arm64);
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_conversions() {
        assert_eq!(ImageType::from(ImageTypeArg::Kernel), ImageType::Kernel);
        assert_eq!(Arch::from(ArchArg::Arm64), Arch::Arm64);
        assert_eq!(Compression::from(CompressionArg::Gzip), Compression::Gzip);
        assert_eq!(OsType::from(OsTypeArg::Linux), OsType::Linux);
    }
}