//! Jira Ticket/Issue 相关 API
//!
//! 本模块提供了 Ticket/Issue 的完整操作功能：
//! - 获取 ticket 信息（包括 summary、description、status、attachments、comments）
//! - 更新 ticket 状态（通过 transitions）
//! - 分配 ticket 给用户
//! - 添加评论到 ticket
//! - 获取 ticket 的附件列表

use anyhow::{Context, Result};
use regex::Regex;

use super::api::issue::JiraIssueApi;
use super::types::{JiraAttachment, JiraIssue, JiraTransition};

/// Jira Ticket/Issue 操作
///
/// 提供 Ticket/Issue 的完整操作功能，包括：
/// - 获取 ticket 信息
/// - 更新 ticket 状态
/// - 分配 ticket 给用户
/// - 添加评论
/// - 获取附件列表
pub struct JiraTicket;

impl JiraTicket {
    /// 获取 ticket 信息
    ///
    /// 从 Jira API 获取指定 ticket 的完整信息，包括：
    /// - 基本信息（key、id、summary、description）
    /// - 状态信息
    /// - 附件列表
    /// - 评论列表
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回 `JiraIssue` 结构体，包含 ticket 的所有信息。
    pub fn get_info(ticket: &str) -> Result<JiraIssue> {
        JiraIssueApi::get_issue(ticket)
            .context(format!("Failed to get ticket info for: {}", ticket))
    }

    /// 从 URL 中提取附件 ID
    ///
    /// 从 CloudFront URL 中提取附件 ID，格式如：/attachments/bugs/16232306/.../21886523/log0.txt
    /// 附件 ID 通常是路径中的数字部分。
    pub fn extract_attachment_id_from_url(url: &str) -> Option<String> {
        // 匹配 URL 中的附件 ID，通常在路径中，如 /attachments/.../21886523/...
        let id_pattern = Regex::new(r"/attachments/[^/]+/\d+/([^/]+/)*(\d+)/").unwrap();
        if let Some(cap) = id_pattern.captures(url) {
            if let Some(id_match) = cap.get(2) {
                return Some(id_match.as_str().to_string());
            }
        }
        None
    }

    /// 从描述中解析附件链接
    ///
    /// Jira 描述中可能包含附件链接，格式为：`# [filename|url]`
    /// 解析这些链接并返回附件列表。
    fn parse_attachments_from_description(description: &str) -> Vec<JiraAttachment> {
        let mut attachments = Vec::new();
        let link_pattern = Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();

        for cap in link_pattern.captures_iter(description) {
            if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
                let filename = filename_match.as_str().trim().to_string();
                let url = url_match.as_str().trim().to_string();

                if url.contains("attachments")
                    || filename.ends_with(".txt")
                    || filename.ends_with(".log")
                    || filename.ends_with(".zip")
                {
                    attachments.push(JiraAttachment {
                        filename,
                        content_url: url,
                        mime_type: None,
                        size: None,
                    });
                }
            }
        }

        attachments
    }

    /// 获取 ticket 的附件列表
    ///
    /// 会从 `fields.attachment` 获取附件，同时也会从描述中解析附件链接。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回附件列表，如果没有附件则返回空列表。
    pub fn get_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
        let issue = Self::get_info(ticket)?;
        let mut attachments = issue.fields.attachment.unwrap_or_default();

        if let Some(description) = &issue.fields.description {
            for desc_att in Self::parse_attachments_from_description(description) {
                if !attachments.iter().any(|a| a.filename == desc_att.filename) {
                    attachments.push(desc_att);
                }
            }
        }

        Ok(attachments)
    }

    /// 获取 issue 的可用 transitions
    ///
    /// 获取指定 ticket 当前可以执行的所有状态转换。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回可用的 transitions 列表，每个 transition 包含 ID 和名称。
    fn get_transitions(ticket: &str) -> Result<Vec<JiraTransition>> {
        JiraIssueApi::get_issue_transitions(ticket)
            .context(format!("Failed to get transitions for ticket: {}", ticket))
    }

    /// 更新 ticket 状态
    ///
    /// 将 ticket 的状态更新为指定的状态。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `status` - 目标状态名称，如 `"In Progress"`、`"Done"` 等
    ///
    /// # 错误
    ///
    /// 如果指定的状态不在可用 transitions 列表中，返回错误。
    pub fn transition(ticket: &str, status: &str) -> Result<()> {
        let transitions = Self::get_transitions(ticket)?;
        let transition =
            transitions
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(status))
                .context(format!(
                    "Status '{}' not found in available transitions for ticket {}",
                    status, ticket
                ))?;

        JiraIssueApi::transition_issue(ticket, &transition.id).context(format!(
            "Failed to move ticket {} to status {}",
            ticket, status
        ))
    }

    /// 分配 ticket 给用户
    ///
    /// 将 ticket 分配给指定的用户。如果 `assignee` 为 `None`，则分配给当前用户。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `assignee` - 被分配用户的 account_id，如果为 `None` 则分配给当前用户
    pub fn assign(ticket: &str, assignee: Option<&str>) -> Result<()> {
        let assignee_account_id = match assignee {
            Some(user) => user.to_string(),
            None => {
                // 如果没有指定，分配给当前用户
                super::users::JiraUsers::get()?.account_id
            }
        };

        JiraIssueApi::assign_issue(ticket, &assignee_account_id).context(format!(
            "Failed to assign ticket {} to {}",
            ticket, assignee_account_id
        ))
    }

    /// 添加评论到 ticket
    ///
    /// 在指定的 ticket 上添加一条评论。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `comment` - 评论内容
    pub fn add_comment(ticket: &str, comment: &str) -> Result<()> {
        JiraIssueApi::add_issue_comment(ticket, comment)
            .context(format!("Failed to add comment to ticket {}", ticket))
    }
}
