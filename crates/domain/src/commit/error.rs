//! Commit 消息服务错误类型

use thiserror::Error;

use crate::git::GitError;

/// Commit 消息生成错误
#[derive(Error, Debug)]
pub enum CommitMessageError {
    #[error("LLM 调用失败: {0}")]
    LLMError(String),

    #[error("解析失败: {0}")]
    ParseFailed(String),

    #[error("无变更可提交: {0}")]
    EmptyChanges(String),

    #[error("Git 操作失败")]
    Git(#[from] GitError),
}
