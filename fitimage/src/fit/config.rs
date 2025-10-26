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
}

/// Configuration for a single component (kernel, fdt, ramdisk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Name of the component (used as node name in device tree)
    pub name: String,

    /// Raw data of the component
    pub data: Vec<u8>,

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
            load_address: None,
            entry_point: None,
        }
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
            .with_load_address(0x80000)
            .with_entry_point(0x80000);

        assert_eq!(component.name, "test");
        assert_eq!(component.data, vec![1, 2, 3]);
        assert_eq!(component.load_address, Some(0x80000));
        assert_eq!(component.entry_point, Some(0x80000));
    }
}
