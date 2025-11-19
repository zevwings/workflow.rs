//! Jira 辅助函数
//!
//! 本模块提供了 Jira 相关的辅助函数，包括：
//! - 字符串处理（提取项目名、提取 ticket ID、验证格式）
//! - 文件名处理（邮箱地址清理）
//! - 认证和 URL 构建（获取认证信息、构建基础 URL）

use crate::base::settings::Settings;
use anyhow::Result;
use regex::Regex;

/// 从 Jira ticket 提取项目名
///
/// # 示例
/// ```
/// use workflow::jira::helpers::extract_jira_project;
/// assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
/// assert_eq!(extract_jira_project("PROJ"), None);
/// ```
pub fn extract_jira_project(ticket: &str) -> Option<&str> {
    ticket.split('-').next().filter(|s| *s != ticket)
}

/// 验证 Jira ticket 格式
///
/// Jira ticket 应该是 PROJECT-123 格式（ticket），或纯项目名（PROJECT）。
/// 项目名只能包含字母、数字和下划线。
///
/// # 示例
/// ```
/// use workflow::jira::helpers::validate_jira_ticket_format;
/// assert!(validate_jira_ticket_format("PROJ-123").is_ok());
/// assert!(validate_jira_ticket_format("PROJ").is_ok());
/// assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
/// assert!(validate_jira_ticket_format("invalid/ticket").is_err());
/// ```
pub fn validate_jira_ticket_format(ticket: &str) -> Result<()> {
    let is_valid_format: bool = if let Some(project) = extract_jira_project(ticket) {
        // 如果是 ticket 格式（PROJ-123），检查项目名是否有效
        project
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    } else {
        // 如果是项目名格式，检查是否只包含有效字符
        ticket
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    };

    if !is_valid_format {
        anyhow::bail!(
            "Invalid Jira ticket format: '{}'. Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name).\n  - Ticket names should contain only letters, numbers, and hyphens\n  - Project names should contain only letters, numbers, and underscores\n  - Do not use branch names or paths (e.g., 'zw/修改打包脚本问题')",
            ticket
        );
    }

    Ok(())
}

/// 从 PR 标题提取 Jira ticket ID
///
/// # 示例
/// ```
/// use workflow::jira::helpers::extract_jira_ticket_id;
/// assert_eq!(extract_jira_ticket_id("PROJ-123: Fix bug"), Some("PROJ-123"));
/// assert_eq!(extract_jira_ticket_id("Fix bug"), None);
/// ```
pub fn extract_jira_ticket_id(pull_request_title: &str) -> Option<String> {
    // 匹配格式: PROJ-123 或 PROJ-123:
    let re = Regex::new(r"^([A-Z]+-\d+)").ok()?;
    re.captures(pull_request_title)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// 清理邮箱地址作为文件名（方案1：简单替换）
///
/// 将邮箱地址中的特殊字符替换为安全的文件名字符：
/// - `@` → `_at_`
/// - `.` → `_dot_`
/// - `+` → `_plus_`
///
/// # 示例
/// ```
/// use workflow::jira::helpers::sanitize_email_for_filename;
/// assert_eq!(sanitize_email_for_filename("user@example.com"), "user_at_example_dot_com");
/// assert_eq!(sanitize_email_for_filename("user+tag@example.com"), "user_plus_tag_at_example_dot_com");
/// ```
pub fn sanitize_email_for_filename(email: &str) -> String {
    email
        .replace('@', "_at_")
        .replace('.', "_dot_")
        .replace('+', "_plus_")
}

/// 获取认证信息
///
/// 从配置文件中读取 Jira API 认证所需的 email 和 api_token。
///
/// # 返回
///
/// 返回 `(email, api_token)` 元组。
pub fn get_auth() -> Result<(String, String)> {
    let settings = Settings::get();
    let email = settings.jira.email.clone().unwrap_or_default();
    let api_token = settings.jira.api_token.clone().unwrap_or_default();
    Ok((email, api_token))
}

/// 获取 Jira API 基础 URL
///
/// 从配置文件中读取 Jira 服务地址，并构建 REST API 基础 URL。
/// 格式：`{jira_service_address}/rest/api/2`
///
/// # 返回
///
/// 返回完整的 REST API 基础 URL。
///
/// # 错误
///
/// 如果 `jira_service_address` 未设置或为空，返回错误。
pub fn get_base_url() -> Result<String> {
    let settings = Settings::get();
    let base_url = settings.jira.service_address.clone().unwrap_or_default();

    if base_url.is_empty() {
        anyhow::bail!(
            "Jira service address is not configured. \
            Please run 'workflow setup' to configure it."
        );
    }

    Ok(format!("{}/rest/api/2", base_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_jira_project() {
        assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
        assert_eq!(extract_jira_project("PROJ"), None);
        assert_eq!(extract_jira_project("ABC-123-456"), Some("ABC"));
    }

    #[test]
    fn test_extract_jira_ticket_id() {
        assert_eq!(
            extract_jira_ticket_id("PROJ-123: Fix bug"),
            Some("PROJ-123".to_string())
        );
        assert_eq!(
            extract_jira_ticket_id("ABC-456 Description"),
            Some("ABC-456".to_string())
        );
        assert_eq!(extract_jira_ticket_id("Fix bug"), None);
    }

    #[test]
    fn test_sanitize_email_for_filename() {
        assert_eq!(
            sanitize_email_for_filename("user@example.com"),
            "user_at_example_dot_com"
        );
        assert_eq!(
            sanitize_email_for_filename("user+tag@example.com"),
            "user_plus_tag_at_example_dot_com"
        );
        assert_eq!(
            sanitize_email_for_filename("test.user@example.co.uk"),
            "test_dot_user_at_example_dot_co_dot_uk"
        );
    }
}
