use crate::{log_info, log_success, Jira};
use anyhow::{Context, Result};

/// 显示 ticket 信息命令
#[allow(dead_code)]
pub struct InfoCommand;

impl InfoCommand {
    /// 显示 ticket 信息
    #[allow(dead_code)]
    pub fn show(jira_id: &str) -> Result<()> {
        log_info!("Getting ticket info for {}...", jira_id);

        // 获取 ticket 信息
        let issue = Jira::get_ticket_info(jira_id)
            .context(format!("Failed to get ticket info for {}", jira_id))?;

        // 显示基本信息
        log_success!("\n📋 Ticket Information");
        log_info!("Key: {}", issue.key);
        log_info!("ID: {}", issue.id);
        log_info!("Summary: {}", issue.fields.summary);
        log_info!("Status: {}", issue.fields.status.name);

        // 显示描述
        if let Some(description) = &issue.fields.description {
            if !description.trim().is_empty() {
                log_info!("\n📝 Description:");
                log_info!("{}", description);
            }
        }

        // 显示附件列表
        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                log_info!("\n📎 Attachments ({}):", attachments.len());
                for (idx, attachment) in attachments.iter().enumerate() {
                    let size_str = if let Some(size) = attachment.size {
                        format_size(size)
                    } else {
                        "Unknown".to_string()
                    };
                    log_info!("  {}. {} ({})", idx + 1, attachment.filename, size_str);
                }
            } else {
                log_info!("\n📎 Attachments: None");
            }
        } else {
            log_info!("\n📎 Attachments: None");
        }

        // 显示评论数量
        if let Some(comments) = &issue.fields.comment {
            let comment_count = comments.comments.len();
            if comment_count > 0 {
                log_info!("\n💬 Comments: {} comment(s)", comment_count);
            } else {
                log_info!("\n💬 Comments: None");
            }
        } else {
            log_info!("\n💬 Comments: None");
        }

        // 显示 Jira URL
        let settings = crate::Settings::load();
        if !settings.jira_service_address.is_empty() {
            let jira_url = format!("{}/browse/{}", settings.jira_service_address, issue.key);
            log_info!("\n🔗 URL: {}", jira_url);
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
