//! 配置服务错误类型

use thiserror::Error;

use crate::path::PathError;

/// 配置服务错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("The TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("The path error: {0}")]
    Path(#[from] PathError),

    #[error("The lock failed: {0}")]
    LockFailed(String),

    #[error("The operation failed: {0}")]
    OperationFailed(String),
}
