//! 使用 txt 文件测试 FIT image 生成和导出

use fitimage::{ComponentConfig, FitImageBuilder, FitImageConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 使用 txt 文件测试 FIT image ===\n");

    // 读取外层的测试文件
    let kernel_data = fs::read_to_string("../test_kernel.txt")?;
    let fdt_data = fs::read_to_string("../test_dtb.txt")?;
    let ramfs_data = fs::read_to_string("../test_ramfs.txt")?;

    println!("内核数据大小: {} bytes", kernel_data.len());
    println!("FDT数据大小: {} bytes", fdt_data.len());
    println!("Ramdisk数据大小: {} bytes", ramfs_data.len());

    // 创建包含 txt 内容的 FIT image
    let config = FitImageConfig::new("TXT Test FIT Image - Kernel DTB RamFS")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_data.as_bytes().to_vec())
                .with_load_address(0x40080000)
                .with_entry_point(0x40080000),
        )
        .with_fdt(
            ComponentConfig::new("fdt", fdt_data.as_bytes().to_vec())
                .with_load_address(0x42000000),
        )
        .with_ramdisk(
            ComponentConfig::new("ramdisk", ramfs_data.as_bytes().to_vec())
                .with_load_address(0x44000000),
        )
        .with_kernel_compression(false); // 先不压缩，便于验证

    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)?;

    println!("\n正在构建FIT镜像...");
    println!("✅ FIT镜像创建成功!");
    println!("镜像总大小: {} bytes", fit_data.len());
    println!("前16字节: {:02x?}", &fit_data[..16]);

    // 验证设备树魔数
    if &fit_data[0..4] == b"\xd0\x0d\xfe\xed" {
        println!("✅ 设备树魔数正确");
    } else {
        println!("❌ 设备树魔数错误");
    }

    // 保存到外层目录
    fs::write("../test_txt_fit.fit", &fit_data)?;
    println!("✅ FIT镜像已保存到: ../test_txt_fit.fit");

    // 创建压缩版本
    println!("\n创建压缩版本...");
    let config_compressed = FitImageConfig::new("TXT Test FIT Image - Compressed")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_data.as_bytes().to_vec())
                .with_load_address(0x40080000)
                .with_entry_point(0x40080000),
        )
        .with_fdt(
            ComponentConfig::new("fdt", fdt_data.as_bytes().to_vec())
                .with_load_address(0x42000000),
        )
        .with_ramdisk(
            ComponentConfig::new("ramdisk", ramfs_data.as_bytes().to_vec())
                .with_load_address(0x44000000),
        )
        .with_kernel_compression(true);

    let fit_data_compressed = builder.build(config_compressed)?;
    fs::write("../test_txt_fit_compressed.fit", &fit_data_compressed)?;

    println!("压缩版本大小: {} bytes", fit_data_compressed.len());
    println!("非压缩版本大小: {} bytes", fit_data.len());
    let savings = fit_data.len() - fit_data_compressed.len();
    println!("节省空间: {} bytes ({}%)", savings, (savings * 100) / fit_data.len());
    println!("✅ 压缩版本已保存到: ../test_txt_fit_compressed.fit");

    println!("\n=== 测试完成 ===");
    println!("✅ txt 文件内容已成功集成到 FIT image");
    println!("✅ 准备使用 dumpimage 进行导出测试");

    Ok(())
}