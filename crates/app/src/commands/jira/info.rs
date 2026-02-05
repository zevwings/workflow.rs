//! 显示 Jira ticket 信息命令

use domain::JiraIssue;
use prompt::{info, spinner, success};

use crate::registry;
use crate::workflows::utils::jira::get_jira_id_interactive;

/// Jira Info 命令
pub struct JiraInfoCommand {
    jira_id: Option<String>,
    json: bool,
    markdown: bool,
}

impl JiraInfoCommand {
    /// 创建新的 JiraInfoCommand
    pub fn new(jira_id: Option<String>, json: bool, markdown: bool) -> Self {
        Self {
            jira_id,
            json,
            markdown,
        }
    }

    /// 运行 `workflow jira info` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive(self.jira_id.clone())?;

        // 获取 JiraRepository
        let jira_repo = registry::get_jira_repository();

        // 获取 ticket 信息
        let issue = spinner!("Fetching JIRA ticket '{}'...", jira_id)
            .with(|| jira_repo.get_issue_info(&jira_id))
            .map_err(|e| format!("Failed to fetch JIRA ticket: {}", e))?;

        // 根据输出格式显示信息
        if self.json {
            self.output_json(&issue)?;
        } else if self.markdown {
            self.output_markdown(&issue)?;
        } else {
            self.output_human_readable(&issue)?;
        }

        Ok(())
    }

    /// JSON 格式输出
    fn output_json(&self, issue: &JiraIssue) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", serde_json::to_string_pretty(issue)?);
        Ok(())
    }

    /// Markdown 格式输出
    fn output_markdown(&self, issue: &JiraIssue) -> Result<(), Box<dyn std::error::Error>> {
        println!("# {}", issue.key);
        println!();
        println!("**Summary:** {}", issue.fields.summary);
        println!();

        if let Some(description) = &issue.fields.description {
            println!("## Description");
            println!();
            println!("{}", description);
            println!();
        }

        println!("## Details");
        println!();
        println!("- **Status:** {}", issue.fields.status.name);
        println!("- **ID:** {}", issue.id);
        println!("- **Key:** {}", issue.key);

        if let Some(reporter) = &issue.fields.reporter {
            println!(
                "- **Reporter:** {} ({})",
                reporter.display_name, reporter.account_id
            );
        }

        if let Some(assignee) = &issue.fields.assignee {
            println!(
                "- **Assignee:** {} ({})",
                assignee.display_name, assignee.account_id
            );
        }

        if let Some(priority) = &issue.fields.priority {
            println!("- **Priority:** {}", priority.name);
        }

        if let Some(created) = &issue.fields.created {
            println!("- **Created:** {}", created);
        }

        if let Some(updated) = &issue.fields.updated {
            println!("- **Updated:** {}", updated);
        }

        if let Some(labels) = &issue.fields.labels {
            if !labels.is_empty() {
                println!("- **Labels:** {}", labels.join(", "));
            }
        }

        if let Some(components) = &issue.fields.components {
            if !components.is_empty() {
                let component_names: Vec<&str> =
                    components.iter().map(|c| c.name.as_str()).collect();
                println!("- **Components:** {}", component_names.join(", "));
            }
        }

        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                println!();
                println!("## Attachments");
                println!();
                for attachment in attachments {
                    let size_str = if let Some(size) = attachment.size {
                        if size > 0 {
                            format!(" ({})", self.format_size(size))
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    println!(
                        "- [{}]({}){}",
                        attachment.filename, attachment.content_url, size_str
                    );
                }
            }
        }

        if let Some(comments_container) = &issue.fields.comment {
            if !comments_container.comments.is_empty() {
                println!();
                println!("## Comments");
                println!();
                for (idx, comment) in comments_container.comments.iter().enumerate() {
                    println!("### Comment #{}", idx + 1);
                    if let Some(author) = &comment.author {
                        println!(
                            "**Author:** {} ({})",
                            author.display_name, author.account_id
                        );
                    }
                    println!("**Created:** {}", comment.created);
                    println!();
                    println!("{}", comment.body);
                    println!();
                }
            }
        }

        Ok(())
    }

    /// 人类可读格式输出
    fn output_human_readable(&self, issue: &JiraIssue) -> Result<(), Box<dyn std::error::Error>> {
        success!("JIRA Ticket: {}", issue.key);
        info!("Summary: {}", issue.fields.summary);
        info!("Status: {}", issue.fields.status.name);

        if let Some(description) = &issue.fields.description {
            println!();
            info!("Description:");
            println!("{}", description);
        }

        println!();
        info!("Details:");
        println!("  ID: {}", issue.id);
        println!("  Key: {}", issue.key);

        if let Some(reporter) = &issue.fields.reporter {
            println!(
                "  Reporter: {} ({})",
                reporter.display_name, reporter.account_id
            );
        }

        if let Some(assignee) = &issue.fields.assignee {
            println!(
                "  Assignee: {} ({})",
                assignee.display_name, assignee.account_id
            );
        } else {
            println!("  Assignee: Unassigned");
        }

        if let Some(priority) = &issue.fields.priority {
            println!("  Priority: {}", priority.name);
        }

        if let Some(created) = &issue.fields.created {
            println!("  Created: {}", created);
        }

        if let Some(updated) = &issue.fields.updated {
            println!("  Updated: {}", updated);
        }

        if let Some(labels) = &issue.fields.labels {
            if !labels.is_empty() {
                println!("  Labels: {}", labels.join(", "));
            }
        }

        if let Some(components) = &issue.fields.components {
            if !components.is_empty() {
                let component_names: Vec<&str> =
                    components.iter().map(|c| c.name.as_str()).collect();
                println!("  Components: {}", component_names.join(", "));
            }
        }

        if let Some(attachments) = &issue.fields.attachment {
            if !attachments.is_empty() {
                println!();
                info!("Attachments ({}):", attachments.len());
                for attachment in attachments {
                    let size_str = if let Some(size) = attachment.size {
                        if size > 0 {
                            format!(" - {} bytes", size)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    println!("  - {} {}", attachment.filename, size_str);
                }
            }
        }

        if let Some(comments_container) = &issue.fields.comment {
            if !comments_container.comments.is_empty() {
                println!();
                info!("Comments ({}):", comments_container.comments.len());
                for (idx, comment) in comments_container.comments.iter().enumerate() {
                    println!();
                    println!("  Comment #{}:", idx + 1);
                    if let Some(author) = &comment.author {
                        println!(
                            "    Author: {} ({})",
                            author.display_name, author.account_id
                        );
                    }
                    println!("    Created: {}", comment.created);
                    println!("    Body: {}", comment.body);
                }
            }
        }

        Ok(())
    }

    /// 格式化文件大小
    fn format_size(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        if unit_idx == 0 {
            format!("{} {}", bytes, UNITS[unit_idx])
        } else {
            format!("{:.2} {}", size, UNITS[unit_idx])
        }
    }
}
