//! 回滚错误类型

use thiserror::Error;

/// 回滚错误类型
#[derive(Debug, Error)]
pub enum RollbackError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 路径错误
    #[error("Path error: {0}")]
    PathError(#[from] crate::PathError),

    /// 文件系统错误
    #[error("Filesystem error: {0}")]
    FsError(#[from] crate::util::FileError),

    /// 备份二进制文件失败
    #[error("Failed to backup binary file: {src} -> {dest}: {message}")]
    BackupBinaryFailed {
        src: String,
        dest: String,
        message: String,
    },

    /// 恢复二进制文件失败
    #[error("Failed to restore binary file: {src} -> {dest}: {message}")]
    RestoreBinaryFailed {
        src: String,
        dest: String,
        message: String,
    },

    /// 备份补全脚本失败
    #[error("Failed to backup completion script: {src} -> {dest}: {message}")]
    BackupCompletionFailed {
        src: String,
        dest: String,
        message: String,
    },

    /// 清理备份目录失败
    #[error("Failed to remove backup directory: {path}: {reason}")]
    CleanupFailed { path: String, reason: String },
}
