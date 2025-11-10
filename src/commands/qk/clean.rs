use crate::{log_info, log_success, Logs};
use anyhow::{Context, Result};

/// 清理日志命令
#[allow(dead_code)]
pub struct CleanCommand;

impl CleanCommand {
    /// 清理指定 JIRA ID 的日志目录
    #[allow(dead_code)]
    pub fn clean(jira_id: &str, dry_run: bool, list_only: bool) -> Result<()> {
        if list_only {
            log_info!("Listing contents for {}...", jira_id);
        } else if dry_run {
            log_info!("[DRY RUN] Previewing clean operation for {}...", jira_id);
        } else {
            log_info!("Cleaning logs for {}...", jira_id);
        }

        // 调用库函数执行清理
        let deleted = Logs::clean_jira_dir(jira_id, dry_run, list_only)
            .context("Failed to clean logs directory")?;

        if deleted {
            log_success!("\nClean completed successfully!");
        } else if !dry_run && !list_only {
            log_info!("Clean operation was cancelled or directory does not exist.");
        }

        Ok(())
    }
}
