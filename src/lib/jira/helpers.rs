//! Jira 辅助函数

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

