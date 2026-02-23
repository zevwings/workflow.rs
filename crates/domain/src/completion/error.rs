//! Shell Completion 服务错误类型

use thiserror::Error;
use toolkit::ShellError;

use crate::path::PathError;

/// Shell Completion 错误
#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("The input is invalid: {0}")]
    InvalidInput(String),

    #[error("The shell detection failed: {0}")]
    Shell(#[from] ShellError),

    #[error("The path error: {0}")]
    Path(#[from] PathError),
}
