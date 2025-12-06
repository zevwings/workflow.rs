//! Jira 辅助函数测试
//!
//! 测试 Jira 相关的辅助函数，包括项目名提取、ticket ID 提取、邮箱清理等。

use workflow::jira::helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
};

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
