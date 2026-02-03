//! Shell 模块错误类型

use std::path::PathBuf;

use thiserror::Error;

/// Shell 模块错误
#[derive(Debug, Error)]
pub enum ShellError {
    /// Shell 检测失败
    #[error("无法检测当前 Shell 类型")]
    DetectionFailed,

    /// 不支持的 Shell 类型
    #[error("不支持的 Shell 类型: {0}。支持的类型: zsh, bash, fish, powershell, elvish")]
    UnsupportedShell(String),

    /// 无法获取 Home 目录
    #[error("无法获取 Home 目录")]
    HomeNotFound,

    /// 配置文件不存在
    #[error("配置文件不存在: {0}")]
    ConfigFileNotFound(PathBuf),

    /// 配置文件读取失败
    #[error("读取配置文件失败: {path} - {source}")]
    ConfigReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// 配置文件写入失败
    #[error("写入配置文件失败: {path} - {source}")]
    ConfigWriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}
