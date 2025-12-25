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

/// 测试从PR body中提取Jira ticket（参数化测试）
///
/// ## 测试目的
/// 验证 `extract_jira_ticket_from_body()` 函数能够从各种格式的PR body中正确提取Jira ticket ID。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下情况：
/// - 包含Jira链接的body（PROJ-123）
/// - 包含Jira链接的body（ABC-456）
/// - 不包含Jira链接的body
/// - 空body
///
/// ## 预期结果
/// - 包含Jira链接时，正确提取ticket ID
/// - 不包含Jira链接时，返回None
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
fn test_extract_jira_ticket_from_body_various_cases(#[case] body: &str, #[case] expected: Option<&str>) {
    // Arrange: 准备 PR body 文本和预期结果

    // Act: 从 body 中提取 Jira ticket
    let result = extract_jira_ticket_from_body(body);

    // Assert: 验证提取结果与预期一致
    assert_eq!(result, expected.map(|s| s.to_string()));
}

// ==================== Description Extraction Tests ====================

/// 测试从PR body中提取描述（参数化测试）
///
/// ## 测试目的
/// 验证 `extract_description_from_body()` 函数能够从各种格式的PR body中正确提取描述文本。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下情况：
/// - 包含描述的body
/// - 包含描述标题但内容为空的body
/// - 不包含描述的body
/// - 空body
///
/// ## 预期结果
/// - 包含描述时，正确提取描述文本
/// - 不包含描述或描述为空时，返回None
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
fn test_extract_description_from_body_various_cases(#[case] body: &str, #[case] expected: Option<&str>) {
    // Arrange: 准备 PR body 文本和预期结果

    // Act: 从 body 中提取描述
    let result = extract_description_from_body(body);

    // Assert: 验证提取结果与预期一致
    assert_eq!(result, expected.map(|s| s.to_string()));
}

// ==================== Change Types Parsing Tests ====================

/// 测试从PR body中解析变更类型
///
/// ## 测试目的
/// 验证 `parse_change_types_from_body()` 函数能够从Markdown格式的PR body中正确解析变更类型列表。
///
/// ## 测试场景
/// 1. 准备包含变更类型的PR body（Markdown列表格式）
/// 2. 解析变更类型
/// 3. 验证解析结果
///
/// ## 预期结果
/// - 解析成功，返回Some
/// - 变更类型列表不为空
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

/// 测试从SourcePrInfo中提取完整信息
///
/// ## 测试目的
/// 验证 `extract_info_from_source_pr()` 函数能够从 `SourcePrInfo` 结构体中提取所有信息（Jira ticket, description, change types）。
///
/// ## 测试场景
/// 1. 准备包含完整信息的SourcePrInfo（title, body）
/// 2. 从SourcePrInfo中提取信息
/// 3. 验证提取的信息正确
///
/// ## 预期结果
/// - Jira ticket正确提取（从title中）
/// - description正确提取（从body中）
/// - change_types正确解析（从body中）
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
