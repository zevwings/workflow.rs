//! 分支管理实体

use serde::{Deserialize, Serialize};
use std::fmt;

/// 分支类型枚举
///
/// 表示工作流中不同类型的分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchType {
    /// Feature branch - for new features
    Feature,
    /// Bugfix branch - for bug fixes
    Bugfix,
    /// Refactoring branch - for code refactoring
    Refactoring,
    /// Hotfix branch - for urgent production fixes
    Hotfix,
    /// Chore branch - for maintenance tasks
    Chore,
}

impl BranchType {
    /// Get all available branch types
    pub fn all() -> Vec<BranchType> {
        vec![
            BranchType::Feature,
            BranchType::Bugfix,
            BranchType::Refactoring,
            BranchType::Hotfix,
            BranchType::Chore,
        ]
    }

    /// Get branch type as string (for template selection)
    pub fn as_str(&self) -> &'static str {
        match self {
            BranchType::Feature => "feature",
            BranchType::Bugfix => "bugfix",
            BranchType::Refactoring => "refactoring",
            BranchType::Hotfix => "hotfix",
            BranchType::Chore => "chore",
        }
    }

    /// Get Conventional Commits commit type from branch type
    pub fn to_commit_type(&self) -> &'static str {
        match self {
            BranchType::Feature => "feat",
            BranchType::Bugfix => "fix",
            BranchType::Refactoring => "refactor",
            BranchType::Hotfix => "fix",
            BranchType::Chore => "chore",
        }
    }

    /// Parse branch type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "feature" => Some(BranchType::Feature),
            "bugfix" | "bug" | "fix" => Some(BranchType::Bugfix),
            "refactoring" | "refactor" => Some(BranchType::Refactoring),
            "hotfix" => Some(BranchType::Hotfix),
            "chore" => Some(BranchType::Chore),
            _ => None,
        }
    }
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 清理分支名，移除无效字符
///
/// 只保留 ASCII 字母、数字、连字符、下划线和斜杠。
/// 移除其他所有特殊字符，确保分支名符合 Git 规范。
///
/// # 参数
///
/// * `name` - 要清理的分支名
///
/// # 返回
///
/// 返回清理后的分支名。
///
/// # 示例
///
/// ```
/// use domain::branch::sanitize_branch_name;
///
/// assert_eq!(sanitize_branch_name("feature/abc-123"), "feature/abc-123");
/// assert_eq!(sanitize_branch_name("feature@test#123"), "featuretest123");
/// assert_eq!(sanitize_branch_name("  feature/test  "), "feature/test");
/// ```
pub fn sanitize_branch_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '/')
        .collect()
}

/// 同步策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStrategy {
    /// Merge strategy
    Merge,
    /// Rebase strategy
    Rebase,
    /// Fast-forward only
    FastForwardOnly,
    /// Squash merge
    Squash,
}

/// 源分支信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBranchInfo {
    /// 源分支名称
    pub name: String,
    /// 是否存在
    pub exists: bool,
}

/// 分支同步选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSyncOptions {
    /// 同步策略
    pub strategy: SyncStrategy,
    /// 是否强制推送
    pub force: bool,
}

/// 分支同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSyncResult {
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 分支同步回调接口
pub trait BranchSyncCallbacks {
    /// 同步开始回调
    fn on_start(&self);
    /// 同步完成回调
    fn on_complete(&self, result: &BranchSyncResult);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // BranchType 测试
    // ========================================================================

