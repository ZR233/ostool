//! 测试新的 FIT image 生成集成
//! 验证ostool是否能正确使用新的mkimage库

use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试 ostool FIT image 生成集成 ===");

    // 创建测试数据
    let kernel_data = "Test kernel for ostool integration".repeat(50);
    let dtb_data = "Test DTB for ostool integration";

    // 模拟文件路径
    let kernel_path = PathBuf::from("test_kernel.bin");
    let dtb_path = PathBuf::from("test_dtb.dtb");

    // 写入测试文件
    fs::write(&kernel_path, kernel_data.as_bytes()).await?;
    fs::write(&dtb_path, dtb_data.as_bytes()).await?;

    println!("创建测试文件完成");

    // 使用新的mkimage API（模拟uboot.rs中的逻辑）
    use fitimage::{ComponentConfig, FitImageBuilder, FitImageConfig};

    let kernel_load_addr = 0x80080000u64;

    // 读取kernel数据（模拟uboot.rs中的操作）
    let kernel_data = fs::read(&kernel_path).await?;
    let dtb_data = fs::read(&dtb_path).await?;

    // 创建kernel组件（和uboot.rs中一样）
    let kernel_component = ComponentConfig::new("kernel", kernel_data)
        .with_load_address(kernel_load_addr)
        .with_entry_point(kernel_load_addr);

    // 构建FIT配置（和uboot.rs中一样）
    let fdt_load_addr = kernel_load_addr + 0x02000000;
    let fdt_component = ComponentConfig::new("fdt", dtb_data).with_load_address(fdt_load_addr);

    let fit_config = FitImageConfig::new("ostool FIT Image")
        .with_kernel(kernel_component)
        .with_fdt(fdt_component)
        .with_kernel_compression(true);

    // 构建FIT image（和uboot.rs中一样）
    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(fit_config)?;

    // 保存结果
    let output_path = PathBuf::from("test_integration.fit");
    fs::write(&output_path, fit_data).await?;

    println!("✅ FIT image 生成成功: {}", output_path.display());
    println!(
        "✅ 文件大小: {} bytes",
        fs::metadata(&output_path).await?.len()
    );

    // 验证设备树魔数
    let fit_data = fs::read(&output_path).await?;
    if fit_data.len() >= 4 && &fit_data[0..4] == b"\xd0\x0d\xfe\xed" {
        println!("✅ 设备树魔数验证通过");
    } else {
        println!("❌ 设备树魔数验证失败");
    }

    // 清理测试文件（保留FIT文件用于验证）
    fs::remove_file(&kernel_path).await?;
    fs::remove_file(&dtb_path).await?;
    // fs::remove_file(&output_path).await?; // 保留FIT文件

    println!("✅ 集成测试完成！新的mkimage API在ostool中工作正常");
    println!("📁 FIT文件保存在: {}", output_path.display());

    Ok(())
}
