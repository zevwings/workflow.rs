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

/// 分支命名工具
pub struct BranchNaming;

impl BranchNaming {
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
    /// use domain::branch::entity::BranchNaming;
    ///
    /// assert_eq!(BranchNaming::sanitize("feature/abc-123"), "feature/abc-123");
    /// assert_eq!(BranchNaming::sanitize("feature@test#123"), "featuretest123");
    /// assert_eq!(BranchNaming::sanitize("  feature/test  "), "feature/test");
    /// ```
    pub fn sanitize(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '/')
            .collect()
    }
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

/// 分支同步器
///
/// 提供分支同步相关的业务逻辑。
/// 当前功能通过服务层直接调用仓储实现，此实体为未来业务逻辑封装预留。
pub struct BranchSync;
