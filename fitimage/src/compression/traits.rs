//! 压缩接口定义
//!
//! 定义了所有压缩算法需要实现的标准接口

use crate::error::Result;

/// 压缩接口trait
/// 所有压缩算法都需要实现这个接口
pub trait CompressionInterface {
    /// 压缩数据
    ///
    /// # 参数
    /// - `data`: 要压缩的原始数据
    ///
    /// # 返回
    /// 压缩后的数据
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// 解压缩数据（主要用于验证）
    ///
    /// # 参数
    /// - `compressed_data`: 已压缩的数据
    ///
    /// # 返回
    /// 解压缩后的原始数据
    fn decompress(&self, compressed_data: &[u8]) -> Result<Vec<u8>>;

    /// 获取压缩算法名称
    fn get_name(&self) -> &'static str;
}