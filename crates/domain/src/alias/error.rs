//! 别名服务错误类型

use thiserror::Error;

/// 别名服务错误
#[derive(Error, Debug)]
pub enum AliasError {
    #[error("The input is invalid: {0}")]
    InvalidInput(String),

    #[error("The circular reference: {0}")]
    CircularReference(String),

    #[error("The expansion depth exceeded")]
    MaxDepthExceeded,

    #[error("The configuration operation failed: {0}")]
    Config(String),
}
