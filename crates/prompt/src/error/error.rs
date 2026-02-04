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

/// Result 类型别名
pub type Result<T> = std::result::Result<T, PromptError>;

/// 根据错误消息检查是否是用户取消操作
///
/// # Examples
///
/// ```rust
/// use prompt::{PromptError, is_user_cancelled};
///
/// let msg = PromptError::Cancelled.to_string();
/// assert!(is_user_cancelled(&msg));
/// ```
pub fn is_user_cancelled(msg: &str) -> bool {
    let msg = msg.to_lowercase();
    msg.contains("user cancelled") || msg.contains("cancelled")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_error_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let prompt_error = PromptError::from(io_error);
        match prompt_error {
            PromptError::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_prompt_error_display() {
        let error = PromptError::Terminal("Test error".to_string());
        assert_eq!(error.to_string(), "Terminal error: Test error");
    }

    #[test]
    fn test_prompt_error_validation() {
        let error = PromptError::Validation("Invalid input".to_string());
        assert_eq!(error.to_string(), "Validation failed: Invalid input");
    }

    #[test]
    fn test_prompt_error_cancelled() {
        let error = PromptError::Cancelled;
        assert_eq!(error.to_string(), "User cancelled");
    }

    #[test]
    fn test_prompt_error_invalid_input() {
        let error = PromptError::InvalidInput("bad value".to_string());
        assert_eq!(error.to_string(), "Invalid input: bad value");
    }

    #[test]
    fn test_prompt_error_terminal_not_supported() {
        let error = PromptError::TerminalNotSupported;
        assert_eq!(error.to_string(), "Terminal not supported");
    }

    #[test]
    fn test_prompt_error_lock_poisoned() {
        let error = PromptError::LockPoisoned;
        assert_eq!(error.to_string(), "Lock error: mutex was poisoned");
    }

    #[test]
    fn test_prompt_error_is_error() {
        // 测试实现了 std::error::Error trait
        let error: Box<dyn std::error::Error> = Box::new(PromptError::Cancelled);
        assert_eq!(error.to_string(), "User cancelled");
    }
}
