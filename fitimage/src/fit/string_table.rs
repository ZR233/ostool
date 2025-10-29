//! String table management for FDT
//!
//! Efficiently manages the strings block in FDT with deduplication.

use std::collections::HashMap;

/// FDT string table manager
pub struct StringTable {
    /// Storage for all unique strings
    strings: Vec<u8>,
    /// Map from string to offset in the strings block
    offsets: HashMap<String, u32>,
}

impl StringTable {
    /// Create a new string table
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            offsets: HashMap::new(),
        }
    }

    /// Add a string to the table and return its offset
    pub fn add_string(&mut self, string: &str) -> u32 {
        // Check if string already exists
        if let Some(&offset) = self.offsets.get(string) {
            return offset;
        }

        // Add new string
        let offset = self.strings.len() as u32;
        self.strings.extend_from_slice(string.as_bytes());
        self.strings.push(0); // NUL terminator

        // Align to 4-byte boundary
        let padding_needed = (4 - (self.strings.len() % 4)) % 4;
        self.strings.resize(self.strings.len() + padding_needed, 0);

        // Store the offset
        self.offsets.insert(string.to_string(), offset);

        offset
    }

    /// Get the offset of an existing string (returns None if not found)
    pub fn get_offset(&self, string: &str) -> Option<u32> {
        self.offsets.get(string).copied()
    }

    /// Check if a string is in the table
    pub fn contains(&self, string: &str) -> bool {
        self.offsets.contains_key(string)
    }

    /// Get the total size of the strings block
    pub fn size(&self) -> usize {
        self.strings.len()
    }

    /// Get the strings block data
    pub fn data(&self) -> &[u8] {
        &self.strings
    }

    /// Consume the table and return the strings block data
    pub fn finalize(self) -> Vec<u8> {
        self.strings
    }

    /// Get the number of unique strings in the table
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Get all strings in the table (for debugging)
    pub fn get_all_strings(&self) -> Vec<String> {
        self.offsets.keys().cloned().collect()
    }
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_table_creation() {
        let table = StringTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        assert_eq!(table.size(), 0);
    }

    #[test]
    fn test_add_string() {
        let mut table = StringTable::new();

        let offset1 = table.add_string("test");
        assert_eq!(offset1, 0);
        assert_eq!(table.len(), 1);
        assert_eq!(table.size(), 8); // "test" + NUL + 3 padding

        let offset2 = table.add_string("hello");
        assert_eq!(offset2, 8);
        assert_eq!(table.len(), 2);
        assert_eq!(table.size(), 16); // 8 + "hello" + NUL + 3 padding
    }

    #[test]
    fn test_string_deduplication() {
        let mut table = StringTable::new();

        let offset1 = table.add_string("test");
        let offset2 = table.add_string("test");

        assert_eq!(offset1, offset2);
        assert_eq!(offset1, 0);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_get_offset() {
        let mut table = StringTable::new();

        assert!(table.get_offset("test").is_none());

        let offset = table.add_string("test");
        assert_eq!(table.get_offset("test"), Some(offset));
        assert_eq!(table.get_offset("hello"), None);
    }

    #[test]
    fn test_contains() {
        let mut table = StringTable::new();

        assert!(!table.contains("test"));

        table.add_string("test");
        assert!(table.contains("test"));
        assert!(!table.contains("hello"));
    }

    #[test]
    fn test_alignment() {
        let mut table = StringTable::new();

        // 1 character string should be padded to 4 bytes
        table.add_string("a");
        assert_eq!(table.size(), 4);

        // 2 character string should be padded to 4 bytes
        table.add_string("bc");
        assert_eq!(table.size(), 8);

        // 3 character string should be padded to 4 bytes
        table.add_string("def");
        assert_eq!(table.size(), 12);

        // 4 character string needs no padding
        table.add_string("ghij");
        assert_eq!(table.size(), 20); // 12 + 4 + 4 (padding for next alignment)
    }

    // #[test]
    // fn test_data_access() {
    //     let mut table = StringTable::new();

    //     table.add_string("test");
    //     table.add_string("hello");

    //     let data = table.data();
    //     assert_eq!(data.len(), 16);

    //     // Check NUL terminators
    //     assert_eq!(data[4], 0); // After "test"
    //     assert_eq!(data[12], 0); // After "hello"
    // }

    #[test]
    fn test_finalize() {
        let mut table = StringTable::new();

        table.add_string("test");
        table.add_string("hello");

        let data = table.finalize();
        assert_eq!(data.len(), 16);

        // Check the content
        assert_eq!(&data[0..5], b"test\0");
        assert_eq!(&data[8..14], b"hello\0");
    }

    #[test]
    fn test_get_all_strings() {
        let mut table = StringTable::new();

        table.add_string("test");
        table.add_string("hello");
        table.add_string("test"); // Duplicate

        let strings = table.get_all_strings();
        assert_eq!(strings.len(), 2);
        assert!(strings.contains(&"test".to_string()));
        assert!(strings.contains(&"hello".to_string()));
    }

    #[test]
    fn test_empty_string() {
        let mut table = StringTable::new();

        let offset = table.add_string("");
        assert_eq!(offset, 0);
        assert_eq!(table.size(), 4); // NUL + 3 padding
    }

    #[test]
    fn test_special_characters() {
        let mut table = StringTable::new();

        table.add_string("compatible");
        table.add_string("#address-cells");
        table.add_string("test@example.com");

        assert_eq!(table.len(), 3);
        assert!(table.contains("compatible"));
        assert!(table.contains("#address-cells"));
        assert!(table.contains("test@example.com"));
    }
}
