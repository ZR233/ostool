//! FDT (Flattened Device Tree) Token definitions and utilities
//!
//! Implements standard FDT tokens according to the Device Tree specification.

use crate::error::Result;

/// Standard FDT tokens as defined in the Device Tree specification
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FdtToken {
    /// Begin a node - followed by node name (as string, NUL terminated)
    BeginNode = 0x1,
    /// End a node - no data
    EndNode = 0x2,
    /// Property - followed by property info, property name, and property value
    Prop = 0x3,
    /// Nop token - should be ignored
    Nop = 0x4,
    /// End of the structure block
    End = 0x9,
}

impl FdtToken {
    /// Get the u32 value for writing to FDT
    pub fn value(self) -> u32 {
        self as u32
    }

    /// Write token to buffer in big-endian format
    pub fn write_to_buffer(self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.value().to_be_bytes());
    }
}

/// FDT structure block alignment requirement (4 bytes)
pub const FDT_STRUCT_ALIGN: usize = 4;

/// Utility functions for FDT token operations
pub struct FdtTokenUtils;

impl FdtTokenUtils {
    /// Align length to 4-byte boundary as required by FDT spec
    pub fn align_to_4_bytes(length: usize) -> usize {
        (length + FDT_STRUCT_ALIGN - 1) & !(FDT_STRUCT_ALIGN - 1)
    }

    /// Pad buffer to 4-byte alignment
    pub fn pad_to_alignment(buffer: &mut Vec<u8>) {
        let padding_needed = FdtTokenUtils::align_to_4_bytes(buffer.len()) - buffer.len();
        buffer.resize(buffer.len() + padding_needed, 0);
    }

    /// Write string to buffer with NUL terminator and proper alignment
    pub fn write_string(buffer: &mut Vec<u8>, string: &str) -> Result<()> {
        // Write string bytes
        buffer.extend_from_slice(string.as_bytes());
        buffer.push(0); // NUL terminator

        // Align to 4-byte boundary
        FdtTokenUtils::pad_to_alignment(buffer);

        Ok(())
    }

    /// Write property data with proper alignment
    pub fn write_prop_data(buffer: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        buffer.extend_from_slice(data);
        FdtTokenUtils::pad_to_alignment(buffer);
        Ok(())
    }

    /// Write property header (length + name offset)
    pub fn write_prop_header(buffer: &mut Vec<u8>, length: u32, name_offset: u32) -> Result<()> {
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend_from_slice(&name_offset.to_be_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdt_token_values() {
        assert_eq!(FdtToken::BeginNode.value(), 0x1);
        assert_eq!(FdtToken::EndNode.value(), 0x2);
        assert_eq!(FdtToken::Prop.value(), 0x3);
        assert_eq!(FdtToken::Nop.value(), 0x4);
        assert_eq!(FdtToken::End.value(), 0x9);
    }

    #[test]
    fn test_token_writing() {
        let mut buffer = Vec::new();
        FdtToken::BeginNode.write_to_buffer(&mut buffer);
        FdtToken::EndNode.write_to_buffer(&mut buffer);

        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer[0..4], [0x00, 0x00, 0x00, 0x01]);
        assert_eq!(buffer[4..8], [0x00, 0x00, 0x00, 0x02]);
    }

    #[test]
    fn test_alignment() {
        assert_eq!(FdtTokenUtils::align_to_4_bytes(0), 0);
        assert_eq!(FdtTokenUtils::align_to_4_bytes(1), 4);
        assert_eq!(FdtTokenUtils::align_to_4_bytes(4), 4);
        assert_eq!(FdtTokenUtils::align_to_4_bytes(5), 8);
        assert_eq!(FdtTokenUtils::align_to_4_bytes(7), 8);
        assert_eq!(FdtTokenUtils::align_to_4_bytes(8), 8);
    }

    #[test]
    fn test_string_writing() {
        let mut buffer = Vec::new();
        FdtTokenUtils::write_string(&mut buffer, "test").unwrap();

        assert_eq!(buffer.len(), 8); // "test" + NUL + 3 padding bytes
        assert_eq!(buffer[0..4], *b"test\0");
        assert_eq!(buffer[4..8], [0, 0, 0]);
    }

    #[test]
    fn test_string_writing_alignment() {
        let mut buffer = Vec::new();
        buffer.push(1); // Start with 1 byte

        FdtTokenUtils::write_string(&mut buffer, "a").unwrap();

        // Should be aligned to 4-byte boundary after the string
        assert_eq!(buffer.len(), 8); // 1 + ("a" + NUL + 2 padding)
    }

    #[test]
    fn test_prop_data_writing() {
        let mut buffer = Vec::new();
        let data = vec![1, 2, 3, 4, 5];

        FdtTokenUtils::write_prop_data(&mut buffer, &data).unwrap();

        // 5 bytes data + 3 bytes padding = 8 bytes total
        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer[0..5], [1, 2, 3, 4, 5]);
        assert_eq!(buffer[5..8], [0, 0, 0]);
    }

    #[test]
    fn test_prop_header_writing() {
        let mut buffer = Vec::new();
        FdtTokenUtils::write_prop_header(&mut buffer, 100, 200).unwrap();

        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer[0..4], [0x00, 0x00, 0x00, 0x64]); // 100
        assert_eq!(buffer[4..8], [0x00, 0x00, 0x00, 0xC8]); // 200
    }
}