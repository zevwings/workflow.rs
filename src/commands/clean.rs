//! 清理日志命令
//!
//! 提供清理日志下载目录的功能，包括：
//! - 清理整个基础目录
//! - 预览操作（dry-run）
//! - 列出将要删除的内容

use crate::{log_break, log_info, log_success, Logs};
use anyhow::{Context, Result};

/// 清理命令
pub struct CleanCommand;

impl CleanCommand {
    /// 清理整个基础目录
    ///
    /// # 参数
    ///
    /// * `dry_run` - 如果为 true，只预览操作，不实际删除
    /// * `list_only` - 如果为 true，只列出将要删除的内容
    pub fn clean(dry_run: bool, list_only: bool) -> Result<()> {
        if list_only {
            log_info!("Listing contents of base directory...");
        } else if dry_run {
            log_info!("[DRY RUN] Previewing clean operation for base directory...");
        } else {
            log_info!("Cleaning base directory...");
        }

        // 调用库函数执行清理
        let base_dir = Logs::get_base_dir_path().context("Failed to get base directory path")?;
        let deleted = Logs::clean_dir(&base_dir, "the entire base directory", dry_run, list_only)
            .context("Failed to clean base directory")?;

        if deleted {
            log_break!();
            log_success!("Clean completed successfully!");
        } else if !dry_run && !list_only {
            log_info!("Clean operation was cancelled or directory does not exist.");
        }

        Ok(())
    }
}
