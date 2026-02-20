//! Shell 模块错误类型

use std::path::PathBuf;

use thiserror::Error;

/// Shell 模块错误
#[derive(Debug, Error)]
pub enum ShellError {
    /// Shell 检测失败
    #[error("Failed to detect current shell type")]
    DetectionFailed,

    /// 不支持的 Shell 类型
    #[error("Unsupported shell type: {0}. Supported types: zsh, bash, fish, powershell, elvish")]
    UnsupportedShell(String),

    /// 无法获取 Home 目录
    #[error("Failed to get home directory")]
    HomeNotFound,

    /// 配置文件不存在
    #[error("Config file not found: {0}")]
    ConfigFileNotFound(PathBuf),

    /// 配置文件读取失败
    #[error("Failed to read config file: {path} - {source}")]
    ConfigReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// 配置文件写入失败
    #[error("Failed to write config file: {path} - {source}")]
    ConfigWriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
