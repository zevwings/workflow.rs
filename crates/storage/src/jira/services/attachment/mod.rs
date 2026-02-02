//! Attachment 服务模块
//!
//! ## 状态：暂时禁用
//!
//! 此模块包含 Jira 附件下载功能的完整实现，目前暂时被注释禁用。
//!
//! ## 功能说明
//!
//! 此模块提供以下功能：
//! - 附件信息解析和 URL 处理
//! - 多线程并发下载
//! - ZIP 文件处理（合并、解压）
//! - 日志文件筛选和归档
//! - 下载进度回调
//! - 目录清理管理
//!
//! ## 模块结构
//!
//! - `entity.rs` - 附件相关实体定义（UrlResolver 等）
//! - `service.rs` - 附件服务实现（AttachmentServiceImpl）
//! - `downloader.rs` - 下载器实现（Downloader）
//!
//! ## 禁用原因
//!
//! 1. 当前版本暂不需要附件下载功能
//! 2. Repository 层已经返回"功能未实现"错误
//! 3. 保留代码以便将来需要时可以快速启用
//!
//! ## 启用方法
//!
//! 如需启用此功能：
//! 1. 取消注释本文件中的模块声明和导出
//! 2. 取消注释各子模块文件中的代码
//! 3. 更新 `JiraRepositoryImpl` 中的相关方法实现
//! 4. 确保所有依赖项已添加到 `Cargo.toml`（如 `walkdir`、`zip` 等）
//! 5. 运行 `cargo test` 确保功能正常
//!
//! ## 相关文档
//!
//! - 附件下载流程文档：[待补充]
//! - API 使用示例：参考注释代码中的实现
//!
//! ## 维护说明
//!
//! - 保持代码与最新的 Rust 版本兼容
//! - 定期检查依赖项更新
//! - 如果长期不使用（超过 6 个月），考虑移至独立的 feature flag

// 以下代码暂时禁用，保留以便将来使用

// mod downloader;
// mod entity;
// mod service;

// pub use downloader::Downloader;
// pub use service::AttachmentServiceImpl;
