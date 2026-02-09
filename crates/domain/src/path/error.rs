//! 路径操作错误类型

use thiserror::Error;

/// 路径操作错误
#[derive(Debug, Error)]
pub enum PathError {
    /// 无法确定 home 目录
    #[error("Cannot determine {0} directory")]
    CannotDetermine(String),

    /// I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 环境变量错误
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    /// 路径展开错误
    #[error("Path expansion error: {0}")]
    Expansion(String),

    /// 路径不存在
    #[error("Path does not exist: {0}")]
    NotFound(String),

    /// 权限错误
    #[error("Permission error: {0}")]
    Permission(String),

    /// 通用错误
    #[error("{0}")]
    Other(String),
}
