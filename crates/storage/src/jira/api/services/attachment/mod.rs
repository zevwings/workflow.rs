//! Attachment 服务模块
//!
//! 提供完整的 Jira 附件下载功能，包括：
//! - 附件信息解析和 URL 重试策略
//! - 多线程并发下载
//! - 附件过滤（如日志文件筛选）
//! - 目录管理和清理
//!
//! ## 模块结构
//!
//! - `core.rs` - 核心服务实现（AttachmentService trait 和 AttachmentServiceImpl）
//! - `entity.rs` - URL 解析器（UrlResolver）
//! - `downloader.rs` - HTTP 下载器（HttpDownloader、ConcurrentDownloader）
//! - `filter.rs` - 附件过滤器（AttachmentFilter）
//! - `directory.rs` - 目录管理器（DirectoryManager）
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use crate::jira::api::services::attachment::{AttachmentService, AttachmentServiceImpl};
//!
//! let service = AttachmentServiceImpl::new(issue_service, config_context);
//! let result = service.download_attachments("PROJ-123", &base_dir)?;
//! println!("Downloaded {} files", result.downloaded_files.len());
//! ```

mod directory;
mod downloader;
mod entity;
mod service;

// 重新导出公共 API
pub use service::{AttachmentService, AttachmentServiceImpl};
