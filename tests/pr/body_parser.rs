//! PR Body 解析器测试
//!
//! 测试从 PR body 中提取信息的功能。

use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::pr::body_parser::{
    extract_description_from_body, extract_info_from_source_pr, extract_jira_ticket_from_body,
    parse_change_types_from_body, SourcePrInfo,
};

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
fn test_extract_jira_ticket_from_body(#[case] body: &str, #[case] expected: Option<&str>) {
    let result = extract_jira_ticket_from_body(body);
    assert_eq!(result, expected.map(|s| s.to_string()));
}

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
fn test_extract_description_from_body(#[case] body: &str, #[case] expected: Option<&str>) {
    let result = extract_description_from_body(body);
    assert_eq!(result, expected.map(|s| s.to_string()));
}

#[test]
fn test_parse_change_types_from_body() {
    let body = r#"## Types of changes

- [x] Bug fix
- [ ] New feature
- [x] Breaking change"#;
    let types = parse_change_types_from_body(body);
    assert!(types.is_some());
    let types = types.unwrap();
    // 根据 TYPES_OF_CHANGES 的顺序检查
    assert!(types.len() > 0);
}

#[test]
fn test_extract_info_from_source_pr() {
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
    let extracted = extract_info_from_source_pr(&Some(source_pr));
    assert_eq!(extracted.jira_ticket, Some("PROJ-123".to_string()));
    assert_eq!(extracted.description, Some("Test description".to_string()));
    assert!(extracted.change_types.is_some());
}
