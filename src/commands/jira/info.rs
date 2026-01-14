use crate::base::format::DisplayFormatter;
use crate::base::interactive::{TableBuilder, TableStyle};
use crate::jira::table::AttachmentRow;
use crate::jira::Jira;
use crate::spinner;
use crate::{br, info};
use color_eyre::{eyre::WrapErr, Result};
use serde_json;
use serde_saphyr;
use std::collections::HashMap;

use super::helpers::{format_date, get_jira_id, OutputFormat};
use crate::cli::JiraQueryArgs;

/// 显示 ticket 信息命令
pub struct InfoCommand;

impl InfoCommand {
    /// 显示 ticket 信息
    pub fn show(args: JiraQueryArgs) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = get_jira_id(args.jira_id.into_option(), None)?;

        // 根据详细程度控制 Spinner 显示
        let issue = if args.query_display.verbosity.is_quiet() {
            // 静默模式：不显示 Spinner
            Jira::get_ticket_info(&jira_id)
                .wrap_err_with(|| format!("Failed to get ticket info for {}", jira_id))?
        } else {
            // 正常/详细模式：显示 Spinner
            spinner!("Getting ticket info for {}...", jira_id).with(|| {
                Jira::get_ticket_info(&jira_id)
                    .wrap_err_with(|| format!("Failed to get ticket info for {}", jira_id))
            })?
        };

        // 确定输出格式
        let format = OutputFormat::from(&args.query_display.output_format);

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
        br!('=', 40, "Ticket Information");
        info!("Key: {}", issue.key);
        info!("ID: {}", issue.id);
        info!("Summary: {}", issue.fields.summary);
        info!("Status: {}", issue.fields.status.name);

        // 显示更多字段
        if let Some(priority) = &issue.fields.priority {
            info!("Priority: {}", priority.name);
        }

        if let Some(created) = &issue.fields.created {
            info!("Created: {}", format_date(created)?);
        }

        if let Some(updated) = &issue.fields.updated {
            info!("Updated: {}", format_date(updated)?);
        }

        if let Some(reporter) = &issue.fields.reporter {
            info!(
                "Reporter: {} ({})",
                reporter.display_name,
                reporter.email_address.as_deref().unwrap_or("N/A")
            );
        }

        if let Some(assignee) = &issue.fields.assignee {
            info!(
                "Assignee: {} ({})",
                assignee.display_name,
                assignee.email_address.as_deref().unwrap_or("Unassigned")
            );
        } else {
            info!("Assignee: Unassigned");
        }

        if let Some(labels) = &issue.fields.labels {
            if !labels.is_empty() {
                info!("Labels: {}", labels.join(", "));
            }
        }

        if let Some(components) = &issue.fields.components {
            if !components.is_empty() {
                let component_names: Vec<String> =
                    components.iter().map(|c| c.name.clone()).collect();
                info!("Components: {}", component_names.join(", "));
            }
        }

        if let Some(fix_versions) = &issue.fields.fix_versions {
            if !fix_versions.is_empty() {
                let version_names: Vec<String> =
                    fix_versions.iter().map(|v| v.name.clone()).collect();
                info!("Fix Versions: {}", version_names.join(", "));
            }
        }

        if let Some(time_tracking) = &issue.fields.time_tracking {
            br!();
            info!("Time Tracking:");
            if let Some(original) = &time_tracking.original_estimate {
                info!("  Original Estimate: {}", original);
            }
            if let Some(remaining) = &time_tracking.remaining_estimate {
                info!("  Remaining Estimate: {}", remaining);
            }
            if let Some(spent) = &time_tracking.time_spent {
                info!("  Time Spent: {}", spent);
            }
        }

