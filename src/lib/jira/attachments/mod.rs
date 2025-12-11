//! Jira 附件下载模块
//!
//! 提供从 Jira 下载附件的功能，包括：
//! - 下载所有附件
//! - 下载日志附件（带重试逻辑）
//! - ZIP 文件处理（合并分片、解压）
//! - 清理附件目录
//!
//! ## 模块结构
//!
//! - `download` - 主下载器（协调各个组件）
//! - `filter` - 附件过滤逻辑
//! - `url_resolver` - URL 解析和重试策略
//! - `http_client` - HTTP 客户端适配器（利用 base::http）
//! - `directory` - 目录管理
//! - `zip` - ZIP 文件处理
//! - `clean` - 清理功能
//! - `constants` - 常量定义

mod clean;
mod constants;
mod directory;
mod download;
mod filter;
mod http_client;
mod paths;
mod url_resolver;
mod zip;

// 重新导出公共 API
pub use clean::{AttachmentCleaner, CleanResult, DirEntry, DirInfo};
pub use constants::*;
pub use download::{DownloadResult, JiraAttachmentDownloader, ProgressCallback};
pub use zip::ZipProcessor;
