//! Prompt 模块的错误类型
//!
//! 提供所有 prompt 相关功能的统一错误类型定义

use thiserror::Error;

/// Prompt 模块的错误类型
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
    /// 锁错误（Mutex 被毒化）
    #[error("Lock error: mutex was poisoned")]
    LockPoisoned,
}
