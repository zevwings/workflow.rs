//! 校验和工具模块
//!
//! 提供文件校验和计算和验证功能，包括：
//! - 计算文件的 SHA256 哈希值
//! - 解析校验和文件内容
//! - 验证文件完整性
//! - 构建校验和 URL

#[allow(clippy::module_inception)]
pub mod checksum;

// 重新导出公共 API
pub use checksum::{Checksum, VerifyResult};
