use crate::base::indicator::Spinner;
use crate::base::util::table::{TableBuilder, TableStyle};
use crate::jira::table::AttachmentRow;
use crate::jira::Jira;
use crate::{log_break, log_message};
use color_eyre::{eyre::WrapErr, Result};
use serde_json;
use serde_saphyr;
use std::collections::HashMap;

use super::helpers::{format_date, get_jira_id, OutputFormat};
use crate::cli::OutputFormatArgs;

/// 显示 ticket 信息命令
pub struct InfoCommand;

impl InfoCommand {
    /// 显示 ticket 信息
    pub fn show(jira_id: Option<String>, output_format: OutputFormatArgs) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = get_jira_id(jira_id, None)?;

        // 获取 ticket 信息（使用 Spinner 显示加载状态）
        let issue = Spinner::with(format!("Getting ticket info for {}...", jira_id), || {
            Jira::get_ticket_info(&jira_id)
                .wrap_err_with(|| format!("Failed to get ticket info for {}", jira_id))
        })?;

        // 确定输出格式
        let format = OutputFormat::from(&output_format);

        // 根据输出格式选择不同的显示方式
        match format {
            OutputFormat::Json => Self::output_json(&issue)?,
            OutputFormat::Yaml => Self::output_yaml(&issue)?,
            OutputFormat::Markdown => Self::output_markdown(&issue)?,
            OutputFormat::Table => Self::output_table(&issue)?,
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

    /// YAML 格式输出
    fn output_yaml(issue: &crate::jira::JiraIssue) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("issue".to_string(), serde_json::to_value(issue)?);

        log_message!("{}", serde_saphyr::to_string(&output)?);
        Ok(())
    }

    /// Markdown 格式输出
    fn output_markdown(issue: &crate::jira::JiraIssue) -> Result<()> {
        log_message!("# {}\n", issue.key);
        log_message!("**ID:** {}\n", issue.id);
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

        if let Some(labels) = &issue.fields.labels {
            if !labels.is_empty() {
                log_message!("**Labels:** {}\n", labels.join(", "));
            }
        }

        if let Some(components) = &issue.fields.components {
            if !components.is_empty() {
                let component_names: Vec<String> =
                    components.iter().map(|c| c.name.clone()).collect();
                log_message!("**Components:** {}\n", component_names.join(", "));
            }
        }

        if let Some(fix_versions) = &issue.fields.fix_versions {
            if !fix_versions.is_empty() {
                let version_names: Vec<String> =
                    fix_versions.iter().map(|v| v.name.clone()).collect();
                log_message!("**Fix Versions:** {}\n", version_names.join(", "));
            }
        }

        if let Some(time_tracking) = &issue.fields.time_tracking {
            log_message!("\n## Time Tracking\n");
            if let Some(original) = &time_tracking.original_estimate {
                log_message!("- **Original Estimate:** {}\n", original);
            }
            if let Some(remaining) = &time_tracking.remaining_estimate {
                log_message!("- **Remaining Estimate:** {}\n", remaining);
            }
            if let Some(spent) = &time_tracking.time_spent {
                log_message!("- **Time Spent:** {}\n", spent);
            }
        }

        // 显示关联的 Issues
        if let Some(issuelinks) = &issue.fields.issuelinks {
            if !issuelinks.is_empty() {
                log_message!("\n## Linked Issues\n");
                for link in issuelinks {
                    if let Some(inward) = &link.inward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.inward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        log_message!("- **{}:** {} ({})\n", link_type, inward.key, inward.id);
                    }
                    if let Some(outward) = &link.outward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.outward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        log_message!("- **{}:** {} ({})\n", link_type, outward.key, outward.id);
                    }
                }
            }
        }

        // 显示子任务
        if let Some(subtasks) = &issue.fields.subtasks {
            if !subtasks.is_empty() {
                log_message!("\n## Subtasks\n");
                for subtask in subtasks {
                    if let Some(fields) = &subtask.fields {
                        if let Some(summary) = &fields.summary {
                            log_message!("- **{}:** {} ({})\n", subtask.key, summary, subtask.id);
                        } else {
                            log_message!("- **{}:** ({})\n", subtask.key, subtask.id);
                        }
                    } else {
                        log_message!("- **{}:** ({})\n", subtask.key, subtask.id);
                    }
                }
            }
        }

        // 显示描述
        if let Some(description) = &issue.fields.description {
            if !description.trim().is_empty() {
                log_message!("\n## Description\n\n{}\n", description);
            }
        }

        // 显示附件列表
        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                log_message!("\n## Attachments ({})\n\n", attachments.len());
                for attachment in attachments {
                    let size_str = if let Some(size) = attachment.size {
                        format_size(size)
                    } else {
                        "Unknown".to_string()
                    };
                    log_message!(
                        "- **{}** ({}, {})\n",
                        attachment.filename,
                        size_str,
                        attachment.mime_type.as_deref().unwrap_or("-")
                    );
                }
            } else {
                log_message!("\n## Attachments\n\nNone\n");
            }
        } else {
            log_message!("\n## Attachments\n\nNone\n");
        }

        // 显示评论数量
        if let Some(comments) = &issue.fields.comment {
            let comment_count = comments.comments.len();
            if comment_count > 0 {
                log_message!("\n## Comments\n\n{} comment(s)\n", comment_count);
            } else {
                log_message!("\n## Comments\n\nNone\n");
            }
        } else {
            log_message!("\n## Comments\n\nNone\n");
        }

        // 显示 Jira URL
        let settings = crate::base::settings::settings::Settings::get();
        let jira_service_address = settings.jira.service_address.clone().unwrap_or_default();
        if !jira_service_address.is_empty() {
            let jira_url = format!("{}/browse/{}", jira_service_address, issue.key);
            log_message!("\n## URL\n\n{}\n", jira_url);
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
