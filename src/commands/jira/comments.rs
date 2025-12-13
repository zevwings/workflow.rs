use crate::jira::Jira;
use crate::{log_break, log_debug, log_message};
use chrono::{DateTime, FixedOffset};
use color_eyre::{eyre::WrapErr, Result};
use serde_json;
use serde_saphyr;
use std::collections::HashMap;

use super::helpers::{format_date, get_jira_id, OutputFormat};
use crate::cli::OutputFormatArgs;

/// 显示评论命令
pub struct CommentsCommand;

impl CommentsCommand {
    /// 显示 ticket 的评论
    pub fn show(
        jira_id: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        author: Option<String>,
        since: Option<String>,
        output_format: OutputFormatArgs,
    ) -> Result<()> {
        // 获取 JIRA ID（从参数或交互式输入）
        let jira_id = get_jira_id(jira_id, None)?;

        log_debug!("Getting comments for {}...", jira_id);

        // 获取 ticket 信息
        let issue = Jira::get_ticket_info(&jira_id)
            .wrap_err_with(|| format!("Failed to get ticket info for {}", jira_id))?;

        // 确定输出格式
        let format = OutputFormat::from(&output_format);

        // 排序方式：默认使用降序（desc）
        let sort = "desc";

        // 根据输出格式选择不同的显示方式
        match format {
            OutputFormat::Json => Self::output_json(&issue.fields.comment, &jira_id)?,
            OutputFormat::Yaml => Self::output_yaml(&issue.fields.comment, &jira_id)?,
            OutputFormat::Markdown => Self::output_markdown(
                &issue.fields.comment,
                limit,
                offset,
                sort,
                author.as_deref(),
                since.as_deref(),
            )?,
            OutputFormat::Table => Self::output_table(
                &issue.fields.comment,
                limit,
                offset,
                sort,
                author.as_deref(),
                since.as_deref(),
            )?,
        }

        Ok(())
    }

