//! Logger 模块
//!
//! 本模块提供了日志相关的功能，包括：
//! - `LogLevel` - 日志级别管理
//! - `Logger` - 结构化日志记录（用于 Lib 层）
//! - `LogConfigProvider` - 日志配置提供者接口（定义在 `log_config` 模块中）
//!
//! 注意：日志输出宏（success!, error! 等）已迁移到 `interactive::output::message` 模块

pub mod log_config;
pub mod log_level;
pub mod log_macros;
#[allow(clippy::module_inception)]
pub mod logger;

// 重新导出主要类型
pub use log_config::LogConfigProvider;
pub use log_level::LogLevel;
pub use logger::Logger;
