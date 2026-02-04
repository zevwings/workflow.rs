//! Completion 业务域
//!
//! 包含 Shell Completion 相关的实体和服务接口

pub mod entity;
pub mod service;

// Re-export public types
pub use entity::{
    get_all_completion_filenames, get_completion_filename, get_shell_source_path,
    CompletionCheckResult, CompletionGenerateResult, CompletionRemoveResult, ShellCompletionStatus,
    COMPLETIONS_FILE,
};
pub use service::CompletionService;
