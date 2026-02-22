//! SSH 错误类型

/// SSH 操作错误
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    /// ssh-agent 不可用
    #[error(
        "ssh-agent is not running. Start it with `eval $(ssh-agent)` or add to your shell profile."
    )]
    AgentUnavailable,

    /// 密钥文件不存在
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// 密钥文件已存在
    #[error("Key already exists at {0}. Use --force to overwrite.")]
    KeyAlreadyExists(String),

    /// 密钥生成失败
    #[error("Failed to generate key: {0}")]
    GenerationFailed(String),

    /// 添加密钥失败
    #[error("Failed to add key: {0}")]
    AddFailed(String),

    /// 移除密钥失败
    #[error("Failed to remove key: {0}")]
    RemoveFailed(String),

    /// 命令执行失败
    #[error("{0}")]
    CommandFailed(String),
}
