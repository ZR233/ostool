pub mod compression;
pub mod crc;
pub mod error;
pub mod fit;
pub mod hash;

// Re-export main types for convenience
pub use compression::traits::CompressionInterface;
pub use crc::calculate_crc32;
pub use error::{MkImageError, Result};
pub use fit::{ComponentConfig, FitImageBuilder, FitImageConfig};
pub use hash::{calculate_hashes, default_hash_algorithms, HashAlgorithm, HashResult};

/// Current version of the mkimage implementation
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// FIT image magic number
pub const FIT_MAGIC: &[u8] = b"FIT";
