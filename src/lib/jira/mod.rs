//! Jira 相关模块
//! 包含 Jira API 客户端、状态配置等功能

pub mod client;
pub mod helpers;
pub mod status;

// 重新导出所有公共 API，保持向后兼容
pub use client::{extract_jira_ticket_id, JiraAttachment, JiraClient};
pub use helpers::{extract_jira_project, validate_jira_ticket_format};
pub use status::*;

/// Jira 客户端（向后兼容别名）
pub type Jira = JiraClient;

/// Jira API（向后兼容别名）
pub type JiraApi = JiraClient;

// 为了向后兼容，保留旧的模块路径别名
// 注意：这些别名用于 `config::`、`jira::` 等使用场景
#[allow(deprecated)]
pub use status as config;
