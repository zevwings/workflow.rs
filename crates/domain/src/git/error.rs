//! Git 错误类型

use thiserror::Error;

/// Git 操作错误
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git 操作失败: {0}")]
    OperationFailed(String),

    #[error("不是 Git 仓库")]
    NotGitRepo,

    #[error("仓库不存在: {0}")]
    RepositoryNotFound(String),

    #[error("分支不存在: {0}")]
    BranchNotFound(String),

    #[error("分支未完全合并: {0}")]
    BranchNotFullyMerged(String),

    #[error("提交不存在: {0}")]
    CommitNotFound(String),

    #[error("工作区有未提交的更改")]
    UncommittedChanges,

    #[error("合并冲突")]
    MergeConflict,

    #[error("Git 仓库已损坏: {0}\n\n建议修复步骤：\n  1. 运行 'git fsck' 检查仓库完整性\n  2. 如果可能，从远程仓库重新克隆\n  3. 或者尝试 'git fsck --full' 和 'git gc' 来修复")]
    RepositoryCorrupted(String),

    #[error("无效的引用: {0}")]
    InvalidReference(String),

    #[error("对象不存在: {0}")]
    ObjectNotFound(String),

    #[error("索引操作失败: {0}")]
    IndexError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("远程操作失败: {0}")]
    RemoteError(String),

    #[error("签名错误: {0}")]
    SignatureError(String),

    #[error("Hook 执行失败: {0}")]
    HookFailed(String),
}
