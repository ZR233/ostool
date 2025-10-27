use std::process::Command;
use fitimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

/// 测试不同配置节点名称的 U-Boot 兼容性
#[test]
fn test_configuration_naming_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试不同配置节点名称的 U-Boot 兼容性...");

    let kernel_data = b"Test kernel for compatibility test";
    let fdt_data = b"Test FDT for compatibility test";

    // 测试格式列表
    let test_configs = vec![
        ("conf@1", "标准格式（可能不兼容老版本）"),
        ("config-1", "横线格式（更兼容）"),
        ("conf1", "无格式（最兼容）"),
        ("default", "原始格式"),
    ];

    for (config_name, description) in test_configs {
        println!("\n🔍 测试配置格式: {} ({})", config_name, description);

        // 创建使用指定配置名称的 FIT image
        let config = FitImageConfig::new("Compatibility Test FIT")
            .with_kernel(
                ComponentConfig::new("kernel", kernel_data.to_vec())
                    .with_load_address(0x80080000)
                    .with_entry_point(0x80080000)
            )
            .with_fdt(
                ComponentConfig::new("fdt", fdt_data.to_vec())
                    .with_load_address(0x82000000)
            )
            .with_default_config(config_name)
            .with_configuration(config_name, "Test configuration",
                               Some("kernel"), Some("fdt"), None::<String>)
            .with_kernel_compression(false);

        // 生成 FIT image
        let mut builder = FitImageBuilder::new();
        let fit_data = builder.build(config)?;

        // 保存到临时文件
        let temp_path = &format!("/tmp/test_{}.fit", config_name.replace("@", "_"));
        std::fs::write(temp_path, fit_data)?;

        println!("  ✅ 生成 FIT image: {}", temp_path);

        // 使用 dtc 检查结构
        let dtc_output = Command::new("dtc")
            .args(&["-I", "dtb", "-O", "dts", temp_path])
            .output()?;

        if dtc_output.status.success() {
            let dts_content = String::from_utf8_lossy(&dtc_output.stdout);

            // 检查关键配置
            let has_default = dts_content.contains(&format!("default = \"{}\"", config_name));
            let has_config = dts_content.contains(&format!("{} {{", config_name));

            println!("  📋 default 属性: {}", if has_default { "✅ 正确" } else { "❌ 缺失" });
            println!("  📋 配置节点: {}", if has_config { "✅ 存在" } else { "❌ 缺失" });

            if has_default && has_config {
                println!("  ✅ 格式 '{}' 结构正确", config_name);
            } else {
                println!("  ❌ 格式 '{}' 结构有问题", config_name);
            }
        }

        // 使用 dumpimage 检查
        let dump_output = Command::new("dumpimage")
            .args(&["-l", temp_path])
            .output()?;

        if dump_output.status.success() {
            let dump_content = String::from_utf8_lossy(&dump_output.stdout);

            // 检查是否能识别配置
            let has_default_config = dump_content.contains(&format!("Default Configuration: '{}'", config_name));
            let has_config_section = dump_content.contains(&format!("Configuration 0 ({})", config_name));

            println!("  📋 dumpimage 默认配置: {}", if has_default_config { "✅ 识别" } else { "❌ 未识别" });
            println!("  📋 dumpimage 配置节点: {}", if has_config_section { "✅ 识别" } else { "❌ 未识别" });

            if has_default_config && has_config_section {
                println!("  ✅ 格式 '{}' dumpimage 兼容", config_name);
            } else {
                println!("  ❌ 格式 '{}' dumpimage 不兼容", config_name);
            }
        } else {
            println!("  ❌ dumpimage 无法解析: {}", String::from_utf8_lossy(&dump_output.stderr));
        }

        // 清理临时文件
        std::fs::remove_file(temp_path)?;
    }

    Ok(())
}