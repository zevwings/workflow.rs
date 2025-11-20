//! 清理功能相关实现

use anyhow::{Context, Result};
use std::path::Path;

use crate::{confirm, log_break, log_info, log_success};
use super::helpers;

use super::JiraLogs;

impl JiraLogs {
    /// 清理指定 JIRA ID 的日志目录
    ///
    /// 自动构建目录路径，然后清理该目录。
    pub fn clean_dir(&self, jira_id: &str, dry_run: bool, list_only: bool) -> Result<bool> {
        let dir = if jira_id.is_empty() {
            // 如果 jira_id 为空，清理整个基础目录
            self.base_dir.clone()
        } else {
            self.base_dir.join(jira_id)
        };
        let dir_name = if jira_id.is_empty() {
            "the entire base directory".to_string()
        } else {
            format!("the directory for {}", jira_id)
        };

        if !dir.exists() {
            log_info!("Directory does not exist: {:?}", dir);
            return Ok(false);
        }

        let (size, file_count) = helpers::calculate_dir_info(&dir)?;

        if list_only {
            self.display_dir_info(&dir_name, &dir, size, file_count)?;
            return Ok(false);
        }

        if dry_run {
            log_info!("[DRY RUN] Would delete {}: {:?}", dir_name, dir);
            log_info!("[DRY RUN] Total size: {}", helpers::format_size(size));
            log_info!("[DRY RUN] Total files: {}", file_count);
            return Ok(false);
        }

        self.display_dir_info(&dir_name, &dir, size, file_count)?;

        if !confirm(
            &format!(
                "Are you sure you want to delete {}? This will remove {} files ({}).",
                dir_name,
                file_count,
                helpers::format_size(size)
            ),
            false,
            None,
        )? {
            log_info!("Clean operation cancelled.");
            return Ok(false);
        }

        std::fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed to delete {}: {:?}", dir_name, dir))?;

        log_success!("{} deleted successfully: {:?}", dir_name, dir);
        Ok(true)
    }

    /// 显示目录信息
    fn display_dir_info(&self, dir_name: &str, dir: &Path, size: u64, file_count: usize) -> Result<()> {
        // 根据 dir_name 判断显示格式
        if dir_name.starts_with("the directory for") {
            // JIRA 目录格式：提取 JIRA ID
            if let Some(jira_id) = dir_name.strip_prefix("the directory for ") {
                log_info!("JIRA ID: {}", jira_id);
            }
        } else {
            // 基础目录格式
            log_info!("{}: {:?}", dir_name, dir);
        }
        log_info!("Directory: {:?}", dir);
        log_info!("Total size: {}", helpers::format_size(size));
        log_info!("Total files: {}", file_count);
        log_break!();
        log_info!("Contents:");
        let contents = helpers::list_dir_contents(dir)?;
        for path in contents {
            if path.is_file() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    log_info!("  📄 {} ({})", path.display(), helpers::format_size(metadata.len()));
                } else {
                    log_info!("  📄 {}", path.display());
                }
            } else if path.is_dir() {
                log_info!("  📁 {}", path.display());
            }
        }
        Ok(())
    }
}

