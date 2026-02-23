//! 文件系统操作工具
//!
//! 提供文件和目录操作的统一接口：
//! - `directory` - 目录管理工具函数
//! - `file` - 文件读写工具函数
//! - `archive` - 归档文件处理工具

pub mod archive;
pub mod directory;
mod error;
pub mod file;

// 重新导出错误类型
pub use error::FileError;