        // 显示关联的 Issues
        if let Some(issuelinks) = &issue.fields.issuelinks {
            if !issuelinks.is_empty() {
                br!();
                info!("Linked Issues:");
                for link in issuelinks {
                    if let Some(inward) = &link.inward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.inward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        info!("  {} {} ({})", link_type, inward.key, inward.id);
                    }
                    if let Some(outward) = &link.outward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.outward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        info!("  {} {} ({})", link_type, outward.key, outward.id);
                    }
                }
            }
        }

        // 显示子任务
        if let Some(subtasks) = &issue.fields.subtasks {
            if !subtasks.is_empty() {
                br!();
                info!("Subtasks:");
                for subtask in subtasks {
                    if let Some(fields) = &subtask.fields {
                        if let Some(summary) = &fields.summary {
                            info!("  {}: {} ({})", subtask.key, summary, subtask.id);
                        } else {
                            info!("  {} ({})", subtask.key, subtask.id);
                        }
                    } else {
                        info!("  {} ({})", subtask.key, subtask.id);
                    }
                }
            }
        }

        // 显示描述
        if let Some(description) = &issue.fields.description {
            if !description.trim().is_empty() {
                br!();
                info!("Description:");
                info!("{}", description);
            }
        }

        // 显示附件列表
        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                br!();
                let rows: Vec<AttachmentRow> = attachments
                    .iter()
                    .enumerate()
                    .map(|(idx, attachment)| {
                        let size_str = if let Some(size) = attachment.size {
                            DisplayFormatter::size(size)
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

                info!(
                    "{}",
                    TableBuilder::from_tabled(rows)
                        .with_title(format!("Attachments ({})", attachments.len()))
                        .with_style(TableStyle::Modern)
                        .render()
                );
            } else {
                br!();
                info!("Attachments: None");
            }
        } else {
            br!();
            info!("Attachments: None");
        }

        // 显示评论数量
        if let Some(comments) = &issue.fields.comment {
            let comment_count = comments.comments.len();
            if comment_count > 0 {
                br!();
                info!("Comments: {} comment(s)", comment_count);
            } else {
                br!();
                info!("Comments: None");
            }
        } else {
            br!();
            info!("Comments: None");
        }

        // 显示 Jira URL
        let settings = crate::base::settings::Settings::get();
        let jira_service_address = settings.jira.service_address.clone().unwrap_or_default();
        if !jira_service_address.is_empty() {
            let jira_url = format!("{}/browse/{}", jira_service_address, issue.key);
            br!();
            info!("URL: {}", jira_url);
        }

        Ok(())
    }

    /// JSON 格式输出
    fn output_json(issue: &crate::jira::JiraIssue) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("issue".to_string(), serde_json::to_value(issue)?);

        info!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    /// YAML 格式输出
    fn output_yaml(issue: &crate::jira::JiraIssue) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("issue".to_string(), serde_json::to_value(issue)?);

        info!("{}", serde_saphyr::to_string(&output)?);
        Ok(())
    }

    /// Markdown 格式输出
    fn output_markdown(issue: &crate::jira::JiraIssue) -> Result<()> {
        info!("# {}\n", issue.key);
        info!("**ID:** {}\n", issue.id);
        info!("**Summary:** {}\n", issue.fields.summary);
        info!("**Status:** {}\n", issue.fields.status.name);

        if let Some(priority) = &issue.fields.priority {
            info!("**Priority:** {}\n", priority.name);
        }

        if let Some(created) = &issue.fields.created {
            info!("**Created:** {}\n", format_date(created)?);
        }

        if let Some(updated) = &issue.fields.updated {
            info!("**Updated:** {}\n", format_date(updated)?);
        }

        if let Some(reporter) = &issue.fields.reporter {
            info!(
                "**Reporter:** {} ({})\n",
                reporter.display_name,
                reporter.email_address.as_deref().unwrap_or("N/A")
            );
        }

        if let Some(assignee) = &issue.fields.assignee {
            info!(
                "**Assignee:** {} ({})\n",
                assignee.display_name,
                assignee.email_address.as_deref().unwrap_or("Unassigned")
            );
        } else {
            info!("**Assignee:** Unassigned\n");
        }

        if let Some(labels) = &issue.fields.labels {
            if !labels.is_empty() {
                info!("**Labels:** {}\n", labels.join(", "));
            }
        }

        if let Some(components) = &issue.fields.components {
            if !components.is_empty() {
                let component_names: Vec<String> =
                    components.iter().map(|c| c.name.clone()).collect();
                info!("**Components:** {}\n", component_names.join(", "));
            }
        }

        if let Some(fix_versions) = &issue.fields.fix_versions {
            if !fix_versions.is_empty() {
                let version_names: Vec<String> =
                    fix_versions.iter().map(|v| v.name.clone()).collect();
                info!("**Fix Versions:** {}\n", version_names.join(", "));
            }
        }

        if let Some(time_tracking) = &issue.fields.time_tracking {
            info!("\n## Time Tracking\n");
            if let Some(original) = &time_tracking.original_estimate {
                info!("- **Original Estimate:** {}\n", original);
            }
            if let Some(remaining) = &time_tracking.remaining_estimate {
                info!("- **Remaining Estimate:** {}\n", remaining);
            }
            if let Some(spent) = &time_tracking.time_spent {
                info!("- **Time Spent:** {}\n", spent);
            }
        }

        // 显示关联的 Issues
        if let Some(issuelinks) = &issue.fields.issuelinks {
            if !issuelinks.is_empty() {
                info!("\n## Linked Issues\n");
                for link in issuelinks {
                    if let Some(inward) = &link.inward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.inward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        info!("- **{}:** {} ({})\n", link_type, inward.key, inward.id);
                    }
                    if let Some(outward) = &link.outward_issue {
                        let link_type = link
                            .link_type
                            .as_ref()
                            .and_then(|lt| lt.outward.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("linked");
                        info!("- **{}:** {} ({})\n", link_type, outward.key, outward.id);
                    }
                }
            }
        }

        // 显示子任务
        if let Some(subtasks) = &issue.fields.subtasks {
            if !subtasks.is_empty() {
                info!("\n## Subtasks\n");
                for subtask in subtasks {
                    if let Some(fields) = &subtask.fields {
                        if let Some(summary) = &fields.summary {
                            info!("- **{}:** {} ({})\n", subtask.key, summary, subtask.id);
                        } else {
                            info!("- **{}:** ({})\n", subtask.key, subtask.id);
                        }
                    } else {
                        info!("- **{}:** ({})\n", subtask.key, subtask.id);
                    }
                }
            }
        }

        // 显示描述
        if let Some(description) = &issue.fields.description {
            if !description.trim().is_empty() {
                info!("\n## Description\n\n{}\n", description);
            }
        }

        // 显示附件列表
        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                info!("\n## Attachments ({})\n\n", attachments.len());
                for attachment in attachments {
                    let size_str = if let Some(size) = attachment.size {
                        DisplayFormatter::size(size)
                    } else {
                        "Unknown".to_string()
                    };
                    info!(
                        "- **{}** ({}, {})\n",
                        attachment.filename,
                        size_str,
                        attachment.mime_type.as_deref().unwrap_or("-")
                    );
                }
            } else {
                info!("\n## Attachments\n\nNone\n");
            }
        } else {
            info!("\n## Attachments\n\nNone\n");
        }

        // 显示评论数量
        if let Some(comments) = &issue.fields.comment {
            let comment_count = comments.comments.len();
            if comment_count > 0 {
                info!("\n## Comments\n\n{} comment(s)\n", comment_count);
            } else {
                info!("\n## Comments\n\nNone\n");
            }
        } else {
            info!("\n## Comments\n\nNone\n");
        }

        // 显示 Jira URL
        let settings = crate::base::settings::Settings::get();
        let jira_service_address = settings.jira.service_address.clone().unwrap_or_default();
        if !jira_service_address.is_empty() {
            let jira_url = format!("{}/browse/{}", jira_service_address, issue.key);
            info!("\n## URL\n\n{}\n", jira_url);
        }

        Ok(())
    }
}
