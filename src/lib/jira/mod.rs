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
//! - `status` - 状态管理（项目状态获取、状态配置）
//! - `history` - 工作历史记录管理（PR 创建/合并记录）
//! - `types` - 数据模型定义
//! - `helpers` - 辅助函数（字符串处理、认证、URL 构建）

pub mod api;
pub mod client;
pub mod config;
pub mod helpers;
pub mod history;
pub mod logs;
pub mod status;
pub mod ticket;
pub mod types;
pub mod users;

// 重新导出所有公共 API，保持向后兼容
pub use api::{JiraIssueApi, JiraProjectApi, JiraUserApi};
pub use client::JiraClient;
pub use config::ConfigManager;
pub use helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format,
};
pub use logs::{JiraLogs, LogEntry};
// 导出 types 模块的类型
// 注意：types::JiraStatus 是数据模型（用于序列化），status::JiraStatus 是管理结构体
// 如果需要访问数据模型，请使用 jira::types::JiraStatus
pub use history::{JiraWorkHistory, WorkHistoryEntry};
pub use status::{JiraStatus, JiraStatusConfig, ProjectStatusConfig};
pub use types::{
    JiraAttachment, JiraComment, JiraComments, JiraIssue, JiraIssueFields, JiraTransition, JiraUser,
};

/// Jira 客户端（向后兼容别名）
pub type Jira = JiraClient;

/// Jira API（向后兼容别名）
pub type JiraApi = JiraClient;
