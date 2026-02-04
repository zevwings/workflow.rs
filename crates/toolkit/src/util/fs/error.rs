//! 文件系统操作错误类型

use thiserror::Error;

/// 文件系统操作错误
#[derive(Debug, Error)]
pub enum FileError {
    /// I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 解析错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML 解析错误
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// 压缩/解压错误
    #[error("Compression error: {0}")]
    Compression(String),

    /// 文件不存在
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// 通用错误
    #[error("{0}")]
    Other(String),
}
