# ostool

[![CI](https://github.com/ZR233/ostool/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ZR233/ostool/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ostool.svg)](https://crates.io/crates/ostool)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

---

## 🌐 Language | 语言

**[English](README.en.md)** | **简体中文** (当前 | Current)

---

## 📖 项目简介

**ostool** 是一个专为操作系统开发而设计的 Rust 工具集，旨在为 OS 开发者提供便捷的构建、配置和启动环境。它特别适合嵌入式系统开发，支持通过 Qemu 虚拟机和 U-Boot 引导程序进行系统测试和调试。

### ✨ 核心特性

- 🔧 **一体化工具链** - 集构建、配置、运行于一体的完整解决方案
- 🖥️ **现代化 TUI** - 基于终端的用户界面，提供直观的配置编辑体验
- ⚙️ **智能配置管理** - JSON Schema 驱动的配置验证和编辑
- 🚀 **多种启动方式** - 支持 Qemu 虚拟机和 U-Boot 硬件启动
- 🌐 **跨平台支持** - Linux、Windows 等多平台兼容
- 📦 **模块化架构** - 可扩展的组件设计，便于定制和集成

## 🏗️ 项目架构

ostool 采用 Rust 工作空间架构，包含以下核心模块：

### 核心组件

| 组件 | 功能描述 | 主要用途 |
|------|----------|----------|
| **ostool** | 主要工具包 | CLI 工具，构建和运行系统 |
| **jkconfig** | 配置编辑器 | TUI 配置编辑界面 |
| **fitimage** | FIT 镜像构建 | U-Boot 兼容的启动镜像生成 |
| **uboot-shell** | U-Boot 通信 | 串口通信和命令执行 |

### 技术栈

- **Rust** - 核心开发语言，提供内存安全和性能
- **Cursive** - 现代化 TUI 框架
- **JSON Schema** - 配置验证和类型安全
- **Tokio** - 异步运行时
- **Serialport** - 串口通信
- **Clap** - 命令行参数解析

## 🚀 快速开始

### 安装

```bash
# 从 crates.io 安装
cargo install ostool

# 或从源码构建
git clone https://github.com/ZR233/ostool.git
cd ostool
cargo install --path .
```

### 基本使用

#### 1. 查看帮助

```bash
# 查看主帮助
ostool --help

# 查看构建帮助
ostool build --help

# 查看运行帮助
ostool run --help

# 查看配置帮助
ostool menuconfig --help
```

#### 2. 配置管理

```bash
# 使用 TUI 编辑构建配置
ostool menuconfig

# 配置 QEMU 运行参数
ostool menuconfig qemu

# 配置 U-Boot 运行参数
ostool menuconfig uboot
```

#### 3. 构建系统

```bash
# 构建项目（使用默认配置文件 .build.toml）
ostool build

# 指定配置文件构建
ostool build --config custom-build.toml

# 在指定工作目录中构建
ostool --workdir /path/to/project build
```

#### 4. 运行系统

```bash
# 使用 Qemu 运行
ostool run qemu

# 使用 Qemu 运行并启用调试
ostool run qemu --debug

# 使用 Qemu 运行并转储 DTB 文件
ostool run qemu --dtb-dump

# 指定 Qemu 配置文件运行
ostool run qemu --qemu-config my-qemu.toml

# 使用 U-Boot 运行
ostool run uboot

# 指定 U-Boot 配置文件运行
ostool run uboot --uboot-config my-uboot.toml
```

> 交互退出：在串口终端（如 `ostool run uboot`）中，按下 `Ctrl+A` 后再按 `x`，工具会检测到该序列并优雅退出，不会将按键发送到目标设备。
> 更多键盘快捷键映射可参考源码 `ostool/src/sterm/mod.rs`。

## ⚙️ 配置文件

ostool 使用多个独立的 TOML 配置文件，每个文件负责不同的功能模块：

### 构建配置 (.build.toml)

构建配置文件定义了如何编译你的操作系统内核。

#### Cargo 构建系统示例

```toml
[system]
# 使用 Cargo 构建系统
system = "Cargo"

[system.Cargo]
# 目标三元组
target = "aarch64-unknown-none"

# 包名称
package = "my-os-kernel"

# 启用的特性
features = ["page-alloc-4g"]

# 日志级别
log = "Info"

# 环境变量
env = { "RUSTFLAGS" = "-C link-arg=-Tlinker.ld" }

# 额外的 cargo 参数
args = ["--release"]

# 构建前执行的命令
pre_build_cmds = ["make prepare"]

# 构建后执行的命令
post_build_cmds = ["make post-process"]

# 是否输出为二进制文件
to_bin = true
```

#### 自定义构建系统示例

```toml
[system]
# 使用自定义构建系统
system = "Custom"

[system.Custom]
# 构建命令
build_cmd = "make ARCH=aarch64 A=examples/helloworld"

# 生成的 ELF 文件路径
elf_path = "examples/helloworld/helloworld_aarch64-qemu-virt.elf"

# 是否输出为二进制文件
to_bin = true
```

### QEMU 配置 (.qemu.toml)

QEMU 配置文件定义了虚拟机的启动参数。

```toml
# QEMU 启动参数
args = ["-machine", "virt", "-cpu", "cortex-a57", "-nographic"]

# 启用 UEFI 引导
uefi = false

# 输出为二进制文件
to_bin = true

# 成功运行的正则表达式（用于自动检测）
success_regex = ["Hello from my OS", "Kernel booted successfully"]

# 失败运行的正则表达式（用于自动检测）
fail_regex = ["panic", "error", "failed"]
```

### U-Boot 配置 (.uboot.toml)

U-Boot 配置文件定义了硬件启动参数。

```toml
# 串口设备
serial = "/dev/ttyUSB0"

# 波特率
baud_rate = "115200"

# 设备树文件（可选）
dtb_file = "tools/device_tree.dtb"

# 内核加载地址（可选）
kernel_load_addr = "0x80080000"

# 网络启动配置（可选）
[net]
interface = "eth0"
board_ip = "192.168.1.100"

# 板子重置命令（可选）
board_reset_cmd = "reset"

# 板子断电命令（可选）
board_power_off_cmd = "poweroff"

# 成功启动的正则表达式
success_regex = ["Starting kernel", "Boot successful"]

# 失败启动的正则表达式
fail_regex = ["Boot failed", "Error loading kernel"]
```

### 环境变量支持

配置文件支持环境变量替换，使用 `${env:VAR_NAME:-default}` 格式：

```toml
# .uboot.toml 示例
serial = "${env:SERIAL_DEVICE:-/dev/ttyUSB0}"
baud_rate = "${env:BAUD_RATE:-115200}"
```

## 🛠️ 子项目详解

### JKConfig - 智能配置编辑器

**JKConfig** 是一个基于 JSON Schema 的 TUI 配置编辑器，提供以下功能：

#### 主要特性

- 🎯 **智能界面生成** - 自动从 JSON Schema 生成编辑界面
- 🔒 **类型安全** - 支持复杂数据类型和验证规则
- 📝 **多格式支持** - TOML、JSON 格式读写
- 💾 **自动备份** - 保存时自动创建备份文件
- ⌨️ **快捷键支持** - Vim 风格的键盘操作

#### 使用方法

```bash
# 安装
cargo install jkconfig

# 编辑配置
jkconfig -c config.toml -s config-schema.json

# 自动检测 schema
jkconfig -c config.toml
```

#### 键盘快捷键

```text
导航：
↑/↓ 或 j/k     - 上下移动
Enter          - 编辑项目
Esc            - 返回上级

操作：
S              - 保存并退出
Q              - 不保存退出
C              - 清除当前值
M              - 切换菜单状态
Tab            - 切换选项
~              - 调试控制台
```

### FitImage - FIT 镜像构建工具

**FitImage** 是用于创建 U-Boot 兼容的 FIT (Flattened Image Tree) 镜像的专业工具：

#### 主要特性

- 🏗️ **标准 FIT 格式** - 完全符合 U-Boot FIT 规范
- 📦 **多组件支持** - 内核、设备树、ramdisk 等
- 🗜️ **压缩功能** - gzip 压缩减少镜像大小
- 🔐 **校验支持** - CRC32、SHA1 等多种校验算法
- 🎯 **架构兼容** - ARM、ARM64 等多种架构

#### 使用示例

```rust
use fitimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

// 创建 FIT 镜像配置
let config = FitImageConfig::new("My FIT Image")
    .with_kernel(
        ComponentConfig::new("kernel", kernel_data)
            .with_type("kernel")
            .with_arch("arm64")
            .with_load_address(0x80080000)
    )
    .with_fdt(
        ComponentConfig::new("fdt", fdt_data)
            .with_type("flat_dt")
            .with_arch("arm64")
    );

// 构建镜像
let mut builder = FitImageBuilder::new();
let fit_data = builder.build(config)?;

// 保存文件
std::fs::write("image.fit", fit_data)?;
```

## 🎯 使用场景

### 1. 本地开发工作流

```bash
# 1. 初始化项目
git clone <your-os-project>
cd <your-os-project>

# 2. 使用 menuconfig 配置构建参数
ostool menuconfig

# 3. 配置 QEMU 运行参数
ostool menuconfig qemu

# 4. 构建项目
ostool build

# 5. 使用 Qemu 运行
ostool run qemu

# 6. 启用调试模式运行
ostool run qemu --debug
```

### 2. 远程构建和硬件测试

```bash
# 1. 使用 menuconfig 配置自定义构建
ostool menuconfig

# 2. 配置 U-Boot 运行参数
ostool menuconfig uboot

# 3. 执行构建
ostool build

# 4. 通过 U-Boot 启动到硬件
ostool run uboot

# 5. 指定自定义 U-Boot 配置
ostool run uboot --uboot-config custom-uboot.toml
```

### 3. 嵌入式系统开发

- 🎯 **多架构支持** - ARM64、RISC-V64 等多种架构
- 🔧 **设备树管理** - 自动处理 DTB 文件和设备树配置
- 📡 **网络启动** - 支持 TFTP 网络启动和远程加载
- 🖥️ **串口调试** - 实时串口监控和调试信息输出
- 🔐 **FIT 镜像** - 创建 U-Boot 兼容的 FIT 启动镜像
- ⚡ **自动化构建** - 支持构建前后脚本和自定义命令

### 4. 高级调试场景

```bash
# 启用详细日志
RUST_LOG=debug ostool run qemu

# 转储 DTB 文件用于调试
ostool run qemu --dtb-dump

# 在指定工作目录中操作
ostool --workdir /path/to/kernel build
ostool --workdir /path/to/kernel run qemu
```

## 🔧 高级配置

### U-Boot 网络启动设置

```bash
# TFTP 需要 root 权限绑定 69 端口
sudo setcap cap_net_bind_service=+eip $(which ostool)
```

### 调试配置

```toml
[qemu]
args = "-s -S"  # 启用 GDB 调试

[uboot]
# 启用详细日志
log_level = "debug"
```

## 🐛 故障排除

### 常见问题

**Q: U-Boot 启动失败？**
A: 检查以下几点：
- 串口设备路径是否正确（`/dev/ttyUSB0` 或其他）
- 串口权限是否足够（可能需要 `sudo usermod -a -G dialout $USER`）
- 波特率设置是否与硬件匹配
- 设备树文件路径是否正确

**Q: Qemu 无法启动？**
A: 检查以下几点：
- 构建生成的内核文件是否存在
- QEMU 配置中的架构参数是否正确
- 是否安装了对应架构的 QEMU（如 `qemu-system-aarch64`）

**Q: 构建失败？**
A: 检查以下几点：
- 构建配置文件格式是否正确
- 自定义构建命令是否能在终端中执行
- 目标架构的交叉编译工具链是否安装

**Q: 配置文件格式错误？**
A: 检查以下几点：
- TOML 语法是否正确（使用在线 TOML 验证器）
- 配置文件是否使用了正确的字段名
- 数组和字符串格式是否符合规范

**Q: menuconfig 无法启动？**
A: 检查以下几点：
- 终端是否支持 TUI 界面
- 是否安装了必要的依赖（如 ncurses）
- 配置文件权限是否正确

### 调试技巧

```bash
# 启用详细日志
RUST_LOG=debug ostool run qemu

# 查看完整的命令行帮助
ostool --help
ostool build --help
ostool run --help
ostool run qemu --help
ostool run uboot --help
ostool menuconfig --help

# 检查配置文件是否被正确加载
RUST_LOG=debug ostool build 2>&1 | grep -i config

# 在指定工作目录中调试
ostool --workdir /path/to/project build
```

### 权限问题解决

```bash
# 将用户添加到 dialout 组以访问串口设备
sudo usermod -a -G dialout $USER
# 重新登录或重启使权限生效

# 或者临时使用 sudo 运行
sudo ostool run uboot
```

## 🤝 贡献指南

我们欢迎社区贡献！请遵循以下步骤：

1. **Fork** 本仓库
2. **创建** 特性分支 (`git checkout -b feature/amazing-feature`)
3. **提交** 更改 (`git commit -m 'Add some amazing feature'`)
4. **推送** 到分支 (`git push origin feature/amazing-feature`)
5. **创建** Pull Request

### 开发环境设置

```bash
git clone https://github.com/ZR233/ostool.git
cd ostool
cargo build
cargo test
```

## 📄 许可证

本项目采用双重许可证：

- [MIT License](LICENSE)
- [Apache License 2.0](LICENSE)

## 🔗 相关链接

- [GitHub 仓库](https://github.com/ZR233/ostool)
- [Crates.io 包](https://crates.io/crates/ostool)
- [问题反馈](https://github.com/ZR233/ostool/issues)
- [文档 Wiki](https://github.com/ZR233/ostool/wiki)

## 🙏 致谢

感谢所有为 ostool 项目做出贡献的开发者和用户！
