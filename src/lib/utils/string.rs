//! 字符串处理工具函数

/// 从 Jira ticket 提取项目名
///
/// # 示例
/// ```
/// assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
/// assert_eq!(extract_jira_project("PROJ"), None);
/// ```
pub fn extract_jira_project(ticket: &str) -> Option<&str> {
    ticket.split('-').next().filter(|s| *s != ticket)
}

/// 将字符串转换为分支名（替换特殊字符为连字符，去除重复连字符）
pub fn to_branch_name(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_dash = false;

    for c in s.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            result.push('-');
            last_was_dash = true;
        }
    }

    result.trim_matches('-').to_string()
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
    fn test_to_branch_name() {
        assert_eq!(to_branch_name("Hello World!"), "hello-world");
        assert_eq!(to_branch_name("Bug fix #123"), "bug-fix-123");
    }
}

