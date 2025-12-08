//! 清理日志命令
//!
//! 提供清理日志目录的功能，支持：
//! - 清理指定 JIRA ID 的日志目录（当提供 jira_id 时）
//! - 清理整个基础目录（当指定 --all 标志或交互式输入时留空）
//! - 预览操作（dry-run）
//! - 列出将要删除的内容（list-only）

use anyhow::{Context, Result};
use tabled::Tabled;

use crate::base::util::dialog::InputDialog;
use crate::base::util::format_size;
use crate::base::util::table::{TableBuilder, TableStyle};
use crate::jira::logs::JiraLogs;
use crate::{log_break, log_info, log_success};

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
        let result = logs
            .clean_dir(&jira_id, dry_run, list_only)
            .context("Failed to clean logs directory")?;

        // 显示目录信息
        if let Some(ref dir_info) = result.dir_info {
            // 根据 dir_name 判断显示格式
            if let Some(ref jira_id) = dir_info.jira_id {
                log_info!("JIRA ID: {}", jira_id);
            } else {
                log_info!("{}: {:?}", dir_info.dir_name, dir_info.dir);
            }
            log_info!("Directory: {:?}", dir_info.dir);
            log_info!("Total size: {}", format_size(dir_info.size));
            log_info!("Total files: {}", dir_info.file_count);
            log_break!();
            log_info!("Contents:");

            if dir_info.is_base_dir {
                // 按 ticket 分区显示
                #[derive(Tabled, Clone)]
                struct FileRow {
                    #[tabled(rename = "Type")]
                    file_type: String,
                    #[tabled(rename = "Name")]
                    name: String,
                    #[tabled(rename = "Size")]
                    size: String,
                }

                // 按 ticket 分组显示
                let mut current_ticket: Option<String> = None;
                let mut rows: Vec<FileRow> = Vec::new();

                for entry in &dir_info.contents {
                    // 从 entry_type 中提取 ticket ID
                    let ticket_id = if entry.entry_type.contains("(") {
                        entry
                            .entry_type
                            .split('(')
                            .nth(1)
                            .and_then(|s| s.strip_suffix(')'))
                            .map(|s| s.to_string())
                    } else {
                        None
                    };

                    if ticket_id != current_ticket {
                        // 显示之前的表格
                        if !rows.is_empty() {
                            if let Some(ref ticket) = current_ticket {
                                println!(
                                    "{}",
                                    TableBuilder::new(rows.clone())
                                        .with_title(format!("Files: {}", ticket))
                                        .with_style(TableStyle::Modern)
                                        .render()
                                );
                                log_break!();
                            }
                            rows.clear();
                        }
                        current_ticket = ticket_id;
                    }

                    rows.push(FileRow {
                        file_type: entry.entry_type.clone(),
                        name: entry.name.clone(),
                        size: entry.size.clone().unwrap_or_else(|| "-".to_string()),
                    });
                }

                // 显示最后一个表格
                if !rows.is_empty() {
                    if let Some(ref ticket) = current_ticket {
                        println!(
                            "{}",
                            TableBuilder::new(rows)
                                .with_title(format!("Files: {}", ticket))
                                .with_style(TableStyle::Modern)
                                .render()
                        );
                        log_break!();
                    }
                }
            } else {
                // 单个 ticket 目录，直接列出内容
                for entry in &dir_info.contents {
                    if let Some(ref size) = entry.size {
                        log_info!("  {} {} ({})", entry.entry_type, entry.name, size);
                    } else {
                        log_info!("  {} {}", entry.entry_type, entry.name);
                    }
                }
            }
        }

        if result.deleted {
            log_break!();
            log_success!("Clean completed successfully!");
        } else if result.cancelled {
            log_info!("Clean operation was cancelled.");
        } else if !result.dir_exists {
            log_info!("Directory does not exist.");
        } else if result.dry_run {
            log_info!("[DRY RUN] Preview completed.");
        } else if result.list_only {
            // list_only 模式，信息已在上面的 dir_info 显示中输出
        } else {
            log_info!("Clean operation was cancelled or directory does not exist.");
        }

        Ok(())
    }
}
