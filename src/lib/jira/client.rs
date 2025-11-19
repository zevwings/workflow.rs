//! Jira REST API 客户端（向后兼容包装器）
//!
//! 本模块提供了 `JiraClient` 结构体，作为向后兼容的包装器。
//! 所有方法都委托到对应的功能模块：
//! - 用户相关方法 → `users` 模块
//! - Ticket/Issue 相关方法 → `ticket` 模块
//! - 项目状态相关方法 → `status` 模块（`get_project_statuses`）

use anyhow::Result;

use super::models::{JiraAttachment, JiraIssue, JiraUser};
use super::ticket::JiraTicket;
use super::users::JiraUsers;

/// Jira REST API 客户端
///
/// 这是一个向后兼容的包装器，所有方法都委托到对应的功能模块。
/// 建议新代码直接使用各功能模块的函数，而不是通过 `JiraClient`。
pub struct JiraClient;

impl JiraClient {
    /// 获取当前 Jira 用户信息
    ///
    /// 先检查本地缓存（`${HOME}/.workflow/users/${email}.json`），
    /// 如果缓存不存在或读取失败，则从 Jira API 获取并保存到本地。
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，包含用户的 `account_id`、`display_name` 和 `email_address`。
    pub fn get_user_info() -> Result<JiraUser> {
        JiraUsers::get()
    }

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
    pub fn get_ticket_info(ticket: &str) -> Result<JiraIssue> {
        JiraTicket::get_info(ticket)
    }

    /// 获取 ticket 的附件列表
    ///
    /// 用于日志下载等功能。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    ///
    /// # 返回
    ///
    /// 返回附件列表，如果没有附件则返回空列表。
    pub fn get_attachments(ticket: &str) -> Result<Vec<JiraAttachment>> {
        JiraTicket::get_attachments(ticket)
    }

    /// 更新 ticket 状态
    ///
    /// 将 ticket 的状态更新为指定的状态。
    /// 会先获取 ticket 的可用 transitions，然后查找匹配的状态进行转换。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `status` - 目标状态名称，如 `"In Progress"`、`"Done"` 等
    pub fn move_ticket(ticket: &str, status: &str) -> Result<()> {
        JiraTicket::transition(ticket, status)
    }

    /// 分配 ticket 给用户
    ///
    /// 将 ticket 分配给指定的用户。如果 `assignee` 为 `None`，则分配给当前用户。
    ///
    /// # 参数
    ///
    /// * `ticket` - Jira ticket ID，格式如 `PROJ-123`
    /// * `assignee` - 被分配用户的 account_id，如果为 `None` 则分配给当前用户
    pub fn assign_ticket(ticket: &str, assignee: Option<&str>) -> Result<()> {
        JiraTicket::assign(ticket, assignee)
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
        JiraTicket::add_comment(ticket, comment)
    }
}
