//! 对话框错误类型定义

use color_eyre::eyre;
use thiserror::Error;

/// Dialog 模块的错误类型
#[derive(Debug, Error)]
pub enum PromptError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// 终端错误
    #[error("Terminal error: {0}")]
    Terminal(String),
    /// 验证错误
    #[error("Validation failed: {0}")]
    Validation(String),
    /// 用户取消操作
    #[error("User cancelled")]
    Cancelled,
    /// 无效输入
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    /// 终端不支持
    #[error("Terminal not supported")]
    TerminalNotSupported,
}

/// Result 类型别名（使用 eyre::Result 以保持与代码库一致）
/// PromptError 会自动转换为 eyre::Report（因为实现了 std::error::Error）
pub type Result<T> = eyre::Result<T>;