    /// 表格格式输出
    fn output_table(
        comments: &Option<crate::jira::JiraComments>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort: &str,
        author: Option<&str>,
        since: Option<&str>,
    ) -> Result<()> {
        let Some(comments_data) = comments else {
            log_break!();
            log_message!("Comments: None");
            return Ok(());
        };

        let mut filtered_comments = comments_data.comments.clone();

        // 按作者过滤
        if let Some(author_email) = author {
            filtered_comments.retain(|c| {
                c.author
                    .as_ref()
                    .and_then(|a| a.email_address.as_ref())
                    .map(|e| e == author_email)
                    .unwrap_or(false)
            });
        }

        // 按时间过滤
        if let Some(since_date) = since {
            if let Ok(since_dt) = DateTime::<FixedOffset>::parse_from_rfc3339(since_date) {
                filtered_comments.retain(|c| {
                    DateTime::<FixedOffset>::parse_from_rfc3339(&c.created)
                        .map(|dt| dt >= since_dt)
                        .unwrap_or(false)
                });
            }
        }

        // 排序
        filtered_comments.sort_by(|a, b| {
            let a_time = DateTime::<FixedOffset>::parse_from_rfc3339(&a.created).ok();
            let b_time = DateTime::<FixedOffset>::parse_from_rfc3339(&b.created).ok();
            match (a_time, b_time) {
                (Some(a_dt), Some(b_dt)) => {
                    if sort == "asc" {
                        a_dt.cmp(&b_dt)
                    } else {
                        b_dt.cmp(&a_dt)
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        // 分页
        let start = offset.unwrap_or(0);
        let end = limit.map(|l| start + l).unwrap_or(filtered_comments.len());
        let paginated_comments =
            filtered_comments.into_iter().skip(start).take(end - start).collect::<Vec<_>>();

        if paginated_comments.is_empty() {
            log_break!();
            log_message!("Comments: None");
            return Ok(());
        }

        log_break!();
        log_break!('=', 40, "Comments");
        log_message!(
            "Showing {}/{} comment(s):",
            paginated_comments.len(),
            comments_data.comments.len()
        );

        for (idx, comment) in paginated_comments.iter().enumerate() {
            log_break!();
            log_message!("Comment #{}:", idx + 1 + start);
            if let Some(author) = &comment.author {
                log_message!(
                    "  Author: {} ({})",
                    author.display_name,
                    author.email_address.as_deref().unwrap_or("N/A")
                );
            }
            log_message!("  Created: {}", format_date(&comment.created)?);
            if let Some(updated) = &comment.updated {
                if updated != &comment.created {
                    log_message!("  Updated: {}", format_date(updated)?);
                }
            }
            log_message!("  Content:");
            // 每行添加缩进
            for line in comment.body.lines() {
                log_message!("    {}", line);
            }
        }

        Ok(())
    }

    /// JSON 格式输出
    fn output_json(comments: &Option<crate::jira::JiraComments>, _jira_id: &str) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(comments_data) = comments {
            output.insert("comments".to_string(), serde_json::to_value(comments_data)?);
        } else {
            output.insert("comments".to_string(), serde_json::json!([]));
        }

        log_message!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    /// YAML 格式输出
    fn output_yaml(comments: &Option<crate::jira::JiraComments>, _jira_id: &str) -> Result<()> {
        let mut output: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(comments_data) = comments {
            output.insert("comments".to_string(), serde_json::to_value(comments_data)?);
        } else {
            output.insert("comments".to_string(), serde_json::json!([]));
        }

        log_message!("{}", serde_saphyr::to_string(&output)?);
        Ok(())
    }

    /// Markdown 格式输出
    fn output_markdown(
        comments: &Option<crate::jira::JiraComments>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort: &str,
        author: Option<&str>,
        since: Option<&str>,
    ) -> Result<()> {
        let Some(comments_data) = comments else {
            log_message!("# Comments\n\nNo comments.\n");
            return Ok(());
        };

        let mut filtered_comments = comments_data.comments.clone();

        if let Some(author_email) = author {
            filtered_comments.retain(|c| {
                c.author
                    .as_ref()
                    .and_then(|a| a.email_address.as_ref())
                    .map(|e| e == author_email)
                    .unwrap_or(false)
            });
        }

        if let Some(since_date) = since {
            if let Ok(since_dt) = DateTime::<FixedOffset>::parse_from_rfc3339(since_date) {
                filtered_comments.retain(|c| {
                    DateTime::<FixedOffset>::parse_from_rfc3339(&c.created)
                        .map(|dt| dt >= since_dt)
                        .unwrap_or(false)
                });
            }
        }

        filtered_comments.sort_by(|a, b| {
            let a_time = DateTime::<FixedOffset>::parse_from_rfc3339(&a.created).ok();
            let b_time = DateTime::<FixedOffset>::parse_from_rfc3339(&b.created).ok();
            match (a_time, b_time) {
                (Some(a_dt), Some(b_dt)) => {
                    if sort == "asc" {
                        a_dt.cmp(&b_dt)
                    } else {
                        b_dt.cmp(&a_dt)
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        let start = offset.unwrap_or(0);
        let end = limit.map(|l| start + l).unwrap_or(filtered_comments.len());
        let paginated_comments =
            filtered_comments.into_iter().skip(start).take(end - start).collect::<Vec<_>>();

        log_message!("# Comments\n");

        for (idx, comment) in paginated_comments.iter().enumerate() {
            log_message!("## Comment #{}", idx + 1 + start);
            if let Some(author) = &comment.author {
                log_message!(
                    "**Author:** {} ({})\n",
                    author.display_name,
                    author.email_address.as_deref().unwrap_or("N/A")
                );
            }
            log_message!("**Created:** {}\n", format_date(&comment.created)?);
            log_message!("{}\n", comment.body);
        }

        Ok(())
    }
}
