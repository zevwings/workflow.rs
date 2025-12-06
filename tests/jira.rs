//! Jira 模块测试
//!
//! 测试 `jira` 模块中的 Jira 客户端和 API 功能。
//!
//! 注意：这些测试主要测试辅助函数和数据结构，实际的 API 调用需要 mock 或集成测试。

use workflow::jira::helpers::{
    extract_jira_project, extract_jira_ticket_id, sanitize_email_for_filename,
    validate_jira_ticket_format,
};

// ==================== 辅助函数测试（扩展） ====================

#[test]
fn test_validate_jira_ticket_format_valid() {
    // 测试有效的 ticket 格式
    assert!(validate_jira_ticket_format("PROJ-123").is_ok());
    assert!(validate_jira_ticket_format("PROJ").is_ok());
    assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
    assert!(validate_jira_ticket_format("ABC123-456").is_ok());
    assert!(validate_jira_ticket_format("PROJECT_NAME-123").is_ok());
    assert!(validate_jira_ticket_format("PROJ_123-456").is_ok());
}

#[test]
fn test_validate_jira_ticket_format_invalid() {
    // 测试无效的 ticket 格式
    assert!(validate_jira_ticket_format("invalid/ticket").is_err());
    assert!(validate_jira_ticket_format("PROJ-").is_err());
    assert!(validate_jira_ticket_format("-123").is_err());
    assert!(validate_jira_ticket_format("PROJ@123").is_err());
    assert!(validate_jira_ticket_format("PROJ#123").is_err());
    assert!(validate_jira_ticket_format("PROJ 123").is_err());
}

#[test]
fn test_validate_jira_ticket_format_edge_cases() {
    // 测试边界情况
    assert!(validate_jira_ticket_format("").is_err());
    assert!(validate_jira_ticket_format("A-1").is_ok()); // 最短有效格式
    assert!(validate_jira_ticket_format("A1B2C3-123456").is_ok());
    assert!(validate_jira_ticket_format("PROJECT_NAME_WITH_UNDERSCORES-123").is_ok());
}

#[test]
fn test_extract_jira_project_edge_cases() {
    // 测试边界情况
    assert_eq!(extract_jira_project(""), None);
    assert_eq!(extract_jira_project("A"), None);
    assert_eq!(extract_jira_project("A-"), Some("A"));
    assert_eq!(extract_jira_project("ABC-DEF-GHI"), Some("ABC"));
}

#[test]
fn test_extract_jira_ticket_id_variations() {
    // 测试各种格式的 ticket ID 提取
    assert_eq!(
        extract_jira_ticket_id("PROJ-123: Fix bug"),
        Some("PROJ-123".to_string())
    );
    assert_eq!(
        extract_jira_ticket_id("PROJ-123 Fix bug"),
        Some("PROJ-123".to_string())
    );
    assert_eq!(
        extract_jira_ticket_id("PROJ-123"),
        Some("PROJ-123".to_string())
    );
    assert_eq!(extract_jira_ticket_id("Fix bug"), None);
    assert_eq!(extract_jira_ticket_id(""), None);
    assert_eq!(
        extract_jira_ticket_id("ABC-456 Description"),
        Some("ABC-456".to_string())
    );
}

#[test]
fn test_extract_jira_ticket_id_with_numbers() {
    // 测试包含数字的项目名
    assert_eq!(
        extract_jira_ticket_id("PROJ123-456: Title"),
        Some("PROJ123-456".to_string())
    );
    assert_eq!(
        extract_jira_ticket_id("ABC123-789 Description"),
        Some("ABC123-789".to_string())
    );
}

#[test]
fn test_sanitize_email_for_filename_variations() {
    // 测试各种邮箱格式
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
    assert_eq!(
        sanitize_email_for_filename("user_name@sub.example.com"),
        "user_name_at_sub_dot_example_dot_com"
    );
}

#[test]
fn test_sanitize_email_for_filename_edge_cases() {
    // 测试边界情况
    assert_eq!(sanitize_email_for_filename(""), "");
    assert_eq!(sanitize_email_for_filename("@"), "_at_");
    assert_eq!(sanitize_email_for_filename("user@"), "user_at_");
    assert_eq!(
        sanitize_email_for_filename("@example.com"),
        "_at_example_dot_com"
    );
}

// ==================== 数据结构测试 ====================

#[test]
fn test_jira_ticket_format_consistency() {
    // 测试 ticket 格式的一致性
    let tickets = vec!["PROJ-123", "ABC-456", "PROJECT-789"];
    for ticket in tickets {
        assert!(validate_jira_ticket_format(ticket).is_ok());
        let project = extract_jira_project(ticket);
        assert!(project.is_some());
    }
}

#[test]
fn test_jira_project_extraction_round_trip() {
    // 测试项目名提取的往返一致性
    let ticket = "PROJ-123";
    let project = extract_jira_project(ticket).unwrap();
    assert_eq!(project, "PROJ");

    // 验证可以重新构建 ticket
    let reconstructed = format!("{}-123", project);
    assert_eq!(reconstructed, ticket);
}

// ==================== 错误处理测试 ====================

#[test]
fn test_validate_jira_ticket_format_error_message() {
    // 测试错误消息的格式
    let result = validate_jira_ticket_format("invalid/ticket");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid Jira ticket format"));
    assert!(error_msg.contains("invalid/ticket"));
}

// ==================== 集成测试场景 ====================

#[test]
fn test_jira_workflow_extraction() {
    // 测试典型的 Jira 工作流：从 PR 标题提取 ticket
    let pr_titles = vec![
        "PROJ-123: Fix critical bug",
        "ABC-456 Implement new feature",
        "PROJECT-789: Update documentation",
    ];

    for title in pr_titles {
        let ticket_id = extract_jira_ticket_id(title);
        assert!(ticket_id.is_some());

        let ticket = ticket_id.unwrap();
        assert!(validate_jira_ticket_format(&ticket).is_ok());

        let project = extract_jira_project(&ticket);
        assert!(project.is_some());
    }
}

#[test]
fn test_jira_email_sanitization_workflow() {
    // 测试邮箱清理的完整工作流
    let emails = vec![
        "user@example.com",
        "developer+test@company.co.uk",
        "admin.user@subdomain.example.org",
    ];

    for email in emails {
        let sanitized = sanitize_email_for_filename(email);
        // 验证清理后的字符串不包含特殊字符
        assert!(!sanitized.contains('@'));
        assert!(!sanitized.contains('.'));
        assert!(!sanitized.contains('+'));
        // 验证可以安全地用作文件名
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('\\'));
    }
}
