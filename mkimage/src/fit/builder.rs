//! FIT Image构建器
//!
//! 提供流式API来构建FIT images，支持多种组件组合和配置


use crate::fit::types::{FitImage, FitComponent, FitComponentType, FitConfiguration};
use crate::compression::traits::CompressionInterface;
use crate::error::Result;

/// FIT Image构建器
/// 提供流式API来构建FIT image
pub struct FitImageBuilder {
    image: FitImage,
    compressor: Box<dyn CompressionInterface>,
    default_config: String,
    config_counter: u32,
}

impl FitImageBuilder {
    /// 创建新的FIT构建器
    pub fn new() -> Self {
        Self {
            image: FitImage::new(),
            compressor: Box::new(crate::compression::gzip::GzipCompressor::default()),
            default_config: "default".to_string(),
            config_counter: 0,
        }
    }

    /// 创建带有指定压缩器的FIT构建器
    pub fn with_compressor(compressor: Box<dyn CompressionInterface>) -> Self {
        Self {
            image: FitImage::new(),
            compressor,
            default_config: "default".to_string(),
            config_counter: 0,
        }
    }

    /// 设置FIT image描述
    pub fn description(mut self, desc: &str) -> Self {
        self.image.description = desc.to_string();
        self
    }

    /// 添加kernel组件
    pub fn kernel(mut self, name: &str, data: Vec<u8>, load_address: u64, entry_point: u64) -> Self {
        let mut component = FitComponent::new(FitComponentType::Kernel, data)
            .with_description(&format!("{} kernel", name))
            .with_load_address(load_address)
            .with_entry_point(entry_point);

        // 对kernel数据进行压缩
        let compressed_data = self.compressor.compress(&component.data)
            .expect("Failed to compress kernel data");
        component.data = compressed_data;
        component.compression = self.compressor.get_compression_type();

        self.image.add_image(name.to_string(), component);
        self
    }

    /// 添加设备树组件
    pub fn fdt(mut self, name: &str, data: Vec<u8>, load_address: u64) -> Self {
        let mut component = FitComponent::new(FitComponentType::Fdt, data)
            .with_description(&format!("{} device tree", name))
            .with_load_address(load_address);

        // 设备树通常不压缩，但为了通用性我们还是使用配置的压缩器
        let compressed_data = self.compressor.compress(&component.data)
            .expect("Failed to compress fdt data");
        component.data = compressed_data;
        component.compression = self.compressor.get_compression_type();

        self.image.add_image(name.to_string(), component);
        self
    }

    /// 添加ramdisk组件
    pub fn ramdisk(mut self, name: &str, data: Vec<u8>, load_address: u64) -> Self {
        let mut component = FitComponent::new(FitComponentType::Ramdisk, data)
            .with_description(&format!("{} ramdisk", name))
            .with_load_address(load_address);

        let compressed_data = self.compressor.compress(&component.data)
            .expect("Failed to compress ramdisk data");
        component.data = compressed_data;
        component.compression = self.compressor.get_compression_type();

        self.image.add_image(name.to_string(), component);
        self
    }

    /// 添加FIT配置
    pub fn configuration(mut self, kernel: &str, fdt: &str) -> Self {
        let config = FitConfiguration::new()
            .with_description(&format!("Boot with {} and {}", kernel, fdt))
            .with_kernel(kernel)
            .with_fdt(fdt)
            .with_compatible("test-vendor,test-device");

        let config_name = self.config_counter.to_string();
        self.config_counter += 1;

        // 第一个配置作为默认配置
        if self.config_counter == 1 {
            self.default_config = config_name.clone();
        }

        self.image.add_configuration(config_name, config);
        self
    }

    /// 设置默认配置名称
    pub fn default_config(mut self, config_name: &str) -> Self {
        self.default_config = config_name.to_string();
        self
    }

    /// 构建FIT二进制数据
    pub fn build(self) -> Result<Vec<u8>> {
        // 设置默认配置
        let mut final_image = self.image;
        if !final_image.configurations.is_empty() {
            final_image.default = Some(self.default_config);
        }

        // 使用序列化器将FIT结构转换为设备树格式
        let serializer = crate::fit::serializer::FitSerializer::new();
        serializer.serialize(&final_image, &*self.compressor)
    }

    /// 构建并保存到文件
    pub fn build_and_save(self, path: &str) -> Result<()> {
        let data = self.build()?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

impl Default for FitImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便利函数用于快速创建简单FIT
pub mod convenience {
    use super::*;
    use crate::image_types::Compression;

    /// 创建简单的kernel + fdt FIT image
    pub fn simple_kernel_fdt(
        kernel_data: Vec<u8>,
        fdt_data: Vec<u8>,
        kernel_load: u64,
        kernel_entry: u64,
        fdt_load: u64,
    ) -> FitImageBuilder {
        FitImageBuilder::new()
            .description("Simple FIT with kernel and FDT")
            .kernel("kernel", kernel_data, kernel_load, kernel_entry)
            .fdt("fdt", fdt_data, fdt_load)
            .configuration("kernel", "fdt")
    }

    /// 创建带压缩的kernel + fdt FIT image
    pub fn compressed_kernel_fdt(
        kernel_data: Vec<u8>,
        fdt_data: Vec<u8>,
        kernel_load: u64,
        kernel_entry: u64,
        fdt_load: u64,
        compression: Compression,
    ) -> FitImageBuilder {
        let compressor = crate::compression::traits::create_compressor(compression);
        FitImageBuilder::with_compressor(compressor)
            .description("Compressed FIT with kernel and FDT")
            .kernel("kernel", kernel_data, kernel_load, kernel_entry)
            .fdt("fdt", fdt_data, fdt_load)
            .configuration("kernel", "fdt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::gzip::GzipCompressor;

    #[test]
    fn test_fit_image_builder() {
        let builder = FitImageBuilder::new()
            .description("Test FIT image")
            .kernel("test_kernel", vec![1, 2, 3, 4], 0x80080000, 0x80080000)
            .fdt("test_fdt", vec![5, 6, 7, 8], 0x82000000)
            .configuration("test_kernel", "test_fdt");

        assert_eq!(builder.image.description, "Test FIT image");
        assert_eq!(builder.image.images.len(), 2);
        assert_eq!(builder.image.configurations.len(), 1);
    }

    #[test]
    fn test_convenience_builder() {
        let kernel_data = vec![1, 2, 3, 4];
        let fdt_data = vec![5, 6, 7, 8];

        let builder = convenience::simple_kernel_fdt(
            kernel_data.clone(),
            fdt_data.clone(),
            0x80080000,
            0x80080000,
            0x82000000,
        );

        assert_eq!(builder.image.description, "Simple FIT with kernel and FDT");

        // 验证kernel组件
        let kernel_component = builder.image.images.get("test_kernel").unwrap();
        assert_eq!(kernel_component.component_type, FitComponentType::Kernel);
        assert_eq!(kernel_component.load_address, Some(0x80080000));

        // 验证fdt组件
        let fdt_component = builder.image.images.get("test_fdt").unwrap();
        assert_eq!(fdt_component.component_type, FitComponentType::Fdt);
        assert_eq!(fdt_component.load_address, Some(0x82000000));
    }

    #[test]
    fn test_build_without_configuration() {
        let builder = FitImageBuilder::new()
            .kernel("kernel", vec![1, 2, 3], 0x80080000, 0x80080000)
            .fdt("fdt", vec![4, 5, 6], 0x82000000);

        // 没有明确配置时应该自动创建默认配置
        let _result = builder.build();

        assert!(!builder.image.configurations.is_empty());
        assert!(builder.image.default.is_some());
    }
}