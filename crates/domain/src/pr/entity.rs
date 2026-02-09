//! Pull Request 实体

use serde::{Deserialize, Serialize};

use crate::branch::BranchType;

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

/// 将分支类型映射到 PR 变更类型索引
///
/// 索引与 `CHANGE_TYPES` 顺序一致：0=Bug fix, 1=New feature, 2=Refactoring, 3=Hotfix, 4=Chore
pub fn get_change_type_index_by_branch_type(branch_type: BranchType) -> Option<usize> {
    match branch_type {
        BranchType::Feature => Some(1),
        BranchType::Bugfix => Some(0),
        BranchType::Refactoring => Some(2),
        BranchType::Hotfix => Some(3),
        BranchType::Chore => Some(4),
    }
}

/// 将分支类型映射为 PR 变更类型勾选向量
///
/// 返回长度为 `CHANGE_TYPES.len()` 的布尔向量，对应分支类型的那一项为 true
pub fn get_change_types_by_branch_type(branch_type: BranchType) -> Vec<bool> {
    let mut result = vec![false; CHANGE_TYPES.len()];
    if let Some(index) = get_change_type_index_by_branch_type(branch_type) {
        result[index] = true;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // CHANGE_TYPES 常量测试
    // ========================================================================

    #[test]
    fn test_change_types_count() {
        assert_eq!(CHANGE_TYPES.len(), 5);
    }

    #[test]
    fn test_change_types_have_required_fields() {
        for ct in CHANGE_TYPES {
            assert!(!ct.name.is_empty());
            assert!(!ct.description.is_empty());
            assert!(!ct.example.is_empty());
        }
    }

    // ========================================================================
    // get_all_change_types 测试
    // ========================================================================

    #[test]
    fn test_get_all_change_types() {
        let types = get_all_change_types();
        assert_eq!(types.len(), 5);
        // 验证返回的内容与常量相同
        for (i, ct) in types.iter().enumerate() {
            assert_eq!(ct.name, CHANGE_TYPES[i].name);
        }
    }

    #[test]
    fn test_get_all_change_types_contains_expected() {
        let types = get_all_change_types();
        let names: Vec<&str> = types.iter().map(|ct| ct.name).collect();

        assert!(names.iter().any(|n| n.contains("Bug fix")));
        assert!(names.iter().any(|n| n.contains("New feature")));
        assert!(names.iter().any(|n| n.contains("Refactoring")));
        assert!(names.iter().any(|n| n.contains("Hotfix")));
        assert!(names.iter().any(|n| n.contains("Chore")));
    }

    // ========================================================================
    // get_change_type_by_index 测试
    // ========================================================================

    #[test]
    fn test_get_change_type_by_index_valid() {
        let ct = get_change_type_by_index(0);
        assert!(ct.is_some());
        assert!(ct.unwrap().name.contains("Bug fix"));

        let ct = get_change_type_by_index(1);
        assert!(ct.is_some());
        assert!(ct.unwrap().name.contains("New feature"));

        let ct = get_change_type_by_index(4);
        assert!(ct.is_some());
        assert!(ct.unwrap().name.contains("Chore"));
    }

    #[test]
    fn test_get_change_type_by_index_out_of_bounds() {
        assert!(get_change_type_by_index(5).is_none());
        assert!(get_change_type_by_index(100).is_none());
        assert!(get_change_type_by_index(usize::MAX).is_none());
    }

    // ========================================================================
    // get_change_type_by_name 测试
    // ========================================================================

    #[test]
    fn test_get_change_type_by_name_exact_match() {
        let ct = get_change_type_by_name("Bug fix (non-breaking change which fixes an issue)");
        assert!(ct.is_some());
        assert!(ct.unwrap().name.contains("Bug fix"));
    }

    #[test]
    fn test_get_change_type_by_name_not_found() {
        assert!(get_change_type_by_name("Invalid Type").is_none());
        assert!(get_change_type_by_name("").is_none());
        assert!(get_change_type_by_name("bug fix").is_none()); // 区分大小写
    }

    #[test]
    fn test_get_change_type_by_name_partial_no_match() {
        // 部分匹配不应该成功
        assert!(get_change_type_by_name("Bug fix").is_none());
        assert!(get_change_type_by_name("New feature").is_none());
    }

    // ========================================================================
    // map_branch_type_to_change_type_index 测试
    // ========================================================================

    #[test]
    fn test_map_branch_type_to_change_type_index() {
        use crate::branch::BranchType;

        assert_eq!(
            get_change_type_index_by_branch_type(BranchType::Feature),
            Some(1)
        );
        assert_eq!(
            get_change_type_index_by_branch_type(BranchType::Bugfix),
            Some(0)
        );
        assert_eq!(
            get_change_type_index_by_branch_type(BranchType::Refactoring),
            Some(2)
        );
        assert_eq!(
            get_change_type_index_by_branch_type(BranchType::Hotfix),
            Some(3)
        );
        assert_eq!(
            get_change_type_index_by_branch_type(BranchType::Chore),
            Some(4)
        );
    }

    // ========================================================================
    // map_branch_type_to_change_types 测试
    // ========================================================================

    #[test]
    fn test_map_branch_type_to_change_types() {
        use crate::branch::BranchType;

        let feature = get_change_types_by_branch_type(BranchType::Feature);
        assert_eq!(feature.len(), 5);
        assert!(!feature[0]);
        assert!(feature[1]);
        assert!(!feature[2]);
        assert!(!feature[3]);
        assert!(!feature[4]);

        let bugfix = get_change_types_by_branch_type(BranchType::Bugfix);
        assert!(bugfix[0]);
        assert!(!bugfix[1]);
    }
}
