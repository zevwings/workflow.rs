//! Completion 业务域
//!
//! 包含 Shell Completion 相关的实体和服务接口

pub mod entity;
pub mod error;
pub mod service;

// Re-export public types
pub use entity::{
    get_all_completion_filenames, get_completion_cache_shell_dir, get_completion_filename,
    get_completion_shell_dir, get_completion_shell_path, get_completion_source_shell_path,
    get_shell_source_path, CompletionCheckResult, CompletionGenerateResult, CompletionRemoveResult,
    ShellCompletionStatus,
};
pub use error::CompletionError;
pub use service::CompletionService;
