//! 显示 Jira ticket 信息命令

use crate::registry;
use crate::workflows::utils::jira::get_jira_id_interactive;
use color_eyre::Result;
use domain::JiraIssue;
use prompt::{info, success, Spinner};

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
        let issue = Spinner::new(format!("Fetching JIRA ticket '{}'...", jira_id))
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
        // 由于 domain::JiraIssue 没有实现 Serialize，我们手动构建 JSON
        // 或者可以创建一个临时的可序列化结构
        let json = serde_json::json!({
            "id": issue.id,
            "key": issue.key,
            "summary": issue.summary,
            "status": issue.status,
            "assignee": issue.assignee,
            "description": issue.description,
            "priority": issue.priority,
            "created": issue.created,
            "updated": issue.updated,
            "labels": issue.labels,
            "components": issue.components,
            "attachments": issue.attachments.iter().map(|a| serde_json::json!({
                "filename": a.filename,
                "size": a.size,
                "url": a.url,
            })).collect::<Vec<_>>(),
            "comments": issue.comments.iter().map(|c| serde_json::json!({
                "id": c.id,
                "body": c.body,
                "created": c.created,
                "author": c.author.as_ref().map(|u| serde_json::json!({
                    "display_name": u.display_name,
                    "account_id": u.account_id,
                })),
            })).collect::<Vec<_>>(),
            "reporter": issue.reporter.as_ref().map(|u| serde_json::json!({
                "display_name": u.display_name,
                "account_id": u.account_id,
            })),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        Ok(())
    }

    /// Markdown 格式输出
    fn output_markdown(&self, issue: &JiraIssue) -> Result<(), Box<dyn std::error::Error>> {
        println!("# {}", issue.key);
        println!();
        println!("**Summary:** {}", issue.summary);
        println!();

        if let Some(description) = &issue.description {
            println!("## Description");
            println!();
            println!("{}", description);
            println!();
        }

        println!("## Details");
        println!();
        println!("- **Status:** {}", issue.status);
        println!("- **ID:** {}", issue.id);
        println!("- **Key:** {}", issue.key);

        if let Some(reporter) = &issue.reporter {
            println!(
                "- **Reporter:** {} ({})",
                reporter.display_name, reporter.account_id
            );
        }

        if let Some(assignee) = &issue.assignee {
            println!("- **Assignee:** {}", assignee);
        }

        if let Some(priority) = &issue.priority {
            println!("- **Priority:** {}", priority);
        }

        if let Some(created) = &issue.created {
            println!("- **Created:** {}", created);
        }

        if let Some(updated) = &issue.updated {
            println!("- **Updated:** {}", updated);
        }

        if !issue.labels.is_empty() {
            println!("- **Labels:** {}", issue.labels.join(", "));
        }

        if !issue.components.is_empty() {
            println!("- **Components:** {}", issue.components.join(", "));
        }

        if !issue.attachments.is_empty() {
            println!();
            println!("## Attachments");
            println!();
            for attachment in &issue.attachments {
                let size_str = if attachment.size > 0 {
                    format!(" ({})", self.format_size(attachment.size))
                } else {
                    String::new()
                };
                println!(
                    "- [{}]({}){}",
                    attachment.filename, attachment.url, size_str
                );
            }
        }

        if !issue.comments.is_empty() {
            println!();
            println!("## Comments");
            println!();
            for (idx, comment) in issue.comments.iter().enumerate() {
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

        Ok(())
    }

    /// 人类可读格式输出
    fn output_human_readable(&self, issue: &JiraIssue) -> Result<(), Box<dyn std::error::Error>> {
        success!("JIRA Ticket: {}", issue.key);
        info!("Summary: {}", issue.summary);
        info!("Status: {}", issue.status);

        if let Some(description) = &issue.description {
            println!();
            info!("Description:");
            println!("{}", description);
        }

        println!();
        info!("Details:");
        println!("  ID: {}", issue.id);
        println!("  Key: {}", issue.key);

        if let Some(reporter) = &issue.reporter {
            println!(
                "  Reporter: {} ({})",
                reporter.display_name, reporter.account_id
            );
        }

        if let Some(assignee) = &issue.assignee {
            println!("  Assignee: {}", assignee);
        } else {
            println!("  Assignee: Unassigned");
        }

        if let Some(priority) = &issue.priority {
            println!("  Priority: {}", priority);
        }

        if let Some(created) = &issue.created {
            println!("  Created: {}", created);
        }

        if let Some(updated) = &issue.updated {
            println!("  Updated: {}", updated);
        }

        if !issue.labels.is_empty() {
            println!("  Labels: {}", issue.labels.join(", "));
        }

        if !issue.components.is_empty() {
            println!("  Components: {}", issue.components.join(", "));
        }

        if !issue.attachments.is_empty() {
            println!();
            info!("Attachments ({}):", issue.attachments.len());
            for attachment in &issue.attachments {
                let size_str = if attachment.size > 0 {
                    format!(" - {} bytes", attachment.size)
                } else {
                    String::new()
                };
                println!("  - {} {}", attachment.filename, size_str);
            }
        }

        if !issue.comments.is_empty() {
            println!();
            info!("Comments ({}):", issue.comments.len());
            for (idx, comment) in issue.comments.iter().enumerate() {
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
