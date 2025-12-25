//! PR Body 解析器测试
//!
//! 测试从 PR body 中提取信息的功能。

use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::pr::body_parser::{
    extract_description_from_body, extract_info_from_source_pr, extract_jira_ticket_from_body,
    parse_change_types_from_body, SourcePrInfo,
};

// ==================== Jira Ticket Extraction Tests ====================

#[rstest]
#[case(
    r#"#### Jira Link:

https://jira.example.com/browse/PROJ-123"#,
    Some("PROJ-123")
)]
#[case(
    r#"#### Jira Link:

https://jira.example.com/browse/ABC-456"#,
    Some("ABC-456")
)]
#[case("No Jira link", None)]
#[case("", None)]
fn test_extract_jira_ticket_from_body_with_various_bodies_returns_ticket(#[case] body: &str, #[case] expected: Option<&str>) {
    // Arrange: 准备 PR body 文本和预期结果

    // Act: 从 body 中提取 Jira ticket
    let result = extract_jira_ticket_from_body(body);

    // Assert: 验证提取结果与预期一致
    assert_eq!(result, expected.map(|s| s.to_string()));
}

// ==================== Description Extraction Tests ====================

#[rstest]
#[case(
    r#"#### Short description:

This is a test description"#,
    Some("This is a test description")
)]
#[case(
    r#"#### Short description:

"#,
    None
)]
#[case("No description", None)]
#[case("", None)]
fn test_extract_description_from_body_with_various_bodies_returns_description(#[case] body: &str, #[case] expected: Option<&str>) {
    // Arrange: 准备 PR body 文本和预期结果

    // Act: 从 body 中提取描述
    let result = extract_description_from_body(body);

    // Assert: 验证提取结果与预期一致
    assert_eq!(result, expected.map(|s| s.to_string()));
}

// ==================== Change Types Parsing Tests ====================

#[test]
fn test_parse_change_types_from_body_with_markdown_list_returns_types() {
    // Arrange: 准备包含变更类型的 PR body
    let body = r#"## Types of changes

- [x] Bug fix
- [ ] New feature
- [x] Breaking change"#;

    // Act: 解析变更类型
    let types = parse_change_types_from_body(body);

    // Assert: 验证解析成功且包含类型
    assert!(types.is_some());
    let types = types.expect("change types should be parsed");
    assert!(types.len() > 0);
}

// ==================== Source PR Info Extraction Tests ====================

#[test]
fn test_extract_info_from_source_pr_with_complete_info_returns_extracted_info() {
    // Arrange: 准备包含完整信息的 SourcePrInfo
    let source_pr = SourcePrInfo {
        title: Some("PROJ-123: Fix bug".to_string()),
        url: None,
        body: Some(
            r#"#### Short description:

Test description
## Types of changes

- [x] Bug fix"#
                .to_string(),
        ),
    };

    // Act: 从 SourcePrInfo 中提取信息
    let extracted = extract_info_from_source_pr(&Some(source_pr));

    // Assert: 验证提取的信息正确
    assert_eq!(extracted.jira_ticket, Some("PROJ-123".to_string()));
    assert_eq!(extracted.description, Some("Test description".to_string()));
    assert!(extracted.change_types.is_some());
}
