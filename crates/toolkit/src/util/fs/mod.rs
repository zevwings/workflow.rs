//! 文件系统操作工具
//!
//! 提供文件和目录操作的统一接口：
//! - `directory` - 目录管理工具（DirectoryWalker）
//! - `file` - 文件读写工具（FileReader, FileWriter）
//! - `archive` - 归档文件处理工具（Archive）

pub mod archive;
pub mod directory;
mod error;
pub mod file;

// 重新导出主要类型
pub use archive::Archive;
pub use directory::DirectoryWalker;
pub use error::FsError;
pub use file::{FileReader, FileWriter};
