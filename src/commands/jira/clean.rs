//! 清理日志命令
//!
//! 提供清理日志目录的功能，支持：
//! - 清理指定 JIRA ID 的日志目录（当提供 jira_id 时）
//! - 清理整个基础目录（当指定 --all 标志或交互式输入时留空）
//! - 预览操作（dry-run）
//! - 列出将要删除的内容（list-only）

use crate::base::util::dialog::InputDialog;
use crate::jira::logs::JiraLogs;
use crate::{log_break, log_info, log_success};
use anyhow::{Context, Result};

/// 清理日志命令
pub struct CleanCommand;

impl CleanCommand {
    /// 清理日志目录
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ID（如 "PROJ-123"）。如果为 None，会交互式输入；如果为空字符串，会报错（应使用 --all 或省略参数）
    /// * `all` - 如果为 true，清理整个基础目录（忽略 jira_id）
    /// * `dry_run` - 如果为 true，只预览操作，不实际删除
    /// * `list_only` - 如果为 true，只列出将要删除的内容
    pub fn clean(jira_id: Option<String>, all: bool, dry_run: bool, list_only: bool) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if all {
            // 如果指定了 --all，直接使用空字符串表示清理全部
            String::new()
        } else if let Some(id) = jira_id {
            // 如果提供了参数，验证是否为空字符串
            let trimmed = id.trim();
            if trimmed.is_empty() {
                anyhow::bail!(
                    "Empty JIRA ID is not allowed. Use '--all' flag or omit the argument to enter interactive mode."
                );
            }
            trimmed.to_string()
        } else {
            // 交互式输入：允许用户输入 JIRA ID，留空表示清理全部
            InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123, or leave empty to clean all)")
                .allow_empty(true)
                .prompt()
                .context("Failed to read Jira ticket ID")?
                .trim()
                .to_string()
        };

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
            .clean_dir(&jira_id, dry_run, list_only)
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
