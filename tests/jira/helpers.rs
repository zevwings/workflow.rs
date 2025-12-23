//! Jira Helpers 模块测试
//!
//! 测试 Jira 辅助函数的核心功能，包括字符串处理、验证等。

use pretty_assertions::assert_eq;
use workflow::jira::helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format,
};

#[test]
fn test_extract_jira_project() {
    assert_eq!(extract_jira_project("PROJ-123"), Some("PROJ"));
    assert_eq!(extract_jira_project("PROJ-456"), Some("PROJ"));
    assert_eq!(extract_jira_project("TEST-789"), Some("TEST"));
    assert_eq!(extract_jira_project("PROJ"), None);
    assert_eq!(extract_jira_project(""), None);
}

#[test]
fn test_extract_jira_ticket_id() {
    assert_eq!(
        extract_jira_ticket_id("PROJ-123: Fix bug"),
        Some("PROJ-123".to_string())
    );
    assert_eq!(
        extract_jira_ticket_id("PROJ-123"),
        Some("PROJ-123".to_string())
    );
    assert_eq!(extract_jira_ticket_id("Fix bug"), None);
    assert_eq!(extract_jira_ticket_id(""), None);
    assert_eq!(
        extract_jira_ticket_id("PROJ-456: Add feature"),
        Some("PROJ-456".to_string())
    );
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
    assert_eq!(sanitize_email_for_filename(""), "");
}

#[test]
fn test_validate_jira_ticket_format_valid() {
    assert!(validate_jira_ticket_format("PROJ-123").is_ok());
    assert!(validate_jira_ticket_format("PROJ").is_ok());
    assert!(validate_jira_ticket_format("TEST-456").is_ok());
    assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
    assert!(validate_jira_ticket_format("PROJECT_123").is_ok());
}

#[test]
fn test_validate_jira_ticket_format_invalid() {
    assert!(validate_jira_ticket_format("").is_err());
    assert!(validate_jira_ticket_format("   ").is_err());
    assert!(validate_jira_ticket_format("PROJ-").is_err());
    assert!(validate_jira_ticket_format("invalid/ticket").is_err());
    assert!(validate_jira_ticket_format("PROJ-abc").is_err());
}

#[test]
fn test_validate_jira_ticket_format_edge_cases() {
    // 测试边界情况
    assert!(validate_jira_ticket_format("A-1").is_ok());
    assert!(validate_jira_ticket_format("PROJECT-999999").is_ok());
    assert!(validate_jira_ticket_format("PROJ_123").is_ok());
    assert!(validate_jira_ticket_format("PROJ-123-456-789").is_ok());
}

