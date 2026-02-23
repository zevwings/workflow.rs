//! Commit 消息服务错误类型

use thiserror::Error;

use crate::git::GitError;

/// Commit 消息生成错误
#[derive(Error, Debug)]
pub enum CommitMessageError {
    #[error("The LLM call failed: {0}")]
    LLMError(String),

    #[error("The parsing failed: {0}")]
    ParseFailed(String),

    #[error("There are no changes to commit: {0}")]
    EmptyChanges(String),

    #[error("The Git operation failed: {0}")]
    Git(#[from] GitError),
}
