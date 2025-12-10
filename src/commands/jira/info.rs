use crate::base::dialog::InputDialog;
use crate::base::util::table::{TableBuilder, TableStyle};
use crate::jira::table::AttachmentRow;
use crate::jira::Jira;
use crate::{log_break, log_debug, log_message};
use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use serde_json;
use std::collections::HashMap;

/// 显示 ticket 信息命令
pub struct InfoCommand;

impl InfoCommand {
    /// 显示 ticket 信息
    pub fn show(
        jira_id: Option<String>,
        table: bool,
        json: bool,
        yaml: bool,
        markdown: bool,
    ) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if let Some(id) = jira_id {
            id
        } else {
            InputDialog::new("Enter Jira ticket ID (e.g., PROJ-123)")
                .prompt()
                .context("Failed to read Jira ticket ID")?
        };

        log_debug!("Getting ticket info for {}...", jira_id);

        // 获取 ticket 信息
        let issue = Jira::get_ticket_info(&jira_id)
            .with_context(|| format!("Failed to get ticket info for {}", jira_id))?;

        // 确定输出格式（优先级：json > yaml > markdown > table（默认））
        // 注意：table 是默认格式，即使不指定 --table 标志也使用 table
        let _ = table; // table 是默认行为，显式标记以避免未使用警告
        let format = if json {
            "json"
        } else if yaml {
            "yaml"
        } else if markdown {
            "markdown"
        } else {
            // table 是默认格式
            "table"
        };

        // 根据输出格式选择不同的显示方式
        match format {
            "json" => Self::output_json(&issue)?,
            "yaml" => Self::output_yaml(&issue)?,
            "markdown" => Self::output_markdown(&issue)?,
            _ => Self::output_table(&issue)?,
        }

