//! 日志初始化错误类型

use thiserror::Error;

/// 日志初始化错误
#[derive(Debug, Error)]
pub enum LoggerError {
    /// I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 日志初始化失败
    #[error("Logger initialization failed: {0}")]
    InitializationFailed(String),

    /// 创建日志目录失败
    #[error("Failed to create log directory: {0}")]
    CreateDirectoryFailed(String),
}
