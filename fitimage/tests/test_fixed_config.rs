use fitimage::{ComponentConfig, FitImageBuilder, FitImageConfig};
use std::process::Command;

/// 测试修复后的默认配置格式
#[test]
fn test_fixed_default_config_format() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试修复后的默认配置格式...");

    // 创建测试数据
    let kernel_data = b"Test kernel for fixed format";
    let fdt_data = b"Test FDT for fixed format";

    // 创建默认配置（应该使用 config-1 格式）
    let config = FitImageConfig::new("Fixed Format Test FIT")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_data.to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000),
        )
        .with_fdt(ComponentConfig::new("fdt", fdt_data.to_vec()).with_load_address(0x82000000));

    // 生成 FIT image
    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)?;

    // 保存到临时文件
    let temp_path = "/tmp/fixed_test.fit";
    std::fs::write(temp_path, fit_data)?;

    println!("✅ 修复后的 FIT image 生成: {}", temp_path);

    // 使用 dtc 检查结构
    let dtc_output = Command::new("dtc")
        .args(["-I", "dtb", "-O", "dts", temp_path])
        .output()?;

    if dtc_output.status.success() {
        let dts_content = String::from_utf8_lossy(&dtc_output.stdout);
        println!("=== 修复后的 FIT image 设备树结构 ===");
        println!("{}", dts_content);

        // 验证关键配置
        let has_default_attr = dts_content.contains("default = \"config-1\"");
        let has_config_node = dts_content.contains("config-1 {");

        assert!(has_default_attr, "default 属性应该指向 'config-1'");
        assert!(has_config_node, "配置节点 'config-1' 应该存在");

        if has_default_attr && has_config_node {
            println!("✅ 修复成功！使用 'config-1' 格式");
        } else {
            panic!("修复失败：配置格式不正确");
        }
    } else {
        panic!(
            "dtc 解析失败: {}",
            String::from_utf8_lossy(&dtc_output.stderr)
        );
    }

    // 使用 dumpimage 验证兼容性
    let dump_output = Command::new("dumpimage").args(["-l", temp_path]).output()?;

    if dump_output.status.success() {
        let dump_content = String::from_utf8_lossy(&dump_output.stdout);
        println!("=== dumpimage 输出 ===");
        println!("{}", dump_content);

        // 验证 dumpimage 能识别配置
        let has_config_section = dump_content.contains("Configuration 0 (config-1)");
        let has_kernel_ref = dump_content.contains("Kernel:       kernel");
        let has_fdt_ref = dump_content.contains("FDT:          fdt");

        assert!(has_config_section, "dumpimage 应该识别配置节点 'config-1'");
        assert!(has_kernel_ref, "dumpimage 应该识别 kernel 引用");
        assert!(has_fdt_ref, "dumpimage 应该识别 FDT 引用");

        if has_config_section && has_kernel_ref && has_fdt_ref {
            println!("✅ dumpimage 兼容性验证成功！");
        } else {
            panic!("dumpimage 兼容性验证失败");
        }
    } else {
        panic!(
            "dumpimage 解析失败: {}",
            String::from_utf8_lossy(&dump_output.stderr)
        );
    }

    // 清理临时文件
    std::fs::remove_file(temp_path)?;

    println!("🎉 所有测试通过！修复成功解决了 U-Boot 兼容性问题。");

    Ok(())
}
