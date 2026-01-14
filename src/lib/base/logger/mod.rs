//! Logger 模块
//!
//! 本模块提供了日志相关的功能，包括：
//! - `LogLevel` - 日志级别管理
//! - `Tracer` - 结构化日志记录（用于 Lib 层）
//!
//! 注意：日志输出宏（success!, error! 等）已迁移到 `base::interactive::output::message` 模块

pub mod log_level;
pub mod tracing;

// 重新导出主要类型
pub use log_level::LogLevel;
pub use tracing::Tracer;
