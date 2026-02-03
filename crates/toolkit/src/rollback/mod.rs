//! 回滚工具模块
//!
//! 本模块提供了更新失败时的回滚机制，包括：
//! - 备份当前版本的二进制文件和补全脚本
//! - 在更新失败时恢复备份的文件
//! - 清理备份文件
//! - Shell 配置重载

mod helpers;
mod manager;
mod reload;

pub use helpers::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell,
};
pub use manager::{BackupInfo, BackupResult, RollbackManager, RollbackResult};
pub use reload::{Reload, ReloadResult};
