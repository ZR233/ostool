//! FIT (Flattened Image Tree) 数据类型定义
//!
//! 定义了FIT image相关的所有数据结构

use serde::{Deserialize, Serialize};
use crate::image_types::Compression;
use std::collections::HashMap;

/// FIT Image主结构
/// 包含描述信息、镜像组件集合和配置集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitImage {
    /// 描述信息
    pub description: String,

    /// 镜像组件集合
    /// 键为组件名称，值为组件实例
    pub images: HashMap<String, FitComponent>,

    /// 配置集合
    /// 键为配置名称，值为配置实例
    pub configurations: HashMap<String, FitConfiguration>,

    /// 默认配置名称
    pub default: Option<String>,
}

impl FitImage {
    /// 创建新的FIT image
    pub fn new() -> Self {
        Self {
            description: String::new(),
            images: HashMap::new(),
            configurations: HashMap::new(),
            default: None,
        }
    }

    /// 添加镜像组件
    pub fn add_image(&mut self, name: String, component: FitComponent) {
        self.images.insert(name, component);
    }

    /// 添加配置
    pub fn add_configuration(&mut self, name: String, config: FitConfiguration) {
        if self.default.is_none() {
            self.default = Some(name.clone());
        }
        self.configurations.insert(name, config);
    }

    /// 获取默认配置
    pub fn get_default_config(&self) -> Option<&FitConfiguration> {
        self.default.as_ref()
            .and_then(|default_name| self.configurations.get(default_name))
    }
}

/// FIT镜像组件
/// 可以是kernel、设备树、ramdisk等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitComponent {
    /// 组件描述
    pub description: String,

    /// 组件类型
    #[serde(rename = "type")]
    pub component_type: FitComponentType,

    /// 压缩类型
    pub compression: Compression,

    /// 数据载荷
    pub data: Vec<u8>,

    /// 加载地址
    #[serde(rename = "load", skip_serializing_if = "Option::is_none")]
    pub load_address: Option<u64>,

    /// 入口地址
    #[serde(rename = "entry", skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<u64>,

    /// 哈希值（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl FitComponent {
    /// 创建新的FIT组件
    pub fn new(component_type: FitComponentType, data: Vec<u8>) -> Self {
        Self {
            description: String::new(),
            component_type,
            compression: Compression::None,
            data,
            load_address: None,
            entry_point: None,
            hash: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// 设置压缩
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    /// 设置加载地址
    pub fn with_load_address(mut self, addr: u64) -> Self {
        self.load_address = Some(addr);
        self
    }

    /// 设置入口地址
    pub fn with_entry_point(mut self, addr: u64) -> Self {
        self.entry_point = Some(addr);
        self
    }

    /// 获取数据大小
    pub fn get_size(&self) -> usize {
        self.data.len()
    }
}

/// FIT组件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FitComponentType {
    /// 内核镜像
    Kernel,
    /// 扁平设备树 (Flattened Device Tree)
    #[serde(rename = "fdt")]
    Fdt,
    /// RAM磁盘镜像
    Ramdisk,
    /// 固件
    Firmware,
    /// 脚本
    Script,
    /// 文件系统
    Filesystem,
    /// 独立程序
    Standalone,
}

impl FitComponentType {
    /// 获取组件类型字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kernel => "kernel",
            Self::Fdt => "fdt",
            Self::Ramdisk => "ramdisk",
            Self::Firmware => "firmware",
            Self::Script => "script",
            Self::Filesystem => "filesystem",
            Self::Standalone => "standalone",
        }
    }
}

/// FIT配置
/// 定义如何组合多个组件形成一个可启动的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitConfiguration {
    /// 配置描述
    pub description: String,

    /// 关联的kernel组件名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,

    /// 关联的设备树组件名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fdt: Option<String>,

    /// 关联的ramdisk组件名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ramdisk: Option<String>,

    /// 关联的firmware组件名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<String>,

    /// 兼容性字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatible: Option<String>,

    /// 设备树兼容性字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fdt_compatible: Option<String>,
}

impl FitConfiguration {
    /// 创建新的FIT配置
    pub fn new() -> Self {
        Self {
            description: String::new(),
            kernel: None,
            fdt: None,
            ramdisk: None,
            firmware: None,
            compatible: None,
            fdt_compatible: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// 设置kernel组件
    pub fn with_kernel(mut self, kernel: &str) -> Self {
        self.kernel = Some(kernel.to_string());
        self
    }

    /// 设置设备树组件
    pub fn with_fdt(mut self, fdt: &str) -> Self {
        self.fdt = Some(fdt.to_string());
        self
    }

    /// 设置ramdisk组件
    pub fn with_ramdisk(mut self, ramdisk: &str) -> Self {
        self.ramdisk = Some(ramdisk.to_string());
        self
    }

    /// 设置兼容性
    pub fn with_compatible(mut self, compatible: &str) -> Self {
        self.compatible = Some(compatible.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_image_creation() {
        let mut fit = FitImage::new();
        assert!(fit.images.is_empty());
        assert!(fit.configurations.is_empty());
        assert!(fit.default.is_none());

        // 添加默认配置
        fit.default = Some("default".to_string());
        assert_eq!(fit.default, Some("default".to_string()));
    }

    #[test]
    fn test_fit_component_creation() {
        let data = b"test data";
        let component = FitComponent::new(FitComponentType::Kernel, data.to_vec())
            .with_description("Test kernel")
            .with_compression(Compression::Gzip)
            .with_load_address(0x80080000)
            .with_entry_point(0x80080000);

        assert_eq!(component.description, "Test kernel");
        assert_eq!(component.compression, Compression::Gzip);
        assert_eq!(component.load_address, Some(0x80080000));
        assert_eq!(component.entry_point, Some(0x80080000));
    }

    #[test]
    fn test_fit_configuration_creation() {
        let config = FitConfiguration::new()
            .with_description("Default configuration")
            .with_kernel("kernel1")
            .with_fdt("fdt1")
            .with_compatible("test-vendor,test-device");

        assert_eq!(config.description, "Default configuration");
        assert_eq!(config.kernel, Some("kernel1".to_string()));
        assert_eq!(config.fdt, Some("fdt1".to_string()));
        assert_eq!(config.compatible, Some("test-vendor,test-device".to_string()));
    }

    #[test]
    fn test_fit_component_type_as_str() {
        assert_eq!(FitComponentType::Kernel.as_str(), "kernel");
        assert_eq!(FitComponentType::Fdt.as_str(), "fdt");
        assert_eq!(FitComponentType::Ramdisk.as_str(), "ramdisk");
    }
}