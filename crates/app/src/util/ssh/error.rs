use thiserror::Error;

/// 文件系统操作错误
#[derive(Debug, Error)]
pub enum SshOperationError {
    /// ssh-agent 不可用
    #[error("ssh-agent is not available")]
    AgentNotAvailable,

    /// 用户取消了操作
    #[error("operation cancelled by user")]
    OperationCancelled,

    /// 没有可用的 SSH 密钥
    #[error("no SSH keys available")]
    NoKeysAvailable,

    /// 操作失败
    #[error("operation failed: {0}")]
    OperationFailed(String),
}
