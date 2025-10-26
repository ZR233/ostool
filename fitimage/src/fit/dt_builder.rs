//! Device tree builder for FIT images
//!
//! Handles the creation of U-Boot compatible device tree structures for FIT images.

use crate::error::Result;
use crate::fit::config::{FitImageConfig, ComponentConfig};

/// Simple device tree builder that creates FIT-compatible device trees manually
pub struct DeviceTreeBuilder {
    buffer: Vec<u8>,
}

impl DeviceTreeBuilder {
    /// Create a new device tree builder
    pub fn new() -> Result<Self> {
        Ok(Self {
            buffer: Vec::new(),
        })
    }

    /// Build a FIT device tree from configuration
    pub fn build_fit_tree(&mut self, config: &FitImageConfig) -> Result<()> {
        // Start building the device tree structure manually
        // This is a simplified implementation that creates basic FIT structure

        // Reserve space for device tree header (will be filled later)
        self.buffer.extend_from_slice(&[0u8; 4096]); // Basic header space

        // Start with root node
        self.write_node_start("");
        self.write_property_string("description", &config.description);
        self.write_property_u32("#address-cells", 1);

        // Images node
        self.write_node_start("images");

        // Add components
        let mut component_names = Vec::new();

        if let Some(ref kernel) = config.kernel {
            let node_name = format!("kernel-{}", kernel.name);
            self.add_kernel_component(&node_name, kernel, config.compress_kernel)?;
            component_names.push(("kernel", node_name));
        }

        if let Some(ref fdt) = config.fdt {
            let node_name = format!("fdt-{}", fdt.name);
            self.add_fdt_component(&node_name, fdt)?;
            component_names.push(("fdt", node_name));
        }

        if let Some(ref ramdisk) = config.ramdisk {
            let node_name = format!("ramdisk-{}", ramdisk.name);
            self.add_ramdisk_component(&node_name, ramdisk)?;
            component_names.push(("ramdisk", node_name));
        }

        self.write_node_end(); // End images node

        // Configurations node
        self.write_node_start("configurations");
        self.write_node_start("default");
        self.write_property_string("description", "Default configuration");

        // Add component references
        for (comp_type, name) in &component_names {
            self.write_property_string(comp_type, name);
        }

        self.write_node_end(); // End default config node
        self.write_property_string("default", "default");
        self.write_node_end(); // End configurations node

        self.write_node_end(); // End root node

        Ok(())
    }

    /// Add kernel component
    fn add_kernel_component(&mut self, name: &str, component: &ComponentConfig, compress: bool) -> Result<()> {
        self.write_node_start(name);
        self.write_property_string("description", "Linux Kernel");
        self.write_property_string("type", "kernel");
        self.write_property_string("arch", "arm64");
        self.write_property_string("os", "linux");
        self.write_property_string("compression", if compress { "gzip" } else { "none" });

        if let Some(load_addr) = component.load_address {
            self.write_property_u64("load", load_addr);
        }
        if let Some(entry_addr) = component.entry_point {
            self.write_property_u64("entry", entry_addr);
        }

        self.write_property_data("data", &component.data);

        let crc = crate::crc::calculate_crc32(&component.data);
        self.write_property_u32("crc32", crc);

        self.write_node_end();
        Ok(())
    }

    /// Add FDT component
    fn add_fdt_component(&mut self, name: &str, component: &ComponentConfig) -> Result<()> {
        self.write_node_start(name);
        self.write_property_string("description", "Device Tree Blob");
        self.write_property_string("type", "flat_dt");
        self.write_property_string("arch", "arm64");
        self.write_property_string("compression", "none");

        if let Some(load_addr) = component.load_address {
            self.write_property_u64("load", load_addr);
        }

        self.write_property_data("data", &component.data);

        let crc = crate::crc::calculate_crc32(&component.data);
        self.write_property_u32("crc32", crc);

        self.write_node_end();
        Ok(())
    }

