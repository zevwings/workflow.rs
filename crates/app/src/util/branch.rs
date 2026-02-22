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
/// ```
/// use app::util::branch::to_slug;
///
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
                // 跳过其他特殊字符
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // 移除多余的连字符
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // 清理结果
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
///
/// # Examples
///
/// ```ignore
/// // `strip_branch_type_prefix` 是内部辅助函数（非公有 API）。
/// use app::util::branch::strip_branch_type_prefix;
/// assert_eq!(strip_branch_type_prefix("feature/my-branch"), "my-branch");
/// assert_eq!(strip_branch_type_prefix("my-branch"), "my-branch");
/// assert_eq!(strip_branch_type_prefix("bugfix/fix-issue"), "fix-issue");
/// ```
pub fn strip_branch_type_prefix(name: &str) -> String {
    let prefixes = ["feature/", "bugfix/", "hotfix/", "refactoring/", "chore/"];

    for prefix in prefixes {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }

    name.to_string()
}
