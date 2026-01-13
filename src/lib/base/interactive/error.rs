//! 错误类型定义

use color_eyre::eyre;

/// Prompt 模块的错误类型
#[derive(Debug)]
pub enum PromptError {
    /// IO 错误
    Io(std::io::Error),
    /// 终端错误
    Terminal(String),
    /// 验证错误
    Validation(String),
    /// 用户取消操作
    Cancelled,
    /// 无效输入
    InvalidInput(String),
    /// 终端不支持
    TerminalNotSupported,
}

impl std::fmt::Display for PromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptError::Io(e) => write!(f, "IO error: {}", e),
            PromptError::Terminal(msg) => write!(f, "Terminal error: {}", msg),
            PromptError::Validation(msg) => write!(f, "Validation failed: {}", msg),
            PromptError::Cancelled => write!(f, "User cancelled"),
            PromptError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            PromptError::TerminalNotSupported => write!(f, "Terminal not supported"),
        }
    }
}

impl std::error::Error for PromptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PromptError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PromptError {
    fn from(err: std::io::Error) -> Self {
        PromptError::Io(err)
    }
}

/// Result 类型别名（使用 eyre::Result 以保持与代码库一致）
/// PromptError 会自动转换为 eyre::Report（因为实现了 std::error::Error）
pub type Result<T> = eyre::Result<T>;
