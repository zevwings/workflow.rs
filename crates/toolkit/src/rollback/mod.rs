//! 回滚工具模块
//!
//! 本模块提供了更新失败时的回滚机制，包括：
//! - 备份当前版本的二进制文件和补全脚本
//! - 在更新失败时恢复备份的文件
//! - 清理备份文件
//! - Shell 配置重载

mod backup;
mod error;
mod helpers;
mod reload;
mod restore;

pub use backup::backup;
pub use error::RollbackError;
pub use helpers::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell,
    CompletionHelperError,
};
pub use reload::{reload_shell, ReloadError, ReloadResult};
pub use restore::{cleanup_backup, rollback};
