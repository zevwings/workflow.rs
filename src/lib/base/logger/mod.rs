//! Logger 模块
//!
//! 本模块提供了日志相关的功能，包括：
//! - `LogLevel` - 日志级别管理
//! - `Logger` - 用户友好的控制台日志输出（用于 Commands 层）
//! - `Tracer` - 结构化日志记录（用于 Lib 层）

pub mod console;
pub mod log_level;
pub mod tracing;

// 重新导出主要类型
pub use console::Logger;
pub use log_level::LogLevel;
pub use tracing::Tracer;
