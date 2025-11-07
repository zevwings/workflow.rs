//! Jira 相关模块
//!
//! 本模块提供了与 Jira REST API 交互的完整功能，包括：
//! - 用户信息管理（获取、缓存）
//! - Ticket/Issue 操作（查询、状态更新、分配、评论）
//! - 项目状态管理（获取状态列表、配置状态映射）
//! - 数据模型定义（Issue、User、Attachment 等）
//! - 辅助工具函数（字符串处理、认证、URL 构建）
//!
//! ## 模块结构
//!
//! - `client` - JiraClient 包装器，提供向后兼容的 API
//! - `users` - 用户相关 API（获取用户信息、本地缓存）
//! - `ticket` - Ticket/Issue 相关 API（查询、更新、分配、评论）
//! - `status` - 状态管理（项目状态获取、状态配置、工作历史）
//! - `models` - 数据模型定义
//! - `helpers` - 辅助函数（字符串处理、认证、URL 构建）

pub mod client;
pub mod helpers;
pub mod models;
pub mod status;
pub mod ticket;
pub mod users;

// 重新导出所有公共 API，保持向后兼容
pub use client::JiraClient;
pub use models::{
    JiraComments, JiraAttachment, JiraComment, JiraIssue, JiraIssueFields,
    JiraStatus, JiraTransition, JiraUser,
};
pub use helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format,
};
pub use status::*;

/// Jira 客户端（向后兼容别名）
pub type Jira = JiraClient;

/// Jira API（向后兼容别名）
pub type JiraApi = JiraClient;

// 为了向后兼容，保留旧的模块路径别名
// 注意：这些别名用于 `config::`、`jira::` 等使用场景
#[allow(deprecated)]
pub use status as config;
