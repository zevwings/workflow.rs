//! Jira 辅助函数

use anyhow::Result;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_jira_project() {
        assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
        assert_eq!(extract_jira_project("PROJ"), None);
        assert_eq!(extract_jira_project("ABC-123-456"), Some("ABC"));
    }
}
