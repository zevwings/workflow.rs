//! Jira 相关模块
//! 包含 Jira API 客户端、状态配置等功能

pub mod api;
pub mod commands;
pub mod status;

// 重新导出所有公共 API，保持向后兼容
pub use api::{extract_jira_ticket_id, JiraApi, JiraAttachment};
pub use commands::Jira;
pub use status::*;

// 为了向后兼容，在 Jira 结构体上添加 REST API 方法的便捷访问
impl Jira {
    /// 获取项目状态列表（通过 REST API）
    pub fn get_project_statuses(project: &str) -> anyhow::Result<Vec<String>> {
        JiraApi::get_project_statuses(project)
    }

    /// 从 Jira REST API 获取 issue summary
    pub fn get_summary(ticket: &str) -> anyhow::Result<String> {
        JiraApi::get_summary(ticket)
    }

    /// 下载附件（用于日志下载功能）
    pub fn get_attachments(ticket: &str) -> anyhow::Result<Vec<JiraAttachment>> {
        JiraApi::get_attachments(ticket)
    }
}

// 为了向后兼容，保留旧的模块路径别名
// 注意：这些别名用于 `config::`、`jira::` 等使用场景
#[allow(deprecated)]
pub use status as config;
