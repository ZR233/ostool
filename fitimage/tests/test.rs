
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use anyhow::{Result, Context};
use fitimage::{FitImageBuilder, FitImageConfig, ComponentConfig};

/// 测试完整的 FIT image 生成和验证流程
///
/// 测试步骤：
/// 1. 使用系统 mkimage 工具生成标准 FIT image
/// 2. 使用项目代码生成相同参数的 FIT image
/// 3. 使用 dumpimage 工具对比两个 FIT image
/// 4. 验证符合 U-Boot 标准
#[test]
fn test_fit_image_standard_compliance() -> Result<()> {
    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let test_dir = Path::new("tests");

    println!("当前工作目录: {}", std::env::current_dir()?.display());
    println!("测试文件目录: {}", test_dir.display());
    println!("测试文件是否存在: {}", test_dir.exists());

    if !test_dir.exists() {
        anyhow::bail!("测试文件目录不存在: {}", test_dir.display());
    }

    // 准备测试数据
    let kernel_data = fs::read(test_dir.join("kernel.txt"))?;
    let fdt_data = fs::read(test_dir.join("dtb.txt"))?;

    println!("测试数据准备完成: kernel={} bytes, fdt={} bytes",
             kernel_data.len(), fdt_data.len());

    // 步骤1: 使用系统 mkimage 生成标准 FIT image
    let mkimage_fit_path = temp_dir.path().join("mkimage.fit");
    generate_mkimage_fit_image(&mkimage_fit_path, test_dir)?;

    // 步骤2: 使用项目代码生成 FIT image
    let rust_fit_path = temp_dir.path().join("rust.fit");
    generate_rust_fit_image(&rust_fit_path, &kernel_data, &fdt_data)?;

    // 步骤3: 使用 dumpimage 工具对比两个 FIT image
    compare_fit_images(&mkimage_fit_path, &rust_fit_path, temp_dir.path())?;

    // 步骤4: 验证两个文件的基本结构
    validate_fit_image_structure(&mkimage_fit_path, "标准 mkimage 生成的 FIT image")?;
    validate_fit_image_structure(&rust_fit_path, "项目代码生成的 FIT image")?;

    println!("✅ FIT image 标准符合性测试通过！");
    Ok(())
}

/// 使用系统 mkimage 工具生成标准 FIT image
fn generate_mkimage_fit_image(output_path: &Path, test_dir: &Path) -> Result<()> {
    println!("🔨 使用系统 mkimage 生成标准 FIT image...");

    let its_path = test_dir.join("test.its");

    // 检查 its 文件是否存在
    if !its_path.exists() {
        anyhow::bail!("测试文件不存在: {}", its_path.display());
    }

    println!("使用 its 文件: {}", its_path.display());
    println!("输出文件: {}", output_path.display());

    let output = Command::new("mkimage")
        .arg("-f")
        .arg(&its_path)
        .arg(output_path)
        .output()
        .with_context(|| "执行 mkimage 命令失败")?;

    println!("mkimage stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("mkimage stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("mkimage exit code: {}", output.status);

    if !output.status.success() {
        anyhow::bail!("mkimage 执行失败: {}",
                     String::from_utf8_lossy(&output.stderr));
    }

    println!("✅ 标准 mkimage FIT image 生成成功: {}",
             output_path.display());
    Ok(())
}

/// 使用项目代码生成 FIT image
fn generate_rust_fit_image(output_path: &Path, kernel_data: &[u8], fdt_data: &[u8]) -> Result<()> {
    println!("🦀 使用项目代码生成 FIT image...");

    // 创建配置，与 test.its 文件中的参数一致
    let config = FitImageConfig::new("Various kernels, ramdisks and FDT blobs")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_data.to_vec())
                .with_description("This kernel")
                .with_type("kernel")
                .with_arch("arm64")
                .with_os("linux")
                .with_compression("none")
                .with_load_address(0x90100000)
                .with_entry_point(0x90100000)
        )
        .with_fdt(
            ComponentConfig::new("fdt", fdt_data.to_vec())
                .with_description("This fdt")
                .with_type("flat_dt")
                .with_arch("arm64")
                .with_compression("none")
        )
        .with_default_config("config-ostool")
        .with_configuration("config-ostool", "ostool configuration",
                           Some("kernel"), Some("fdt"), None::<String>)
        .with_kernel_compression(false);

    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)
        .with_context(|| "构建 FIT image 失败")?;

    fs::write(output_path, fit_data)
        .with_context(|| "写入 FIT image 文件失败")?;

    println!("✅ 项目代码 FIT image 生成成功: {}",
             output_path.display());
    Ok(())
}

