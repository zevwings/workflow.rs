//! 提交总结服务错误类型

use crate::git::GitError;
use thiserror::Error;

/// 提交总结分析错误
#[derive(Error, Debug)]
pub enum CommitSummaryError {
    #[error("LLM 调用失败: {0}")]
    LLMError(String),

    #[error("解析失败: {0}")]
    ParseFailed(String),

    #[error("序列化失败: {0}")]
    SerializeFailed(String),

    #[error("无变更可分析：基准分支之后无已提交变更，且暂存区无变更。请先提交或暂存变更")]
    NoChangesToAnalyze,

    #[error("Git 操作失败")]
    Git(#[from] GitError),
}
