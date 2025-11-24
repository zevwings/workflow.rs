use crate::jira::Jira;
use crate::{log_break, log_debug, log_message};
use anyhow::{Context, Result};
use dialoguer::Input;

/// 显示 ticket 信息命令
pub struct InfoCommand;

impl InfoCommand {
    /// 显示 ticket 信息
    pub fn show(jira_id: Option<String>) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = if let Some(id) = jira_id {
            id
        } else {
            Input::<String>::new()
                .with_prompt("Enter Jira ticket ID (e.g., PROJ-123)")
                .interact()
                .context("Failed to read Jira ticket ID")?
        };

        log_debug!("Getting ticket info for {}...", jira_id);

        // 获取 ticket 信息
        let issue = Jira::get_ticket_info(&jira_id)
            .with_context(|| format!("Failed to get ticket info for {}", jira_id))?;

        // 显示基本信息
        log_break!('=', 40, "Ticket Information");
        log_message!("Key: {}", issue.key);
        log_message!("ID: {}", issue.id);
        log_message!("Summary: {}", issue.fields.summary);
        log_message!("Status: {}", issue.fields.status.name);

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
                log_message!("📎 Attachments ({}):", attachments.len());
                for (idx, attachment) in attachments.iter().enumerate() {
                    let size_str = if let Some(size) = attachment.size {
                        format_size(size)
                    } else {
                        "Unknown".to_string()
                    };
                    log_message!("  {}. {} ({})", idx + 1, attachment.filename, size_str);
                }
            } else {
                log_break!();
                log_message!("📎 Attachments: None");
            }
        } else {
            log_break!();
            log_message!("📎 Attachments: None");
        }

        // 显示评论数量
        if let Some(comments) = &issue.fields.comment {
            let comment_count = comments.comments.len();
            if comment_count > 0 {
                log_break!();
                log_message!("💬 Comments: {} comment(s)", comment_count);
            } else {
                log_break!();
                log_message!("💬 Comments: None");
            }
        } else {
            log_break!();
            log_message!("💬 Comments: None");
        }

        // 显示 Jira URL
        let settings = crate::base::settings::settings::Settings::get();
        let jira_service_address = settings.jira.service_address.clone().unwrap_or_default();
        if !jira_service_address.is_empty() {
            let jira_url = format!("{}/browse/{}", jira_service_address, issue.key);
            log_break!();
            log_message!("🔗 URL: {}", jira_url);
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
