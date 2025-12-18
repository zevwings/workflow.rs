use crate::jira::JiraIssueApi;
use crate::{log_break, log_debug, log_message};
use color_eyre::{eyre::WrapErr, Result};
use serde_json;
use serde_saphyr;
use std::collections::HashMap;

use super::helpers::{format_date, get_jira_id, OutputFormat};
use crate::cli::JiraQueryArgs;

/// 显示变更历史命令
pub struct ChangelogCommand;

impl ChangelogCommand {
    /// 显示 ticket 的变更历史
    pub fn show(args: JiraQueryArgs) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = get_jira_id(args.jira_id.into_option(), None)?;

        // 根据详细程度控制日志输出
        if args.query_display.verbosity.is_verbose() {
            log_debug!("Getting changelog for {}...", jira_id);
        }

        // 确定输出格式
        let format = OutputFormat::from(&args.query_display.output_format);

        // 根据输出格式选择不同的显示方式
        match format {
            OutputFormat::Json => Self::output_json(&jira_id)?,
            OutputFormat::Yaml => Self::output_yaml(&jira_id)?,
            OutputFormat::Markdown => Self::output_markdown(&jira_id)?,
            OutputFormat::Table => Self::output_table(&jira_id)?,
        }

        Ok(())
    }

    /// 表格格式输出
    fn output_table(jira_id: &str) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .wrap_err_with(|| format!("Failed to get changelog for {}", jira_id))?;

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
    fn output_json(jira_id: &str) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .wrap_err_with(|| format!("Failed to get changelog for {}", jira_id))?;

        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("changelog".to_string(), serde_json::to_value(changelog)?);

        log_message!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    /// YAML 格式输出
    fn output_yaml(jira_id: &str) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .wrap_err_with(|| format!("Failed to get changelog for {}", jira_id))?;

        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        output.insert("changelog".to_string(), serde_json::to_value(changelog)?);

        log_message!("{}", serde_saphyr::to_string(&output)?);
        Ok(())
    }

    /// Markdown 格式输出
    fn output_markdown(jira_id: &str) -> Result<()> {
        let changelog = JiraIssueApi::get_issue_changelog(jira_id)
            .wrap_err_with(|| format!("Failed to get changelog for {}", jira_id))?;

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
