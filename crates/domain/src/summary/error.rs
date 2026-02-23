//! 提交总结服务错误类型

use thiserror::Error;

use crate::git::GitError;

/// 提交总结分析错误
#[derive(Error, Debug)]
pub enum CommitSummaryError {
    #[error("The LLM call failed: {0}")]
    LLMError(String),

    #[error("The parsing failed: {0}")]
    ParseFailed(String),

    #[error("The serialization failed: {0}")]
    SerializeFailed(String),

    #[error("There are no changes to analyze: the base branch has no committed changes after the commit, and the staging area has no changes. Please commit or stage the changes first")]
    NoChangesToAnalyze,

    #[error("The Git operation failed: {0}")]
    Git(#[from] GitError),
}
