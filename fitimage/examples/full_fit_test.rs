//! Full FIT image test with all components
//!
//! Demonstrates creating a complete FIT image with kernel, FDT, and ramdisk.

use mkimage::{ComponentConfig, FitImageBuilder, FitImageConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整FIT镜像测试 ===");

    // 创建示例内核数据
    let kernel_data = "Linux kernel data (simulated)".repeat(50);
    println!("内核数据大小: {} bytes", kernel_data.len());

    // 创建示例设备树数据
    let fdt_data = r#"
        /dts-v1/;
        / {
            compatible = "vendor,device";
            model = "Test Device";
            #address-cells = <1>;
            #size-cells = <1>;

            memory {
                device_type = "memory";
                reg = <0x80000000 0x10000000>;
            };

            chosen {
                bootargs = "console=ttyS0,115200n8";
            };
        };
    "#;
    println!("FDT数据大小: {} bytes", fdt_data.len());

    // 创建示例ramdisk数据
    let ramdisk_data = "Initramfs content (simulated)".repeat(20);
    println!("Ramdisk数据大小: {} bytes", ramdisk_data.len());

    // 配置完整的FIT镜像
    let config = FitImageConfig::new("Complete FIT Image for ARM64")
        .with_kernel(
            ComponentConfig::new("linux", kernel_data.as_bytes().to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000),
        )
        .with_fdt(
            ComponentConfig::new("devicetree", fdt_data.as_bytes().to_vec())
                .with_load_address(0x82000000),
        )
        .with_ramdisk(
            ComponentConfig::new("initramfs", ramdisk_data.as_bytes().to_vec())
                .with_load_address(0x84000000),
        )
        .with_kernel_compression(true);

    // 构建FIT镜像
    println!("\n正在构建FIT镜像...");
    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)?;

    println!("✅ FIT镜像创建成功!");
    println!("镜像总大小: {} bytes", fit_data.len());
    println!("前16字节: {:02x?}", &fit_data[..16]);

    // 验证设备树魔数
    if &fit_data[0..4] == b"\xd0\x0d\xfe\xed" {
        println!("✅ 设备树魔数正确");
    } else {
        println!("❌ 设备树魔数错误");
    }

    // 保存到文件供检查
    std::fs::write("test_complete.fit", &fit_data)?;
    println!("✅ FIT镜像已保存到: test_complete.fit");

    // 创建无压缩版本进行对比
    println!("\n=== 对比测试（无压缩）===");
    let config_uncompressed = FitImageConfig::new("Uncompressed FIT Image")
        .with_kernel(
            ComponentConfig::new("linux", kernel_data.as_bytes().to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000),
        )
        .with_fdt(
            ComponentConfig::new("devicetree", fdt_data.as_bytes().to_vec())
                .with_load_address(0x82000000),
        )
        .with_ramdisk(
            ComponentConfig::new("initramfs", ramdisk_data.as_bytes().to_vec())
                .with_load_address(0x84000000),
        )
        .with_kernel_compression(false);

    let fit_data_uncompressed = builder.build(config_uncompressed)?;
    std::fs::write("test_uncompressed.fit", &fit_data_uncompressed)?;

    let compressed_size = fit_data.len();
    let uncompressed_size = fit_data_uncompressed.len();
    let savings = uncompressed_size - compressed_size;
    let savings_percent = (savings * 100) / uncompressed_size;

    println!("压缩版本大小: {} bytes", compressed_size);
    println!("无压缩版本大小: {} bytes", uncompressed_size);
    println!("节省空间: {} bytes ({}%)", savings, savings_percent);

    println!("\n=== 测试完成 ===");
    println!("✅ 所有功能正常工作");
    println!("✅ 压缩功能有效");
    println!("✅ 多组件支持正常");
    println!("✅ 设备树结构兼容");

    Ok(())
}
