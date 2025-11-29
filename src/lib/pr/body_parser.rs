//! PR Body 解析器
//!
//! 提供从 PR body 中提取信息的纯函数，无用户交互。
//! 这些函数可以被多个命令复用（如 pick, sync, rebase 等）。

use crate::jira::helpers::extract_jira_ticket_id;
use regex::Regex;

use super::platform::TYPES_OF_CHANGES;

/// 源 PR 信息
///
/// 包含从源 PR 获取的基本信息，用于提取和复用。
#[derive(Debug, Clone)]
pub struct SourcePrInfo {
    /// PR 标题
    pub title: Option<String>,
    /// PR URL
    pub url: Option<String>,
    /// PR body（完整内容）
    pub body: Option<String>,
}

/// 从源 PR 提取的信息
///
/// 包含从源 PR 中提取的所有可用信息，用于创建新 PR。
#[derive(Debug, Clone)]
pub struct ExtractedPrInfo {
    /// 从标题或 body 提取的 Jira ticket
    pub jira_ticket: Option<String>,
    /// 从 body 提取的描述
    pub description: Option<String>,
    /// 从 body 解析的变更类型
    pub change_types: Option<Vec<bool>>,
}

/// 从 PR body 中提取 Jira ticket ID
///
/// 从 `#### Jira Link:` 部分提取 ticket ID。
/// 格式：`{jira_service}/browse/{TICKET-ID}`
///
/// # 示例
///
/// ```
/// use workflow::pr::body_parser::extract_jira_ticket_from_body;
///
/// let body = r#"#### Jira Link:
///
/// https://jira.example.com/browse/PROJ-123"#;
/// assert_eq!(extract_jira_ticket_from_body(body), Some("PROJ-123".to_string()));
/// ```
pub fn extract_jira_ticket_from_body(body: &str) -> Option<String> {
    // 匹配格式：#### Jira Link:\n\n{url}/browse/{TICKET-ID}
    let re = Regex::new(r"(?i)####\s+Jira\s+Link:.*?\n\n([^\n]+)/browse/([A-Z]+-\d+)").ok()?;
    re.captures(body)
        .and_then(|caps| caps.get(2))
        .map(|m| m.as_str().to_string())
}

/// 从 PR body 中提取简短描述
///
/// 从 `#### Short description:` 部分提取描述内容。
///
/// # 示例
///
/// ```
/// use workflow::pr::body_parser::extract_description_from_body;
///
/// let body = r#"#### Short description:
///
/// This is a description"#;
/// assert_eq!(extract_description_from_body(body), Some("This is a description".to_string()));
/// ```
pub fn extract_description_from_body(body: &str) -> Option<String> {
    // 匹配格式：#### Short description:\n\n{description}\n
    let re = Regex::new(r"(?i)####\s+Short\s+description:.*?\n\n(.*?)(?:\n####|\n#|$)").ok()?;
    re.captures(body)
        .and_then(|caps| caps.get(1))
        .and_then(|m| {
            let desc = m.as_str().trim();
            if desc.is_empty() {
                None
            } else {
                Some(desc.to_string())
            }
        })
}

/// 从 PR body 中解析变更类型
///
/// 从 `## Types of changes` 部分解析复选框状态。
/// 返回一个布尔向量，表示每个类型是否被选中。
///
/// # 示例
///
/// ```
/// use workflow::pr::body_parser::parse_change_types_from_body;
///
/// let body = r#"## Types of changes
///
/// - [x] Bug fix
/// - [ ] New feature"#;
/// let types = parse_change_types_from_body(body);
/// assert!(types.is_some());
/// ```
pub fn parse_change_types_from_body(body: &str) -> Option<Vec<bool>> {
    let mut selected_types = Vec::new();

    // 查找 "## Types of changes" 部分
    let types_section = body
        .split("## Types of changes")
        .nth(1)?
        .split("\n####")
        .next()?;

    // 解析每个变更类型的复选框状态
    for change_type in TYPES_OF_CHANGES {
        // 匹配格式：- [x] 或 - [ ]
        let pattern = format!(r"- \[([x ])\]\s+{}", regex::escape(change_type));
        let re = Regex::new(&pattern).ok()?;
        let is_selected = re
            .captures(types_section)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim() == "x")
            .unwrap_or(false);
        selected_types.push(is_selected);
    }

    Some(selected_types)
}

/// 从源 PR 提取信息（Jira ticket、描述、变更类型）
///
/// 从 `SourcePrInfo` 中提取所有可用信息，包括：
/// - 从标题或 body 提取 Jira ticket
/// - 从 body 提取描述
/// - 从 body 解析变更类型
///
/// # 参数
///
/// * `source_pr_info` - 源 PR 信息
///
/// # 返回
///
/// 返回提取的信息结构体
///
/// # 示例
///
/// ```
/// use workflow::pr::body_parser::{SourcePrInfo, extract_info_from_source_pr};
///
/// let source_pr = SourcePrInfo {
///     title: Some("PROJ-123: Fix bug".to_string()),
///     url: None,
///     body: Some("#### Short description:\n\nDescription here".to_string()),
/// };
/// let extracted = extract_info_from_source_pr(&Some(source_pr));
/// assert_eq!(extracted.jira_ticket, Some("PROJ-123".to_string()));
/// ```
pub fn extract_info_from_source_pr(source_pr_info: &Option<SourcePrInfo>) -> ExtractedPrInfo {
    let mut jira_ticket = None;
    let mut description = None;
    let mut change_types = None;

    if let Some(info) = source_pr_info {
        // 1. 从标题提取 Jira ticket
        if let Some(ref title) = info.title {
            if let Some(ticket) = extract_jira_ticket_id(title) {
                jira_ticket = Some(ticket);
            }
        }

        // 2. 从 body 提取信息
        if let Some(ref body) = info.body {
            // 从 body 提取 Jira ticket（如果标题中没有）
            if jira_ticket.is_none() {
                if let Some(ticket) = extract_jira_ticket_from_body(body) {
                    jira_ticket = Some(ticket);
                }
            }

            // 从 body 提取描述
            if let Some(desc) = extract_description_from_body(body) {
                description = Some(desc);
            }

            // 从 body 解析变更类型
            if let Some(types) = parse_change_types_from_body(body) {
                change_types = Some(types);
            }
        }
    }

    ExtractedPrInfo {
        jira_ticket,
        description,
        change_types,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_jira_ticket_from_body() {
        let body = r#"#### Jira Link:

https://jira.example.com/browse/PROJ-123"#;
        assert_eq!(
            extract_jira_ticket_from_body(body),
            Some("PROJ-123".to_string())
        );

        let body = r#"#### Jira Link:

https://jira.example.com/browse/ABC-456"#;
        assert_eq!(
            extract_jira_ticket_from_body(body),
            Some("ABC-456".to_string())
        );

        let body = "No Jira link";
        assert_eq!(extract_jira_ticket_from_body(body), None);
    }

    #[test]
    fn test_extract_description_from_body() {
        let body = r#"#### Short description:

This is a test description"#;
        assert_eq!(
            extract_description_from_body(body),
            Some("This is a test description".to_string())
        );

        let body = r#"#### Short description:

"#;
        assert_eq!(extract_description_from_body(body), None);

        let body = "No description";
        assert_eq!(extract_description_from_body(body), None);
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
}
