//! Git 错误类型

use thiserror::Error;

/// Git 操作错误
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git operation failed: {0}")]
    OperationFailed(String),

    #[error("The current directory is not a Git repository")]
    NotGitRepo,

    #[error("The repository at {0} is not found")]
    RepositoryNotFound(String),

    #[error("The branch {0} is not found")]
    BranchNotFound(String),

    #[error("The branch {0} is not fully merged")]
    BranchNotFullyMerged(String),

    #[error("The commit {0} is not found")]
    CommitNotFound(String),

    #[error("There are uncommitted changes in the working directory")]
    UncommittedChanges,

    #[error("There are merge conflicts")]
    MergeConflict,

    #[error("The Git repository at {0} is corrupted\n\nSuggested repair steps:\n  1. Run 'git fsck' to check repository integrity\n  2. If possible, re-clone from remote repository\n  3. Or try 'git fsck --full' and 'git gc' to repair")]
    RepositoryCorrupted(String),

    #[error("The reference {0} is invalid")]
    InvalidReference(String),

    #[error("The object {0} is not found")]
    ObjectNotFound(String),

    #[error("The index operation failed: {0}")]
    IndexError(String),

    #[error("The configuration is invalid: {0}")]
    ConfigError(String),

    #[error("The remote operation failed: {0}")]
    RemoteError(String),

    #[error("The signature is invalid: {0}")]
    SignatureError(String),

    #[error("The hook execution failed: {0}")]
    HookFailed(String),
}
