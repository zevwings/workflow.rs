//! Pull Request 实体

use serde::{Deserialize, Serialize};

/// PR 变更类型结构体
///
/// 包含变更类型的完整信息，包括名称、描述和示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeType {
    /// 变更类型名称（用于显示和匹配）
    pub name: &'static str,
    /// 详细描述
    pub description: &'static str,
    /// 使用示例
    pub example: &'static str,
}

/// PR 变更类型定义
pub const CHANGE_TYPES: &[ChangeType] = &[
    ChangeType {
        name: "Bug fix (non-breaking change which fixes an issue)",
        description: "Fix errors or issues in code without changing existing functionality interfaces or behavior",
        example: "Fix null pointer exception in login validation logic",
    },
    ChangeType {
        name: "New feature (non-breaking change which adds functionality)",
        description: "Add new features or capabilities without affecting existing functionality",
        example: "Add user avatar upload functionality",
    },
    ChangeType {
        name: "Refactoring (non-breaking change which does not change functionality)",
        description: "Restructure code to improve quality without changing functional behavior",
        example: "Extract duplicate code into common functions and optimize code structure",
    },
    ChangeType {
        name: "Hotfix (urgent fix for production issues)",
        description: "Urgent fix for critical production issues that require immediate deployment",
        example: "Fix critical security vulnerability in authentication system",
    },
    ChangeType {
        name: "Chore (maintenance tasks and non-functional changes)",
        description: "Maintenance tasks, dependency updates, configuration changes, or other non-functional improvements",
        example: "Update dependencies, improve build configuration, or update documentation",
    },
];

/// PR 状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestStatus {
    /// PR 状态（如 "open", "closed", "merged"）
    pub state: String,
    /// 是否已合并
    pub merged: bool,
    /// 合并时间（如果已合并）
    pub merged_at: Option<String>,
}

/// Pull Request 信息
#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub id: String,
    pub title: String,
    pub body: String,
    pub status: PullRequestStatus,
    pub source_branch: String,
    pub target_branch: String,
}

/// PR 内容
#[derive(Debug, Clone)]
pub struct PrContent {
    pub title: String,
    pub description: String,
}

/// 获取所有变更类型的完整信息
pub fn get_all_change_types() -> &'static [ChangeType] {
    CHANGE_TYPES
}

/// 根据索引获取变更类型信息
pub fn get_change_type_by_index(index: usize) -> Option<&'static ChangeType> {
    CHANGE_TYPES.get(index)
}

/// 根据名称查找变更类型信息
pub fn get_change_type_by_name(name: &str) -> Option<&'static ChangeType> {
    CHANGE_TYPES.iter().find(|ct| ct.name == name)
}
