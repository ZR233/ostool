# mkimage - FIT Image Library

一个用于创建U-Boot兼容FIT (Flattened Image Tree) 镜像的Rust库。

## 特性

- ✅ 完全用Rust实现
- ✅ 支持kernel、FDT、ramdisk组件
- ✅ gzip压缩支持
- ✅ U-Boot兼容的设备树结构
- ✅ 纯库接口，无CLI依赖
- ✅ 单配置支持
- ✅ CRC32校验

## 快速开始

### 添加依赖

```toml
[dependencies]
mkimage = "0.1.0"
```

### 基本用法

```rust
use mkimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

// 创建FIT镜像配置
let config = FitImageConfig::new("My FIT Image")
    .with_kernel(
        ComponentConfig::new("kernel", kernel_data)
            .with_load_address(0x80080000)
            .with_entry_point(0x80080000)
    )
    .with_fdt(
        ComponentConfig::new("fdt", fdt_data)
            .with_load_address(0x82000000)
    )
    .with_kernel_compression(true);

// 构建FIT镜像
let mut builder = FitImageBuilder::new();
let fit_data = builder.build(config)?;

// 保存到文件
std::fs::write("image.fit", fit_data)?;
```

## 核心组件

### FitImageConfig

FIT镜像的主配置结构：

```rust
pub struct FitImageConfig {
    pub description: String,
    pub kernel: Option<ComponentConfig>,
    pub fdt: Option<ComponentConfig>,
    pub ramdisk: Option<ComponentConfig>,
    pub compress_kernel: bool,
}
```

### ComponentConfig

单个组件的配置：

```rust
pub struct ComponentConfig {
    pub name: String,
    pub data: Vec<u8>,
    pub load_address: Option<u64>,
    pub entry_point: Option<u64>,
}
```

### FitImageBuilder

主要的构建器接口：

```rust
impl FitImageBuilder {
    pub fn new() -> Self;
    pub fn build(&mut self, config: FitImageConfig) -> Result<Vec<u8>>;
    pub fn build_with_compressor(
        &mut self,
        config: FitImageConfig,
        compressor: Box<dyn CompressionInterface>
    ) -> Result<Vec<u8>>;
}
```

## 示例

### 完整FIT镜像

```rust
use mkimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

fn create_complete_fit() -> Result<(), Box<dyn std::error::Error>> {
    let config = FitImageConfig::new("Complete FIT Image")
        .with_kernel(
            ComponentConfig::new("linux", kernel_data)
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000)
        )
        .with_fdt(
            ComponentConfig::new("devicetree", fdt_data)
                .with_load_address(0x82000000)
        )
        .with_ramdisk(
            ComponentConfig::new("initramfs", ramdisk_data)
                .with_load_address(0x84000000)
        )
        .with_kernel_compression(true);

    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)?;

    std::fs::write("complete.fit", fit_data)?;
    println!("FIT image created successfully!");
    Ok(())
}
```

## 压缩

库支持gzip压缩内核数据：

```rust
let config = FitImageConfig::new("Compressed FIT")
    .with_kernel(kernel_component)
    .with_kernel_compression(true); // 启用gzip压缩
```

## 兼容性

- ✅ U-Boot FIT格式兼容
- ✅ 标准设备树结构
- ✅ ARM64架构支持
- ✅ Linux OS支持

## 构建和测试

```bash
# 构建库
cargo build --lib

# 运行测试
cargo test --lib

# 运行示例
cargo run --example basic_usage
cargo run --example compression_test
cargo run --example full_fit_test
```

## API文档

运行以下命令生成API文档：

```bash
cargo doc --open
```

## 许可证

MIT OR Apache-2.0

## 贡献

欢迎提交Issue和Pull Request！

## 更新日志

### v0.1.0
- 初始版本
- 完整的FIT镜像创建功能
- gzip压缩支持
- kernel、FDT、ramdisk组件支持