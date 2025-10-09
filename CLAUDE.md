# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ostool` is a Rust-based OS development toolkit that simplifies building and testing operating systems with Qemu and U-Boot. The project consists of a main CLI tool (`ostool`) and a library (`uboot-shell`) for U-Boot communication.

## Build System

This is a Cargo workspace with two main members:
- `ostool/` - Main CLI application
- `uboot-shell/` - U-Boot communication library

### Common Commands

**Build the project:**
```bash
cargo build --workspace
```

**Build release version:**
```bash
cargo build --workspace --release
```

**Run tests:**
```bash
cargo test --workspace
```

**Check code formatting:**
```bash
cargo fmt --all -- --check
```

**Run linter:**
```bash
cargo clippy --workspace --all-features
```

**Install local version:**
```bash
cargo install --path ostool
```

The project uses Rust 2024 edition and requires `rust-objcopy` and `llvm-tools-preview` components.

## Architecture

### Core Components

**Main CLI (`ostool/src/main.rs`):**
- Entry point with clap-based command parsing
- Commands: `build`, `run qemu`, `run uboot`, `run tftp`, `test`, `defconfig`
- Unified `test` command replaces `cargo-test` and `board-test` with cargo test compatibility

**Project Management (`ostool/src/project.rs`):**
- `Project` struct manages workspace, configuration, and build metadata
- Unified configuration loading via `ConfigLoader` with support for includes
- Supports multiple architectures: aarch64, riscv64, x86_64

**Configuration (`ostool/src/config/`):**
- `ConfigLoader` provides unified configuration loading with inheritance
- `ProjectConfig` stores compile, Qemu, and U-Boot settings
- Supports multiple build systems: Cargo, Custom shell commands
- Configuration files use TOML format with `include` support for .board.toml files

**Step System (`ostool/src/step/`):**
- `Step` trait defines build pipeline operations
- Concrete steps: `Compile`, `Qemu`, `Uboot`, `Tftp`, `TestPrepare`
- Unified `TestPrepare` handles both cargo test and manual testing
- Each step can be executed sequentially in the pipeline

**U-Boot Shell Library (`uboot-shell/`):**
- Handles serial communication with U-Boot
- Implements YMODEM protocol for file transfers
- Provides shell-like interaction interface

### Configuration File Format

**Main `.project.toml` structure:**
```toml
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["make ARCH=aarch64"]
kernel = "path/to/kernel.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"
graphic = false

[uboot]
serial = "COM3"
baud_rate = 115200

# Include board-specific configuration
include = [".board.toml"]
```

**Board-specific `.board.toml` (optional):**
```toml
[uboot]
serial = "/dev/ttyUSB0"
dtb_file = "board.dtb"
```

### Key Design Patterns

- **Step Pattern**: Operations implement `Step` trait for pipeline execution
- **Configuration-driven**: Behavior controlled by TOML configuration files
- **Multi-architecture**: Supports aarch64, riscv64, x86_64 through abstraction
- **Workspace-aware**: Integrates with Cargo workspaces for metadata

## Testing

The project includes unit tests and integration tests. Use `cargo test` to run them. The CI workflow runs tests on x86_64 Linux and includes Qemu installation for integration testing.