    #[test]
    fn test_branch_type_all() {
        let all = BranchType::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&BranchType::Feature));
        assert!(all.contains(&BranchType::Bugfix));
        assert!(all.contains(&BranchType::Refactoring));
        assert!(all.contains(&BranchType::Hotfix));
        assert!(all.contains(&BranchType::Chore));
    }

    #[test]
    fn test_branch_type_as_str() {
        assert_eq!(BranchType::Feature.as_str(), "feature");
        assert_eq!(BranchType::Bugfix.as_str(), "bugfix");
        assert_eq!(BranchType::Refactoring.as_str(), "refactoring");
        assert_eq!(BranchType::Hotfix.as_str(), "hotfix");
        assert_eq!(BranchType::Chore.as_str(), "chore");
    }

    #[test]
    fn test_branch_type_to_commit_type() {
        assert_eq!(BranchType::Feature.to_commit_type(), "feat");
        assert_eq!(BranchType::Bugfix.to_commit_type(), "fix");
        assert_eq!(BranchType::Refactoring.to_commit_type(), "refactor");
        assert_eq!(BranchType::Hotfix.to_commit_type(), "fix");
        assert_eq!(BranchType::Chore.to_commit_type(), "chore");
    }

    #[test]
    fn test_branch_type_parse_exact() {
        assert_eq!(BranchType::parse("feature"), Some(BranchType::Feature));
        assert_eq!(BranchType::parse("bugfix"), Some(BranchType::Bugfix));
        assert_eq!(BranchType::parse("refactoring"), Some(BranchType::Refactoring));
        assert_eq!(BranchType::parse("hotfix"), Some(BranchType::Hotfix));
        assert_eq!(BranchType::parse("chore"), Some(BranchType::Chore));
    }

    #[test]
    fn test_branch_type_parse_aliases() {
        // bugfix 别名
        assert_eq!(BranchType::parse("bug"), Some(BranchType::Bugfix));
        assert_eq!(BranchType::parse("fix"), Some(BranchType::Bugfix));

        // refactoring 别名
        assert_eq!(BranchType::parse("refactor"), Some(BranchType::Refactoring));
    }

    #[test]
    fn test_branch_type_parse_case_insensitive() {
        assert_eq!(BranchType::parse("FEATURE"), Some(BranchType::Feature));
        assert_eq!(BranchType::parse("Feature"), Some(BranchType::Feature));
        assert_eq!(BranchType::parse("BUGFIX"), Some(BranchType::Bugfix));
        assert_eq!(BranchType::parse("BUG"), Some(BranchType::Bugfix));
    }

    #[test]
    fn test_branch_type_parse_invalid() {
        assert_eq!(BranchType::parse("invalid"), None);
        assert_eq!(BranchType::parse(""), None);
        assert_eq!(BranchType::parse("feat"), None);
        assert_eq!(BranchType::parse("features"), None);
    }

    #[test]
    fn test_branch_type_display() {
        assert_eq!(format!("{}", BranchType::Feature), "feature");
        assert_eq!(format!("{}", BranchType::Bugfix), "bugfix");
        assert_eq!(format!("{}", BranchType::Refactoring), "refactoring");
        assert_eq!(format!("{}", BranchType::Hotfix), "hotfix");
        assert_eq!(format!("{}", BranchType::Chore), "chore");
    }

    // ========================================================================
    // sanitize_branch_name 测试
    // ========================================================================

    #[test]
    fn test_sanitize_branch_name_valid() {
        // 有效的分支名应该保持不变
        assert_eq!(sanitize_branch_name("feature/abc-123"), "feature/abc-123");
        assert_eq!(sanitize_branch_name("bugfix/fix_issue"), "bugfix/fix_issue");
        assert_eq!(sanitize_branch_name("main"), "main");
        // 注意：点号会被移除，因为不在允许的字符列表中
        assert_eq!(sanitize_branch_name("release-100"), "release-100");
    }

    #[test]
    fn test_sanitize_branch_name_removes_special_chars() {
        // 移除特殊字符
        assert_eq!(sanitize_branch_name("feature@test#123"), "featuretest123");
        assert_eq!(sanitize_branch_name("branch!name"), "branchname");
        assert_eq!(sanitize_branch_name("test$branch%name"), "testbranchname");
        assert_eq!(sanitize_branch_name("a&b*c"), "abc");
    }

    #[test]
    fn test_sanitize_branch_name_removes_spaces() {
        // 移除空格
        assert_eq!(sanitize_branch_name("  feature/test  "), "feature/test");
        assert_eq!(sanitize_branch_name("feature test"), "featuretest");
        assert_eq!(sanitize_branch_name("  "), "");
    }

    #[test]
    fn test_sanitize_branch_name_preserves_allowed_chars() {
        // 保留允许的字符：字母、数字、连字符、下划线、斜杠
        assert_eq!(sanitize_branch_name("a-b_c/d"), "a-b_c/d");
        assert_eq!(sanitize_branch_name("ABC-123_xyz/test"), "ABC-123_xyz/test");
    }

    #[test]
    fn test_sanitize_branch_name_empty() {
        assert_eq!(sanitize_branch_name(""), "");
    }

    #[test]
    fn test_sanitize_branch_name_unicode() {
        // 移除非 ASCII 字符
        assert_eq!(sanitize_branch_name("feature/中文"), "feature/");
        assert_eq!(sanitize_branch_name("修改bug"), "bug");
        assert_eq!(sanitize_branch_name("日本語"), "");
    }
}
