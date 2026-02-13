//! Shell Completion 服务错误类型

use thiserror::Error;
use toolkit::ShellError;

use crate::path::PathError;

/// Shell Completion 错误
#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("Shell 检测失败: {0}")]
    Shell(#[from] ShellError),

    #[error("路径错误")]
    Path(#[from] PathError),
}
