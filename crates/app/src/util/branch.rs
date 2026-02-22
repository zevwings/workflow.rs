use domain::{sanitize_branch_name, BranchType};

/// 将 JIRA summary 转换为 URL 友好的 slug 格式
///
/// # Arguments
///
/// * `summary` - JIRA ticket 摘要
///
/// # Returns
///
/// Slug 格式的字符串（小写、连字符分隔、只包含 ASCII 字符）
///
/// # Examples
///
/// ```ignore
/// assert_eq!(to_slug("Chat Unified Entry"), "chat-unified-entry");
/// assert_eq!(to_slug("Fix: Auth Issue"), "fix-auth-issue");
/// ```
pub fn to_slug(summary: impl AsRef<str>) -> String {
    let slug = summary
        .as_ref()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    sanitize_branch_name(&slug)
}

/// 从分支名解析分支类型
///
/// 按 `/` 分割分支名，依次尝试每个片段与 `BranchType::parse` 匹配（如 `feature`、`bugfix`）。
/// 适用于 `feature/xxx`、`zw/feature/xxx` 等格式。
pub fn branch_type_from_branch_name(branch_name: &str) -> Option<BranchType> {
    for segment in branch_name.split('/') {
        if let Some(bt) = BranchType::parse(segment) {
            return Some(bt);
        }
    }
    None
}

/// 移除分支名中可能存在的类型前缀
///
/// 防御性处理：如果 LLM 返回了带类型前缀的分支名，移除它。
pub fn strip_branch_type_prefix(name: &str) -> String {
    let prefixes = ["feature/", "bugfix/", "hotfix/", "refactoring/", "chore/"];

    for prefix in prefixes {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }

    name.to_string()
}
