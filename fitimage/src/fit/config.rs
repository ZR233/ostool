//! FIT image configuration structures
//!
//! Defines the configuration structures used to build FIT images.

use serde::{Deserialize, Serialize};

/// Configuration for building a FIT image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitImageConfig {
    /// Description of the FIT image
    pub description: String,

    /// Kernel component configuration
    pub kernel: Option<ComponentConfig>,

    /// Device tree component configuration
    pub fdt: Option<ComponentConfig>,

    /// Ramdisk component configuration
    pub ramdisk: Option<ComponentConfig>,

    /// Whether to compress the kernel with gzip
    pub compress_kernel: bool,

    /// Default configuration name
    pub default_config: Option<String>,

    /// Configurations mapping (name -> (description, kernel, fdt, ramdisk))
    pub configurations: std::collections::HashMap<String, (String, Option<String>, Option<String>, Option<String>)>,
}

/// Configuration for a single component (kernel, fdt, ramdisk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Name of the component (used as node name in device tree)
    pub name: String,

    /// Raw data of the component
    pub data: Vec<u8>,

    /// Description of the component
    pub description: Option<String>,

    /// Component type (kernel, flat_dt, ramdisk, etc.)
    pub component_type: Option<String>,

    /// Architecture (arm, arm64, etc.)
    pub arch: Option<String>,

    /// OS type (linux, etc.)
    pub os: Option<String>,

    /// Compression type (none, gzip, etc.)
    pub compression: Option<String>,

    /// Load address in memory
    pub load_address: Option<u64>,

    /// Entry point address (for kernel)
    pub entry_point: Option<u64>,
}

impl ComponentConfig {
    /// Create a new component configuration
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            description: None,
            component_type: None,
            arch: None,
            os: None,
            compression: None,
            load_address: None,
            entry_point: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set component type
    pub fn with_type(mut self, component_type: impl Into<String>) -> Self {
        self.component_type = Some(component_type.into());
        self
    }

    /// Set architecture
    pub fn with_arch(mut self, arch: impl Into<String>) -> Self {
        self.arch = Some(arch.into());
        self
    }

    /// Set OS type
    pub fn with_os(mut self, os: impl Into<String>) -> Self {
        self.os = Some(os.into());
        self
    }

    /// Set compression type
    pub fn with_compression(mut self, compression: impl Into<String>) -> Self {
        self.compression = Some(compression.into());
        self
    }

    /// Set load address
    pub fn with_load_address(mut self, load_address: u64) -> Self {
        self.load_address = Some(load_address);
        self
    }

    /// Set entry point address
    pub fn with_entry_point(mut self, entry_point: u64) -> Self {
        self.entry_point = Some(entry_point);
        self
    }
}

impl FitImageConfig {
    /// Create a new FIT image configuration
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            kernel: None,
            fdt: None,
            ramdisk: None,
            compress_kernel: false,
            default_config: None,
            configurations: std::collections::HashMap::new(),
        }
    }

    /// Set kernel component
    pub fn with_kernel(mut self, kernel: ComponentConfig) -> Self {
        self.kernel = Some(kernel);
        self
    }

    /// Set FDT component
    pub fn with_fdt(mut self, fdt: ComponentConfig) -> Self {
        self.fdt = Some(fdt);
        self
    }

    /// Set ramdisk component
    pub fn with_ramdisk(mut self, ramdisk: ComponentConfig) -> Self {
        self.ramdisk = Some(ramdisk);
        self
    }

    /// Enable kernel compression
    pub fn with_kernel_compression(mut self, compress: bool) -> Self {
        self.compress_kernel = compress;
        self
    }

    /// Set default configuration
    pub fn with_default_config(mut self, default: impl Into<String>) -> Self {
        self.default_config = Some(default.into());
        self
    }

    /// Add a configuration
    pub fn with_configuration(mut self, name: impl Into<String>, description: impl Into<String>,
                             kernel: Option<impl Into<String>>, fdt: Option<impl Into<String>>,
                             ramdisk: Option<impl Into<String>>) -> Self {
        self.configurations.insert(name.into(), (description.into(),
                                                  kernel.map(Into::into),
                                                  fdt.map(Into::into),
                                                  ramdisk.map(Into::into)));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = FitImageConfig::new("Test FIT")
            .with_kernel(ComponentConfig::new("kernel", vec![1, 2, 3]))
            .with_fdt(ComponentConfig::new("fdt", vec![4, 5, 6]))
            .with_kernel_compression(true);

        assert_eq!(config.description, "Test FIT");
        assert!(config.kernel.is_some());
        assert!(config.fdt.is_some());
        assert!(config.ramdisk.is_none());
        assert!(config.compress_kernel);
    }

    #[test]
    fn test_component_config() {
        let component = ComponentConfig::new("test", vec![1, 2, 3])
            .with_description("Test component")
            .with_type("kernel")
            .with_arch("arm64")
            .with_os("linux")
            .with_compression("none")
            .with_load_address(0x80000)
            .with_entry_point(0x80000);

        assert_eq!(component.name, "test");
        assert_eq!(component.data, vec![1, 2, 3]);
        assert_eq!(component.description, Some("Test component".to_string()));
        assert_eq!(component.component_type, Some("kernel".to_string()));
        assert_eq!(component.arch, Some("arm64".to_string()));
        assert_eq!(component.os, Some("linux".to_string()));
        assert_eq!(component.compression, Some("none".to_string()));
        assert_eq!(component.load_address, Some(0x80000));
        assert_eq!(component.entry_point, Some(0x80000));
    }

    #[test]
    fn test_fit_image_config_with_configurations() {
        let config = FitImageConfig::new("Test FIT")
            .with_kernel(ComponentConfig::new("kernel", vec![1, 2, 3]))
            .with_fdt(ComponentConfig::new("fdt", vec![4, 5, 6]))
            .with_default_config("default")
            .with_configuration("default", "Default configuration",
                               Some("kernel"), Some("fdt"), None::<String>);

        assert_eq!(config.description, "Test FIT");
        assert!(config.kernel.is_some());
        assert!(config.fdt.is_some());
        assert_eq!(config.default_config, Some("default".to_string()));
        assert!(config.configurations.contains_key("default"));
    }
}
