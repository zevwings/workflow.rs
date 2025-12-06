//! 日志系统模块
//!
//! 本模块提供完整的日志系统功能，包括：
//! - 日志级别管理（LogLevel）
//! - 日志输出（Logger）
//! - Tracing 集成（RatatuiLayer, LogBuffer）

pub mod logger;
pub mod tracing;

// 重新导出主要类型
pub use logger::{LogLevel, Logger};
pub use tracing::{LogBuffer, LogEntry, RatatuiLayer};
