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
//! ```rust
//! use workflow::core::logger;
//! use workflow::infra::adapters::config::SettingsAdapter;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = SettingsAdapter::new();
//! logger::init(Some("pr-create"), &config)?;
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