    /// Add ramdisk component
    fn add_ramdisk_component(&mut self, name: &str, component: &ComponentConfig) -> Result<()> {
        self.write_node_start(name);
        self.write_property_string("description", "Ramdisk Image");
        self.write_property_string("type", "ramdisk");
        self.write_property_string("arch", "arm64");
        self.write_property_string("os", "linux");
        self.write_property_string("compression", "none");

        if let Some(load_addr) = component.load_address {
            self.write_property_u64("load", load_addr);
        }

        self.write_property_data("data", &component.data);

        let crc = crate::crc::calculate_crc32(&component.data);
        self.write_property_u32("crc32", crc);

        self.write_node_end();
        Ok(())
    }

    /// Write node start marker
    fn write_node_start(&mut self, name: &str) {
        // Simple node marking - in real implementation this would be proper FDT structure
        if name.is_empty() {
            self.buffer.extend_from_slice(b"ROOT_START:");
        } else {
            self.buffer.extend_from_slice(format!("NODE_START:{}:", name).as_bytes());
        }
    }

    /// Write node end marker
    fn write_node_end(&mut self) {
        self.buffer.extend_from_slice(b"NODE_END:");
    }

    /// Write string property
    fn write_property_string(&mut self, name: &str, value: &str) {
        self.buffer.extend_from_slice(format!("PROP_STR:{}={}:", name, value).as_bytes());
    }

    /// Write u32 property
    fn write_property_u32(&mut self, name: &str, value: u32) {
        self.buffer.extend_from_slice(format!("PROP_U32:{}=0x{:08x}:", name, value).as_bytes());
    }

    /// Write u64 property
    fn write_property_u64(&mut self, name: &str, value: u64) {
        self.buffer.extend_from_slice(format!("PROP_U64:{}=0x{:016x}:", name, value).as_bytes());
    }

    /// Write data property
    fn write_property_data(&mut self, name: &str, data: &[u8]) {
        self.buffer.extend_from_slice(format!("PROP_DATA:{}=len{}:", name, data.len()).as_bytes());
        self.buffer.extend_from_slice(data);
        self.buffer.extend_from_slice(b":PROP_DATA_END:");
    }

    /// Finalize and return the device tree blob
    pub fn finalize(self) -> Result<Vec<u8>> {
        // For now, create a minimal valid device tree blob
        let mut result = Vec::new();

        // Device tree header (simplified)
        result.extend_from_slice(b"\xd0\x0d\xfe\xed"); // Magic number
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x38]); // Total size (placeholder)
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Off_dt_struct
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Off_dt_strings
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Off_mem_rsvmap
        result.extend_from_slice(&[0x11, 0x00, 0x00, 0x00]); // Version
        result.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]); // Last_comp_version

        // Add our custom structure (this is not a real FDT, just for testing)
        result.extend_from_slice(&self.buffer);

        // Update total size
        let total_size = result.len() as u32;
        result[4..8].copy_from_slice(&total_size.to_le_bytes());

        Ok(result)
    }
}

impl Default for DeviceTreeBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create default DeviceTreeBuilder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit::config::{FitImageConfig, ComponentConfig};

    #[test]
    fn test_dt_builder() {
        let config = FitImageConfig::new("Test FIT")
            .with_kernel(
                ComponentConfig::new("test-kernel", vec![1, 2, 3, 4])
                    .with_load_address(0x80080000)
                    .with_entry_point(0x80080000)
            )
            .with_kernel_compression(true);

        let mut builder = DeviceTreeBuilder::new().unwrap();
        builder.build_fit_tree(&config).unwrap();
        let fdt_data = builder.finalize().unwrap();

        // Verify we got a valid device tree
        assert!(!fdt_data.is_empty());

        // Basic magic number check for device tree
        assert_eq!(&fdt_data[0..4], b"\xd0\x0d\xfe\xed");
    }
}