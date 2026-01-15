//! Logger 模块
//!
//! 提供结构化日志记录功能，基于 tracing crate。
//!
//! ## 模块组织
//!
//! - [`level`] - 日志级别定义
//! - [`config`] - 配置提供者接口
//! - [`subscriber`] - Tracing subscriber 配置实现
//! - [`path`] - 日志文件路径管理
//! - [`macros`] - 日志宏定义
//!
//! ## 使用示例
//!
//! ### 初始化
//!
//! 首先需要实现 [`ConfigProvider`] trait 来提供配置信息：
//!
//! ```rust,no_run
//! use workflow::core::logger::{self, ConfigProvider, LogLevel};
//! use color_eyre::Result;
//! use std::path::PathBuf;
//!
//! // 实现 ConfigProvider trait
//! struct MyConfigProvider;
//!
//! impl ConfigProvider for MyConfigProvider {
//!     fn log_level(&self) -> Option<LogLevel> {
//!         Some(LogLevel::Info)
//!     }
//!
//!     fn log_format(&self) -> Option<String> {
//!         Some("text".to_string())
//!     }
//!
//!     fn enable_console(&self) -> bool {
//!         true
//!     }
//!
//!     fn logs_dir(&self) -> Result<PathBuf> {
//!         Ok(PathBuf::from("/tmp/logs"))
//!     }
//! }
//!
//! # fn main() -> Result<()> {
//! let config = MyConfigProvider;
//! logger::init(Some("my-app"), &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 记录日志
//!
//! ```rust
//! use workflow::{log_debug, log_info, log_warn, log_error};
//!
//! log_info!("Operation completed");
//! let error = "some error";
//! log_error!("Operation failed: {}", error);
//! ```

pub mod config;
pub mod level;
pub mod macros;
pub(crate) mod path;
pub(crate) mod subscriber;

// 重新导出主要类型和函数
pub use config::ConfigProvider;
pub use level::LogLevel;
pub use subscriber::init;
