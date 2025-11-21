//! 清理日志命令
//!
//! 提供清理日志目录的功能，支持：
//! - 清理指定 JIRA ID 的日志目录（当 jira_id 不为空时）
//! - 清理整个基础目录（当 jira_id 为空字符串时）
//! - 预览操作（dry-run）
//! - 列出将要删除的内容（list-only）

use crate::{log_break, log_info, log_success, JiraLogs};
use anyhow::{Context, Result};

/// 清理日志命令
pub struct CleanCommand;

impl CleanCommand {
    /// 清理日志目录
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ID（如 "PROJ-123"）。如果为空字符串，则清理整个基础目录
    /// * `dry_run` - 如果为 true，只预览操作，不实际删除
    /// * `list_only` - 如果为 true，只列出将要删除的内容
    pub fn clean(jira_id: &str, dry_run: bool, list_only: bool) -> Result<()> {
        // 根据 jira_id 是否为空显示不同的日志消息
        if jira_id.is_empty() {
            if list_only {
                log_info!("Listing contents of base directory...");
            } else if dry_run {
                log_info!("[DRY RUN] Previewing clean operation for base directory...");
            } else {
                log_info!("Cleaning base directory...");
            }
        } else if list_only {
            log_info!("Listing contents for {}...", jira_id);
        } else if dry_run {
            log_info!("[DRY RUN] Previewing clean operation for {}...", jira_id);
        } else {
            log_info!("Cleaning logs for {}...", jira_id);
        }

        // 创建 JiraLogs 实例并执行清理
        let logs = JiraLogs::new().context("Failed to initialize JiraLogs")?;
        let deleted = logs
            .clean_dir(jira_id, dry_run, list_only)
            .context("Failed to clean logs directory")?;

        if deleted {
            log_break!();
            log_success!("Clean completed successfully!");
        } else if !dry_run && !list_only {
            log_info!("Clean operation was cancelled or directory does not exist.");
        }

        Ok(())
    }
}
