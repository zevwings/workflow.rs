use crate::base::dialog::InputDialog;
use crate::jira::JiraIssueApi;
use crate::{log_break, log_debug, log_message};
use anyhow::{Context, Result};
use serde_json;
use std::collections::HashMap;

/// 显示变更历史命令
pub struct ChangelogCommand;

impl ChangelogCommand {
    /// 显示 ticket 的变更历史
    pub fn show(
        jira_id: Option<String>,
        field: Option<String>,
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

        log_debug!("Getting changelog for {}...", jira_id);

        // 确定输出格式（优先级：json > yaml > markdown > table（默认））
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
            "json" => Self::output_json(&jira_id, field.as_deref())?,
            "yaml" => Self::output_yaml(&jira_id, field.as_deref())?,
            "markdown" => Self::output_markdown(&jira_id, field.as_deref())?,
            _ => Self::output_table(&jira_id, field.as_deref())?,
        }

        Ok(())
    }

    /// 表格格式输出
    fn output_table(jira_id: &str, field: Option<&str>) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .with_context(|| format!("Failed to get changelog for {}", jira_id))?;

        log_break!();
        log_break!('=', 40, "Changelog");

        if changelog.histories.is_empty() {
            log_message!("No change history available.");
            return Ok(());
        }

        for history in &changelog.histories {
            log_break!();
            log_message!("Change at {}", format_date(&history.created)?);
            if let Some(author) = &history.author {
                log_message!(
                    "  Author: {} ({})",
                    author.display_name,
                    author.email_address.as_deref().unwrap_or("N/A")
                );
            }

            for item in &history.items {
                // 如果指定了字段过滤，只显示该字段的变更
                if let Some(filter_field) = field {
                    if item.field != filter_field {
                        continue;
                    }
                }

                log_message!("  Field: {}", item.field);
                if let Some(from_str) = &item.from_string {
                    log_message!("    From: {}", from_str);
                } else if let Some(from) = &item.from {
                    log_message!("    From: {}", from);
                }
                if let Some(to_str) = &item.to_string {
                    log_message!("    To: {}", to_str);
                } else if let Some(to) = &item.to {
                    log_message!("    To: {}", to);
                }
            }
        }

        Ok(())
    }

    /// JSON 格式输出
    fn output_json(jira_id: &str, field: Option<&str>) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .with_context(|| format!("Failed to get changelog for {}", jira_id))?;

        let mut output: HashMap<String, serde_json::Value> = HashMap::new();

        // 如果指定了字段过滤，只包含该字段的变更
        if let Some(filter_field) = field {
            let mut filtered_changelog = changelog.clone();
            filtered_changelog.histories = changelog
                .histories
                .into_iter()
                .map(|mut history| {
                    history.items.retain(|item| item.field == filter_field);
                    history
                })
                .filter(|history| !history.items.is_empty())
                .collect();
            output.insert(
                "changelog".to_string(),
                serde_json::to_value(filtered_changelog)?,
            );
        } else {
            output.insert("changelog".to_string(), serde_json::to_value(changelog)?);
        }

        log_message!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    /// YAML 格式输出（暂时使用 JSON）
    fn output_yaml(jira_id: &str, field: Option<&str>) -> Result<()> {
        // 暂时使用 JSON 格式，因为项目中没有 serde_yaml
        Self::output_json(jira_id, field)
    }

    /// Markdown 格式输出
    fn output_markdown(jira_id: &str, field: Option<&str>) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .with_context(|| format!("Failed to get changelog for {}", jira_id))?;

        if changelog.histories.is_empty() {
            log_message!("# Changelog\n\nNo change history available.\n");
            return Ok(());
        }

        log_message!("# Changelog\n");

        for history in &changelog.histories {
            log_message!("## {}", format_date(&history.created)?);
            if let Some(author) = &history.author {
                log_message!(
                    "**Author:** {} ({})\n",
                    author.display_name,
                    author.email_address.as_deref().unwrap_or("N/A")
                );
            }

            for item in &history.items {
                if let Some(filter_field) = field {
                    if item.field != filter_field {
                        continue;
                    }
                }

                log_message!("- **{}**", item.field);
                if let Some(from_str) = &item.from_string {
                    log_message!("  - From: {}", from_str);
                } else if let Some(from) = &item.from {
                    log_message!("  - From: {}", from);
                }
                if let Some(to_str) = &item.to_string {
                    log_message!("  - To: {}", to_str);
                } else if let Some(to) = &item.to {
                    log_message!("  - To: {}", to);
                }
            }
            log_break!();
        }

        Ok(())
    }
}

/// 格式化日期时间
fn format_date(date_str: &str) -> Result<String> {
    chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(date_str)
        .or_else(|_| {
            // 尝试其他格式
            chrono::DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.3f%z")
        })
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .or_else(|_| Ok(date_str.to_string()))
}
