//! Jira Helpers 模块测试
//!
//! 测试 Jira 辅助函数的核心功能，包括字符串处理、验证等。

use pretty_assertions::assert_eq;
use workflow::jira::helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format,
};

// ==================== Jira Project Extraction Tests ====================

/// 测试从Jira ticket中提取项目名
///
/// ## 测试目的
/// 验证 `extract_jira_project()` 函数能够从有效的Jira ticket ID中正确提取项目名。
///
/// ## 测试场景
/// 1. 测试有效的ticket ID（PROJ-123, PROJ-456, TEST-789）
/// 2. 测试无效输入（只有项目名、空字符串）
///
/// ## 预期结果
/// - 有效ticket ID返回项目名（PROJ, TEST等）
/// - 无效输入返回None
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

/// 测试从字符串中提取Jira ticket ID
///
/// ## 测试目的
/// 验证 `extract_jira_ticket_id()` 函数能够从包含ticket ID的字符串中正确提取ticket ID。
///
/// ## 测试场景
/// 1. 测试包含ticket ID的字符串（"PROJ-123: Fix bug", "PROJ-123"等）
/// 2. 测试不包含ticket ID的字符串（"Fix bug", ""）
///
/// ## 预期结果
/// - 包含ticket ID的字符串返回ticket ID（"PROJ-123", "PROJ-456"等）
/// - 不包含ticket ID的字符串返回None
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

/// 测试清理邮箱地址用于文件名
///
/// ## 测试目的
/// 验证 `sanitize_email_for_filename()` 函数能够将邮箱地址转换为适合文件名的格式（替换特殊字符）。
///
/// ## 测试场景
/// 1. 测试各种格式的邮箱地址（普通邮箱、带+号的邮箱、多级域名等）
/// 2. 测试空字符串
///
/// ## 预期结果
/// - 邮箱地址中的特殊字符被替换（@ -> _at_, . -> _dot_, + -> _plus_）
/// - 空字符串返回空字符串
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

/// 测试验证Jira ticket格式（有效格式）
///
/// ## 测试目的
/// 验证 `validate_jira_ticket_format()` 函数能够正确识别有效的Jira ticket格式。
///
/// ## 测试场景
/// 1. 测试各种有效格式（PROJ-123, PROJ, TEST-456, PROJ-123-456, PROJECT_123）
///
/// ## 预期结果
/// - 所有有效格式通过验证，返回Ok
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

/// 测试验证Jira ticket格式（无效格式）
///
/// ## 测试目的
/// 验证 `validate_jira_ticket_format()` 函数能够正确识别无效的Jira ticket格式并返回错误。
///
/// ## 测试场景
/// 1. 测试各种无效格式（空字符串、只有空格、PROJ-、包含斜杠、包含字母等）
///
/// ## 预期结果
/// - 所有无效格式返回错误
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

/// 测试验证Jira ticket格式（边界情况）
///
/// ## 测试目的
/// 验证 `validate_jira_ticket_format()` 函数能够正确处理边界情况的ticket格式。
///
/// ## 测试场景
/// 1. 测试边界情况（最短格式A-1、长数字PROJECT-999999、下划线格式PROJ_123、多段格式PROJ-123-456-789）
///
/// ## 预期结果
/// - 所有边界情况通过验证，返回Ok
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

