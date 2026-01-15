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
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Terminal error"));
        assert!(error_msg.contains("Test error"));
    }

    #[test]
    fn test_prompt_error_validation() {
        let error = PromptError::Validation("Invalid input".to_string());
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Validation failed"));
        assert!(error_msg.contains("Invalid input"));
    }

    #[test]
    fn test_prompt_error_cancelled() {
        let error = PromptError::Cancelled;
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "User cancelled");
    }

    #[test]
    fn test_prompt_error_invalid_input() {
        let error = PromptError::InvalidInput("bad value".to_string());
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Invalid input"));
        assert!(error_msg.contains("bad value"));
    }

    #[test]
    fn test_prompt_error_terminal_not_supported() {
        let error = PromptError::TerminalNotSupported;
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "Terminal not supported");
    }

    #[test]
    fn test_prompt_error_lock_poisoned() {
        let error = PromptError::LockPoisoned;
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "Lock error: mutex was poisoned");
    }

    #[test]
    fn test_prompt_error_is_error() {
        // 测试实现了 std::error::Error trait
        let error: Box<dyn std::error::Error> = Box::new(PromptError::Cancelled);
        let error_msg = format!("{}", error);
        assert_eq!(error_msg, "User cancelled");
    }
}
