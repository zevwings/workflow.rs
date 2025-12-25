//! Jira Helpers 模块测试
//!
//! 测试 Jira 辅助函数的核心功能，包括字符串处理、验证等。

use pretty_assertions::assert_eq;
use workflow::jira::helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format,
};

// ==================== Jira Project Extraction Tests ====================

#[test]
fn test_extract_jira_project_with_valid_ticket_returns_project() {
    // Arrange: 准备有效的 Jira ticket ID
    let tickets = ["PROJ-123", "PROJ-456", "TEST-789"];

    // Act & Assert: 验证提取项目名正确
    assert_eq!(extract_jira_project(tickets[0]), Some("PROJ"));
    assert_eq!(extract_jira_project(tickets[1]), Some("PROJ"));
    assert_eq!(extract_jira_project(tickets[2]), Some("TEST"));
    assert_eq!(extract_jira_project("PROJ"), None);
    assert_eq!(extract_jira_project(""), None);
}

// ==================== Jira Ticket ID Extraction Tests ====================

#[test]
fn test_extract_jira_ticket_id_with_valid_strings_returns_ticket_id() {
    // Arrange: 准备包含 ticket ID 的字符串
    let inputs = [
        "PROJ-123: Fix bug",
        "PROJ-123",
        "PROJ-456: Add feature",
    ];

    // Act & Assert: 验证提取 ticket ID 正确
    assert_eq!(
        extract_jira_ticket_id(inputs[0]),
        Some("PROJ-123".to_string())
    );
    assert_eq!(
        extract_jira_ticket_id(inputs[1]),
        Some("PROJ-123".to_string())
    );
    assert_eq!(
        extract_jira_ticket_id(inputs[2]),
        Some("PROJ-456".to_string())
    );
    assert_eq!(extract_jira_ticket_id("Fix bug"), None);
    assert_eq!(extract_jira_ticket_id(""), None);
}

// ==================== Email Sanitization Tests ====================

#[test]
fn test_sanitize_email_for_filename_with_various_emails_returns_sanitized() {
    // Arrange: 准备各种格式的邮箱地址
    let emails = [
        ("user@example.com", "user_at_example_dot_com"),
        ("user+tag@example.com", "user_plus_tag_at_example_dot_com"),
        ("test.user@example.co.uk", "test_dot_user_at_example_dot_co_dot_uk"),
        ("", ""),
    ];

    // Act & Assert: 验证邮箱地址被正确清理
    for (email, expected) in emails.iter() {
        assert_eq!(sanitize_email_for_filename(email), *expected);
    }
}

// ==================== Jira Ticket Format Validation Tests ====================

#[test]
fn test_validate_jira_ticket_format_with_valid_formats_returns_ok() {
    // Arrange: 准备有效的 ticket 格式
    let valid_tickets = ["PROJ-123", "PROJ", "TEST-456", "PROJ-123-456", "PROJECT_123"];

    // Act & Assert: 验证所有有效格式通过验证
    for ticket in valid_tickets.iter() {
        assert!(
            validate_jira_ticket_format(ticket).is_ok(),
            "Ticket '{}' should be valid",
            ticket
        );
    }
}

#[test]
fn test_validate_jira_ticket_format_with_invalid_formats_returns_error() {
    // Arrange: 准备无效的 ticket 格式
    let invalid_tickets = ["", "   ", "PROJ-", "invalid/ticket", "PROJ-abc"];

    // Act & Assert: 验证所有无效格式返回错误
    for ticket in invalid_tickets.iter() {
        assert!(
            validate_jira_ticket_format(ticket).is_err(),
            "Ticket '{}' should be invalid",
            ticket
        );
    }
}

#[test]
fn test_validate_jira_ticket_format_edge_cases_with_edge_cases_returns_ok() {
    // Arrange: 准备边界情况的 ticket 格式
    let edge_cases = ["A-1", "PROJECT-999999", "PROJ_123", "PROJ-123-456-789"];

    // Act & Assert: 验证边界情况通过验证
    for ticket in edge_cases.iter() {
        assert!(
            validate_jira_ticket_format(ticket).is_ok(),
            "Edge case ticket '{}' should be valid",
            ticket
        );
    }
}