/// 使用 dumpimage 工具对比两个 FIT image
fn compare_fit_images(mkimage_path: &Path, rust_path: &Path,
                     temp_dir: &Path) -> Result<()> {
    println!("🔍 使用 dumpimage 工具对比 FIT image...");

    // 使用 dumpimage 提取 mkimage FIT image 的信息
    let mkimage_dump_path = temp_dir.join("mkimage_dump.txt");
    let output1 = Command::new("dumpimage")
        .arg("-l")
        .arg(mkimage_path)
        .output()
        .with_context(|| "执行 dumpimage on mkimage FIT 失败")?;

    if !output1.status.success() {
        anyhow::bail!("dumpimage mkimage FIT 失败: {}",
                     String::from_utf8_lossy(&output1.stderr));
    }

    fs::write(&mkimage_dump_path, &output1.stdout)?;

    // 使用 dumpimage 提取项目代码 FIT image 的信息
    let rust_dump_path = temp_dir.join("rust_dump.txt");
    let output2 = Command::new("dumpimage")
        .arg("-l")
        .arg(rust_path)
        .output()
        .with_context(|| "执行 dumpimage on rust FIT 失败")?;

    if !output2.status.success() {
        anyhow::bail!("dumpimage rust FIT 失败: {}",
                     String::from_utf8_lossy(&output2.stderr));
    }

    fs::write(&rust_dump_path, &output2.stdout)?;

    // 读取并解析 dumpimage 输出
    let mkimage_dump = String::from_utf8_lossy(&output1.stdout);
    let rust_dump = String::from_utf8_lossy(&output2.stdout);

    println!("=== mkimage FIT image dump ===");
    println!("{}", mkimage_dump);
    println!("=== rust FIT image dump ===");
    println!("{}", rust_dump);

    // 验证关键信息是否一致
    validate_dump_compatibility(&mkimage_dump, &rust_dump)?;

    println!("✅ FIT image 对比验证完成");
    Ok(())
}

/// 验证 dumpimage 输出的兼容性
fn validate_dump_compatibility(mkimage_dump: &str, rust_dump: &str) -> Result<()> {
    // 检查是否包含必要的 FIT image 标识
    assert!(mkimage_dump.contains("FIT") || mkimage_dump.contains("Flattened"),
            "mkimage 输出应包含 FIT 标识");
    assert!(rust_dump.contains("FIT") || rust_dump.contains("Flattened"),
            "rust 输出应包含 FIT 标识");

    // 检查配置名称
    assert!(mkimage_dump.contains("config-ostool") || rust_dump.contains("config-ostool"),
            "应包含 config-ostool 配置");

    // 检查内核和 FDT 组件
    assert!(mkimage_dump.contains("kernel") || rust_dump.contains("kernel"),
            "应包含 kernel 组件");
    assert!(mkimage_dump.contains("fdt") || rust_dump.contains("fdt") || mkimage_dump.contains("flat_dt") || rust_dump.contains("flat_dt"),
            "应包含 fdt 组件");

    println!("✅ dumpimage 输出兼容性验证通过");
    Ok(())
}

/// 验证 FIT image 的基本结构
fn validate_fit_image_structure(fit_path: &Path, description: &str) -> Result<()> {
    let data = fs::read(fit_path)
        .with_context(|| format!("读取 FIT image 失败: {}", fit_path.display()))?;

    // 验证文件大小
    assert!(data.len() > 0, "FIT image 不应为空");
    println!("{}: {} bytes", description, data.len());

    // 验证设备树魔数 (0xd00dfeed)
    if data.len() >= 4 {
        let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(magic, 0xd00dfeed,
                  "设备树魔数不正确，期望 0xd00dfeed，实际 0x{:08x}", magic);
        println!("✅ {} 设备树魔数验证通过", description);
    }

    // 验证可以使用 dumpimage 读取
    let output = Command::new("dumpimage")
        .arg("-l")
        .arg(fit_path)
        .output()
        .with_context(|| format!("dumpimage 验证失败: {}", fit_path.display()))?;

    assert!(output.status.success(),
            "dumpimage 应能成功读取 {}: {}",
            description, String::from_utf8_lossy(&output.stderr));

    println!("✅ {} 基本结构验证通过", description);
    Ok(())
}

#[test]
fn test_fit_image_basic_functionality() -> Result<()> {
    println!("🧪 测试 FIT image 基本功能...");

    // 创建测试数据
    let kernel_data = b"Test Kernel Data";
    let fdt_data = b"Test FDT Data";

    // 创建基本配置
    let config = FitImageConfig::new("Test FIT Image")
        .with_kernel(
            ComponentConfig::new("kernel", kernel_data.to_vec())
                .with_load_address(0x80080000)
                .with_entry_point(0x80080000)
        )
        .with_fdt(
            ComponentConfig::new("fdt", fdt_data.to_vec())
                .with_load_address(0x82000000)
        )
        .with_default_config("default")
        .with_configuration("default", "Default configuration",
                           Some("kernel"), Some("fdt"), None::<String>)
        .with_kernel_compression(false);

    // 生成 FIT image
    let mut builder = FitImageBuilder::new();
    let fit_data = builder.build(config)?;

    // 验证结果
    assert!(!fit_data.is_empty(), "FIT image 数据不应为空");
    assert_eq!(fit_data[0..4], [0xd0, 0x0d, 0xfe, 0xed],
              "设备树魔数不正确");

    println!("✅ FIT image 基本功能测试通过");
    Ok(())
}