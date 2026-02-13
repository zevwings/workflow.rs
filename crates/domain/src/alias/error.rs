//! 别名服务错误类型

use thiserror::Error;

/// 别名服务错误
#[derive(Error, Debug)]
pub enum AliasError {
    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("循环引用: {0}")]
    CircularReference(String),

    #[error("展开深度超限")]
    MaxDepthExceeded,

    #[error("配置操作失败: {0}")]
    Config(String),
}
