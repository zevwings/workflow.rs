//! 配置服务错误类型

use thiserror::Error;

use crate::path::PathError;

/// 配置服务错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("路径错误")]
    Path(#[from] PathError),

    #[error("获取锁失败: {0}")]
    LockFailed(String),

    #[error("操作失败: {0}")]
    OperationFailed(String),
}
