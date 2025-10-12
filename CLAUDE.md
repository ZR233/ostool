# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ostool** is a Rust-based toolset for OS development that simplifies launching with Qemu and U-Boot. It consists of a workspace with two main crates:

- `ostool` - Main CLI tool for OS development workflows
- `uboot-shell` - Library for communicating with U-Boot

## Build and Development Commands

### Installation and Local Development

```bash
# Install from crates.io
cargo install ostool

# Local development build
cargo build --release

# Run tests
cargo test

# Install from local source
cargo install --path .
```

### Usage Commands

```bash
# Generate default configuration file
ostool defconfig

# Build project (compiles kernel according to .project.toml)
ostool build

# Run with Qemu
ostool run qemu
ostool run qemu -d  # debug mode

# Run with U-Boot (requires sudo capabilities for TFTP)
sudo setcap cap_net_bind_service=+eip $(which ostool)
ostool run uboot

# Run board tests
ostool board-test

# Run cargo tests
ostool cargo-test --elf <elf_file>
```

## Architecture

### Core Components

1. **Configuration System** (`src/config/`)
   - `mod.rs` - Main configuration structures
   - `compile/` - Build configuration (Cargo and Custom build systems)
   - `qemu.rs` - Qemu configuration

2. **Command Processing** (`src/cmd/`)
   - `defconfig.rs` - Generates default `.project.toml` configuration

3. **Project Management** (`src/project.rs`)
   - Handles `.project.toml` parsing and project workspace setup

4. **Execution Steps** (`src/step/`)
   - `compile.rs` - Kernel compilation (supports Cargo and custom shell commands)
   - `qemu.rs` - Qemu execution with configuration
   - `uboot/` - U-Boot integration and communication
   - `tftp.rs` - TFTP server for file transfer to U-Boot
   - `prepare_test.rs` - Test environment setup

5. **UI Components** (`src/ui.rs`)
   - Terminal UI using ratatui for progress indication

### Configuration File (.project.toml)

The tool is driven by a TOML configuration file with sections:
- `[compile]` - Target architecture and build system configuration
- `[compile.build.Cargo]` - Cargo-based builds
- `[compile.build.Custom]` - Custom shell command builds
- `[qemu]` - Qemu machine, CPU, graphics settings
- `[uboot]` - Serial port, baud rate, DTB file settings

### Build System Support

The tool supports two build approaches:
1. **Cargo builds** - Direct cargo commands for Rust OS projects
2. **Custom builds** - Shell commands for make-based or remote builds (e.g., cross-compilation)

### U-Boot Integration

The `uboot-shell` crate provides:
- Serial communication with U-Boot
- File transfer via YMODEM protocol
- Command execution and response parsing

## Key Dependencies

- `clap` - CLI argument parsing
- `toml`/`serde` - Configuration file handling
- `serialport` - Serial communication for U-Boot
- `ratatui`/`crossterm` - Terminal UI
- `tftpd` - TFTP server implementation
- `object` - ELF file parsing

## Testing

Tests are located in each crate's standard location. The main integration tests focus on:
- Configuration parsing
- Build step execution
- Qemu launching
- U-Boot communication

## Development Notes

- The project uses Rust 2024 edition
- Workspace structure with separate crates for modularity
- Extensive error handling with `anyhow`
- Colorized terminal output for better UX
- Support for both local and remote build workflows