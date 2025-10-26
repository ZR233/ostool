//! Gzip压缩实现
//!
//! 使用flate2库提供gzip压缩和解压缩功能

use std::io::{Read, Write};

use crate::compression::traits::CompressionInterface;
use crate::error::Result;
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzipLevel};

/// Gzip压缩器
/// 支持可配置的压缩级别
pub struct GzipCompressor {
    /// 压缩级别 (0-9, 0表示无压缩)
    level: u8,
    /// 是否启用压缩（false时直接复制数据）
    enabled: bool,
}

impl Default for GzipCompressor {
    fn default() -> Self {
        Self::new(6) //
    }
}

impl GzipCompressor {
    /// 创建指定压缩级别的gzip压缩器
    pub fn new(level: u8) -> Self {
        Self {
            level: level.clamp(0, 9), // 限制在0-9范围内
            enabled: level > 0,       // 级别0表示不压缩
        }
    }

    /// 创建禁用压缩的实例
    pub fn new_disabled() -> Self {
        Self {
            level: 0,
            enabled: false,
        }
    }

    /// 获取flate2的压缩级别
    fn get_compression_level(&self) -> GzipLevel {
        if !self.enabled {
            GzipLevel::none()
        } else {
            match self.level {
                0 => GzipLevel::none(),
                1 => GzipLevel::fast(),
                9 => GzipLevel::best(),
                _ => GzipLevel::default(),
            }
        }
    }
}

impl CompressionInterface for GzipCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.enabled {
            // 如果禁用压缩，直接返回数据副本
            return Ok(data.to_vec());
        }

        let mut encoder = GzEncoder::new(Vec::new(), self.get_compression_level());

        encoder.write_all(data).map_err(|e| {
            crate::error::MkImageError::compression_error(format!("Gzip compression failed: {}", e))
        })?;

        encoder.finish().map_err(|e| {
            crate::error::MkImageError::compression_error(format!("Gzip finish failed: {}", e))
        })
    }

    fn decompress(&self, compressed_data: &[u8]) -> Result<Vec<u8>> {
        if !self.enabled {
            // 如果没有压缩，直接返回数据副本
            return Ok(compressed_data.to_vec());
        }

        let mut decoder = GzDecoder::new(compressed_data);
        let mut buffer = Vec::new();

        decoder.read_to_end(&mut buffer).map_err(|e| {
            crate::error::MkImageError::compression_error(format!(
                "Gzip decompression failed: {}",
                e
            ))
        })?;

        Ok(buffer)
    }

    fn get_name(&self) -> &'static str {
        if self.enabled {
            "gzip"
        } else {
            "none"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_compression() {
        let compressor = GzipCompressor::new(6);
        // 使用更大的数据以确保压缩效果
        let original_data = "Hello, World! This is a test string for gzip compression. ".repeat(10);
        let original_bytes = original_data.as_bytes();

        // 测试压缩
        let compressed = compressor
            .compress(original_bytes)
            .expect("Compression should succeed");
        assert!(
            compressed.len() < original_bytes.len(),
            "Compressed data should be smaller"
        );

        // 测试解压缩
        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompression should succeed");
        assert_eq!(
            decompressed, original_bytes,
            "Decompressed data should match original"
        );
    }

    #[test]
    fn test_disabled_compression() {
        let compressor = GzipCompressor::new_disabled();
        let original_data = b"Hello, World!";

        // 禁用压缩时应该返回原数据
        let compressed = compressor
            .compress(original_data)
            .expect("Compression should succeed");
        assert_eq!(
            compressed, original_data,
            "Disabled compression should return original data"
        );

        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompression should succeed");
        assert_eq!(
            decompressed, original_data,
            "Decompressed data should match original"
        );
    }

    #[test]
    fn test_compressor_name() {
        let enabled_compressor = GzipCompressor::new(6);
        assert_eq!(enabled_compressor.get_name(), "gzip");

        let disabled_compressor = GzipCompressor::new_disabled();
        assert_eq!(disabled_compressor.get_name(), "none");
    }
}
