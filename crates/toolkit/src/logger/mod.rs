//! Logger 模块
//!
//! 提供结构化日志记录功能，基于 tracing crate。
//!
//! ## 模块组织
//!
//! - [`config`] - 日志配置结构体
//! - [`subscriber`] - Tracing subscriber 配置实现
//! - [`path`] - 日志文件路径管理
//! - [`macros`] - 日志宏定义
//!
//! ## 使用示例
//!
//! ### 初始化
//!
//! 使用 [`LoggerConfig`] 结构体来配置日志：
//!
//! ```rust,no_run
//! use toolkit::logger::{self, LoggerConfig};
//! use toolkit::logger::LoggerError;
//! use std::path::PathBuf;
//!
//! # fn main() -> std::result::Result<(), LoggerError> {
//! let config = LoggerConfig::new(
//!     Some("info".to_string()),  // 日志级别
//!     Some("text".to_string()),   // 日志格式
//!     true,                        // 启用控制台输出
//!     PathBuf::from("/tmp/logs"),  // 日志目录
//! );
//! logger::init(Some("my-app"), &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 记录日志
//!
//! ```rust
//! use toolkit::{log_debug, log_info, log_warn, log_error};
//!
//! log_info!("Operation completed");
//! let error = "some error";
//! log_error!("Operation failed: {}", error);
//! ```

pub mod config;
mod error;
pub mod macros;
pub(crate) mod path;
pub(crate) mod subscriber;

// 重新导出主要类型和函数
pub use config::LoggerConfig;
pub use error::LoggerError;
pub use subscriber::init;