        Ok(())
    }

    /// 表格格式输出
    fn output_table(issue: &crate::jira::JiraIssue) -> Result<()> {
        // 显示基本信息
        log_break!('=', 40, "Ticket Information");
        log_message!("Key: {}", issue.key);
        log_message!("ID: {}", issue.id);
        log_message!("Summary: {}", issue.fields.summary);
        log_message!("Status: {}", issue.fields.status.name);

        // 显示更多字段
        if let Some(priority) = &issue.fields.priority {
            log_message!("Priority: {}", priority.name);
        }

        if let Some(created) = &issue.fields.created {
            log_message!("Created: {}", format_date(created)?);
        }

        if let Some(updated) = &issue.fields.updated {
            log_message!("Updated: {}", format_date(updated)?);
        }

        if let Some(reporter) = &issue.fields.reporter {
            log_message!(
                "Reporter: {} ({})",
                reporter.display_name,
                reporter.email_address.as_deref().unwrap_or("N/A")
            );
        }

        if let Some(assignee) = &issue.fields.assignee {
            log_message!(
                "Assignee: {} ({})",
                assignee.display_name,
                assignee.email_address.as_deref().unwrap_or("Unassigned")
            );
        } else {
            log_message!("Assignee: Unassigned");
        }

        if let Some(labels) = &issue.fields.labels {
            if !labels.is_empty() {
                log_message!("Labels: {}", labels.join(", "));
            }
        }

        if let Some(components) = &issue.fields.components {
            if !components.is_empty() {
                let component_names: Vec<String> =
                    components.iter().map(|c| c.name.clone()).collect();
                log_message!("Components: {}", component_names.join(", "));
            }
        }

        if let Some(fix_versions) = &issue.fields.fix_versions {
            if !fix_versions.is_empty() {
                let version_names: Vec<String> =
                    fix_versions.iter().map(|v| v.name.clone()).collect();
                log_message!("Fix Versions: {}", version_names.join(", "));
            }
        }

        if let Some(time_tracking) = &issue.fields.time_tracking {
            log_break!();
            log_message!("Time Tracking:");
            if let Some(original) = &time_tracking.original_estimate {
                log_message!("  Original Estimate: {}", original);
            }
            if let Some(remaining) = &time_tracking.remaining_estimate {
                log_message!("  Remaining Estimate: {}", remaining);
            }
            if let Some(spent) = &time_tracking.time_spent {
                log_message!("  Time Spent: {}", spent);
            }
        }

        // 显示关联的 Issues
        if let Some(issuelinks) = &issue.fields.issuelinks {
            if !issuelinks.is_empty() {
                log_break!();
                log_message!("Linked Issues:");
                for link in issuelinks {
                    if let Some(inward) = &link.inward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.inward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        log_message!("  {} {} ({})", link_type, inward.key, inward.id);
                    }
                    if let Some(outward) = &link.outward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.outward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        log_message!("  {} {} ({})", link_type, outward.key, outward.id);
                    }
                }
            }
        }

        // 显示子任务
        if let Some(subtasks) = &issue.fields.subtasks {
            if !subtasks.is_empty() {
                log_break!();
                log_message!("Subtasks:");
                for subtask in subtasks {
                    if let Some(fields) = &subtask.fields {
                        if let Some(summary) = &fields.summary {
                            log_message!("  {}: {} ({})", subtask.key, summary, subtask.id);
                        } else {
                            log_message!("  {} ({})", subtask.key, subtask.id);
                        }
                    } else {
                        log_message!("  {} ({})", subtask.key, subtask.id);
                    }
                }
            }
        }

        // 显示描述
        if let Some(description) = &issue.fields.description {
            if !description.trim().is_empty() {
                log_break!();
                log_message!("Description:");
                log_message!("{}", description);
            }
        }

        // 显示附件列表
        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                log_break!();
                let rows: Vec<AttachmentRow> = attachments
                    .iter()
                    .enumerate()
                    .map(|(idx, attachment)| {
                        let size_str = if let Some(size) = attachment.size {
                            format_size(size)
                        } else {
                            "Unknown".to_string()
                        };

                        AttachmentRow {
                            index: (idx + 1).to_string(),
                            filename: attachment.filename.clone(),
                            size: size_str,
                            mime_type: attachment
                                .mime_type
                                .clone()
                                .unwrap_or_else(|| "-".to_string()),
                        }
                    })
                    .collect();

                log_message!(
                    "{}",
                    TableBuilder::new(rows)
                        .with_title(format!("Attachments ({})", attachments.len()))
                        .with_style(TableStyle::Modern)
                        .render()
                );
            } else {
                log_break!();
                log_message!("Attachments: None");
            }
        } else {
            log_break!();
            log_message!("Attachments: None");
        }

        // 显示评论数量
        if let Some(comments) = &issue.fields.comment {
            let comment_count = comments.comments.len();
            if comment_count > 0 {
                log_break!();
                log_message!("Comments: {} comment(s)", comment_count);
            } else {
                log_break!();
                log_message!("Comments: None");
            }
        } else {
            log_break!();
            log_message!("Comments: None");
        }

        // 显示 Jira URL
        let settings = crate::base::settings::settings::Settings::get();
        let jira_service_address = settings.jira.service_address.clone().unwrap_or_default();
        if !jira_service_address.is_empty() {
            let jira_url = format!("{}/browse/{}", jira_service_address, issue.key);
            log_break!();
            log_message!("URL: {}", jira_url);
        }

        Ok(())
    }

    /// JSON 格式输出
    fn output_json(issue: &crate::jira::JiraIssue) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("issue".to_string(), serde_json::to_value(issue)?);

        log_message!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    /// YAML 格式输出（暂时使用 JSON，后续可以添加 serde_yaml）
    fn output_yaml(issue: &crate::jira::JiraIssue) -> Result<()> {
        // 暂时使用 JSON 格式，因为项目中没有 serde_yaml
        // 如果需要真正的 YAML 支持，需要添加 serde_yaml 依赖
        Self::output_json(issue)
    }

    /// Markdown 格式输出
    fn output_markdown(issue: &crate::jira::JiraIssue) -> Result<()> {
        log_message!("# {}\n", issue.key);
        log_message!("**Summary:** {}\n", issue.fields.summary);
        log_message!("**Status:** {}\n", issue.fields.status.name);

        if let Some(priority) = &issue.fields.priority {
            log_message!("**Priority:** {}\n", priority.name);
        }

        if let Some(created) = &issue.fields.created {
            log_message!("**Created:** {}\n", format_date(created)?);
        }

        if let Some(updated) = &issue.fields.updated {
            log_message!("**Updated:** {}\n", format_date(updated)?);
        }

        if let Some(reporter) = &issue.fields.reporter {
            log_message!(
                "**Reporter:** {} ({})\n",
                reporter.display_name,
                reporter.email_address.as_deref().unwrap_or("N/A")
            );
        }

        if let Some(assignee) = &issue.fields.assignee {
            log_message!(
                "**Assignee:** {} ({})\n",
                assignee.display_name,
                assignee.email_address.as_deref().unwrap_or("Unassigned")
            );
        } else {
            log_message!("**Assignee:** Unassigned\n");
        }

        if let Some(description) = &issue.fields.description {
            if !description.trim().is_empty() {
                log_message!("## Description\n\n{}\n", description);
            }
        }

        Ok(())
    }
}

/// 格式化文件大小
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 格式化日期时间
fn format_date(date_str: &str) -> Result<String> {
    DateTime::<FixedOffset>::parse_from_rfc3339(date_str)
        .or_else(|_| {
            // 尝试其他格式
            DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.3f%z")
        })
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .or_else(|_| Ok(date_str.to_string()))
}
