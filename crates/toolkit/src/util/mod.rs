//! Util 模块
//!
//! 本模块提供了各种工具函数和实用工具，包括：
//! - 基础工具（文件、目录、平台检测等）
//! - 浏览器操作
//! - 剪贴板操作（通过 `traits::clipboard_ext` 模块）
//! - 校验和计算（SHA256）
//!
//! ## 模块结构
//!
//! - `platform` - 平台检测工具（操作系统和架构检测）
//! - `traits/` - Trait 扩展模块（为现有类型提供扩展方法）
//! - `fs/` - 文件系统操作工具（目录和文件操作）
//! - `checksum` - 校验和工具（SHA256 计算和验证）
//!
//! ## 使用方式
//!
//! 所有工具都通过 `toolkit::{xx}` 访问：
//!
//! ```rust
//! use toolkit::{archive, calculate_sha256, Browser, ClipboardExt, Platform, Sensitive};
//! ```

pub mod checksum;
pub mod fs;
pub mod platform;
pub mod traits;

// Checksum 模块
pub use checksum::{
    build_checksum_url, calculate_sha256, parse_hash_from_content, verify as verify_checksum,
    verify_lenient as verify_checksum_lenient, ChecksumError, VerifyResult as ChecksumVerifyResult,
};

// FS 模块
pub use fs::FileError;

// Platform 模块
pub use platform::{Platform, PlatformError};

// Traits 模块 - 重新导出所有扩展 trait 和类型
pub use traits::{
    Browser, BrowserError, BrowserExt, ClipboardError, ClipboardExt, PathExt, Sensitive, SizeExt,
    Truncate,
};
