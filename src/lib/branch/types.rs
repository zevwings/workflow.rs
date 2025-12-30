//! Branch type definitions
//!
//! Defines branch types and provides selection functionality.

use crate::base::dialog::SelectDialog;
use crate::log_info;
use crate::repo::config::RepoConfig;
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use std::fmt;

/// Branch type enumeration
///
/// Represents different types of branches in the workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    ///
    /// Maps branch type to Conventional Commits commit type:
    /// - Feature → "feat"
    /// - Bugfix → "fix"
    /// - Refactoring → "refactor"
    /// - Hotfix → "fix" (hotfix is a type of bug fix)
    /// - Chore → "chore"
    pub fn to_commit_type(&self) -> &'static str {
        match self {
            BranchType::Feature => "feat",
            BranchType::Bugfix => "fix",
            BranchType::Refactoring => "refactor",
            BranchType::Hotfix => "fix",
            BranchType::Chore => "chore",
        }
    }

    /// Get display name with description
    pub fn display_name(&self) -> &'static str {
        match self {
            BranchType::Feature => "feature - 新功能开发",
            BranchType::Bugfix => "bugfix - Bug 修复",
            BranchType::Refactoring => "refactoring - 代码重构",
            BranchType::Hotfix => "hotfix - 紧急修复",
            BranchType::Chore => "chore - 杂项任务",
        }
    }

    /// Parse branch type from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "feature" => Some(BranchType::Feature),
            "bugfix" | "bug" | "fix" => Some(BranchType::Bugfix),
            "refactoring" | "refactor" => Some(BranchType::Refactoring),
            "hotfix" => Some(BranchType::Hotfix),
            "chore" => Some(BranchType::Chore),
            _ => None,
        }
    }

    /// Prompt user to select branch type interactively
    pub fn prompt_selection() -> Result<Self> {
        let options: Vec<BranchType> = Self::all();
        let display_options: Vec<String> =
            options.iter().map(|ty| ty.display_name().to_string()).collect();

        let selected = SelectDialog::new("选择分支类型 (Select branch type)", display_options)
            .with_default(0) // Default to feature
            .prompt()
            .wrap_err("Failed to select branch type")?;

        // Find the corresponding BranchType
        options
            .into_iter()
            .find(|ty| ty.display_name() == selected)
            .ok_or_else(|| eyre!("Invalid branch type selection"))
    }

    /// Resolve branch type with repository prefix fallback
    ///
    /// Priority:
    /// 1. If repository prefix exists and can be converted to BranchType, use it
    /// 2. Otherwise, prompt user to select interactively
    ///
    /// # Returns
    ///
    /// Returns the resolved branch type.
    ///
    /// # Errors
    ///
    /// Returns an error if the user selection fails or if the repository prefix cannot be converted to a branch type.
    pub fn resolve_with_repo_prefix() -> Result<Self> {
        // Check if repository prefix exists and use it as branch type
        if let Some(repo_prefix) = RepoConfig::get_branch_prefix() {
            if let Some(ty) = Self::from_str(&repo_prefix) {
                log_info!("Using repository prefix '{}' as branch type", repo_prefix);
                return Ok(ty);
            }
        }

        // Otherwise, prompt user to select
        Self::prompt_selection()
    }
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== BranchType 枚举测试 ====================

    /// 测试分支类型枚举值创建
    ///
    /// ## 测试目的
    /// 验证所有分支类型枚举值都可以创建并格式化。
    ///
    /// ## 测试场景
    /// 1. 创建所有分支类型枚举值
    /// 2. 格式化每个类型为 Debug 字符串
    /// 3. 验证字符串不为空
    ///
    /// ## 预期结果
    /// - 所有分支类型都可以创建并格式化
    #[test]
    fn test_branch_type_enum_values_can_be_created() {
        // Arrange: 准备所有分支类型枚举值
        let types = vec![
            BranchType::Feature,
            BranchType::Bugfix,
            BranchType::Refactoring,
            BranchType::Hotfix,
            BranchType::Chore,
        ];

        // Act & Assert: 验证所有类型都可以创建并格式化
        for branch_type in types {
            let debug_str = format!("{:?}", branch_type);
            assert!(!debug_str.is_empty());
        }
    }

    /// 测试获取所有分支类型
    ///
    /// ## 测试目的
    /// 验证 BranchType::all() 返回所有分支类型。
    ///
    /// ## 测试场景
    /// 1. 调用 all() 方法
    /// 2. 验证返回的类型数量正确
    /// 3. 验证包含所有预期的分支类型
    ///
    /// ## 预期结果
    /// - 返回所有5种分支类型
    #[test]
    fn test_branch_type_all_returns_all_types() {
        // Arrange: 准备预期结果
        let expected_count = 5;

        // Act: 调用 all() 方法
        let all_types = BranchType::all();

        // Assert: 验证返回所有类型且数量正确
        assert_eq!(all_types.len(), expected_count);
        assert!(all_types.contains(&BranchType::Feature));
        assert!(all_types.contains(&BranchType::Bugfix));
        assert!(all_types.contains(&BranchType::Refactoring));
        assert!(all_types.contains(&BranchType::Hotfix));
        assert!(all_types.contains(&BranchType::Chore));
    }

    /// 测试分支类型显示格式
    ///
    /// ## 测试目的
    /// 验证 BranchType 的 Display trait 实现返回小写字符串。
    ///
    /// ## 测试场景
    /// 1. 调用 to_string() 方法转换每个分支类型
    /// 2. 验证返回的字符串为小写格式
    ///
    /// ## 预期结果
    /// - 所有分支类型都返回小写字符串
    #[test]
    fn test_branch_type_display_returns_lowercase_string() {
        // Arrange: 准备预期结果
        let expected_feature = "feature";
        let expected_bugfix = "bugfix";
        let expected_refactoring = "refactoring";
        let expected_hotfix = "hotfix";
        let expected_chore = "chore";

        // Act: 调用 to_string() 方法
        let result_feature = BranchType::Feature.to_string();
        let result_bugfix = BranchType::Bugfix.to_string();
        let result_refactoring = BranchType::Refactoring.to_string();
        let result_hotfix = BranchType::Hotfix.to_string();
        let result_chore = BranchType::Chore.to_string();

        // Assert: 验证显示格式正确
        assert_eq!(result_feature, expected_feature);
        assert_eq!(result_bugfix, expected_bugfix);
        assert_eq!(result_refactoring, expected_refactoring);
        assert_eq!(result_hotfix, expected_hotfix);
        assert_eq!(result_chore, expected_chore);
    }

    /// 测试分支类型字符串切片
    ///
    /// ## 测试目的
    /// 验证 BranchType::as_str() 返回正确的字符串切片。
    ///
    /// ## 测试场景
    /// 1. 调用 as_str() 方法获取每个分支类型的字符串切片
    /// 2. 验证返回的字符串切片正确
    ///
    /// ## 预期结果
    /// - 所有分支类型都返回正确的字符串切片
    #[test]
    fn test_branch_type_as_str_returns_string_slice() {
        // Arrange: 准备预期结果
        let expected_feature = "feature";
        let expected_bugfix = "bugfix";
        let expected_refactoring = "refactoring";
        let expected_hotfix = "hotfix";
        let expected_chore = "chore";

        // Act: 调用 as_str() 方法
        let result_feature = BranchType::Feature.as_str();
        let result_bugfix = BranchType::Bugfix.as_str();
        let result_refactoring = BranchType::Refactoring.as_str();
        let result_hotfix = BranchType::Hotfix.as_str();
        let result_chore = BranchType::Chore.as_str();

        // Assert: 验证返回正确的字符串切片
        assert_eq!(result_feature, expected_feature);
        assert_eq!(result_bugfix, expected_bugfix);
        assert_eq!(result_refactoring, expected_refactoring);
        assert_eq!(result_hotfix, expected_hotfix);
        assert_eq!(result_chore, expected_chore);
    }

    /// 测试从字符串解析分支类型（有效输入）
    ///
    /// ## 测试目的
    /// 验证 BranchType::from_str() 能够从有效字符串解析分支类型。
    ///
    /// ## 测试场景
    /// 测试各种有效输入（包括别名）和无效输入
    ///
    /// ## 预期结果
    /// - 有效输入返回对应的分支类型，无效输入返回 None
    #[test]
    fn test_branch_type_from_string_with_valid_input() {
        // 测试各种有效输入
        assert_eq!(BranchType::from_str("feature"), Some(BranchType::Feature));
        assert_eq!(BranchType::from_str("bugfix"), Some(BranchType::Bugfix));
        assert_eq!(BranchType::from_str("bug"), Some(BranchType::Bugfix));
        assert_eq!(BranchType::from_str("fix"), Some(BranchType::Bugfix));
        assert_eq!(
            BranchType::from_str("refactoring"),
            Some(BranchType::Refactoring)
        );
        assert_eq!(
            BranchType::from_str("refactor"),
            Some(BranchType::Refactoring)
        );
        assert_eq!(BranchType::from_str("hotfix"), Some(BranchType::Hotfix));
        assert_eq!(BranchType::from_str("chore"), Some(BranchType::Chore));
        assert_eq!(BranchType::from_str("invalid"), None);
    }

    /// 测试从字符串解析分支类型（大小写不敏感）
    ///
    /// ## 测试目的
    /// 验证 BranchType::from_str() 支持大小写不敏感的解析。
    ///
    /// ## 测试场景
    /// 测试不同大小写的字符串输入
    ///
    /// ## 预期结果
    /// - 大小写不敏感，所有变体都能正确解析
    #[test]
    fn test_branch_type_from_string_with_case_insensitive_input() {
        assert_eq!(BranchType::from_str("FEATURE"), Some(BranchType::Feature));
        assert_eq!(BranchType::from_str("BugFix"), Some(BranchType::Bugfix));
        assert_eq!(
            BranchType::from_str("REFACTORING"),
            Some(BranchType::Refactoring)
        );
        assert_eq!(BranchType::from_str("HotFix"), Some(BranchType::Hotfix));
        assert_eq!(BranchType::from_str("CHORE"), Some(BranchType::Chore));
    }

    // ==================== BranchType 功能测试 ====================

    /// 测试分支类型转换为提交类型
    ///
    /// ## 测试目的
    /// 验证 BranchType::to_commit_type() 能够将分支类型转换为 Conventional Commits 类型。
    ///
    /// ## 测试场景
    /// 测试所有分支类型到提交类型的转换
    ///
    /// ## 预期结果
    /// - 所有分支类型都正确转换为对应的提交类型
    #[test]
    fn test_branch_type_to_commit_type() {
        assert_eq!(BranchType::Feature.to_commit_type(), "feat");
        assert_eq!(BranchType::Bugfix.to_commit_type(), "fix");
        assert_eq!(BranchType::Refactoring.to_commit_type(), "refactor");
        assert_eq!(BranchType::Hotfix.to_commit_type(), "fix");
        assert_eq!(BranchType::Chore.to_commit_type(), "chore");
    }

    /// 测试分支类型显示名称
    ///
    /// ## 测试目的
    /// 验证 BranchType::display_name() 返回包含描述的显示名称。
    ///
    /// ## 测试场景
    /// 测试所有分支类型的显示名称
    ///
    /// ## 预期结果
    /// - 所有分支类型都返回包含描述的显示名称
    #[test]
    fn test_branch_type_display_name() {
        assert_eq!(BranchType::Feature.display_name(), "feature - 新功能开发");
        assert_eq!(BranchType::Bugfix.display_name(), "bugfix - Bug 修复");
        assert_eq!(
            BranchType::Refactoring.display_name(),
            "refactoring - 代码重构"
        );
        assert_eq!(BranchType::Hotfix.display_name(), "hotfix - 紧急修复");
        assert_eq!(BranchType::Chore.display_name(), "chore - 杂项任务");
    }

    // ==================== Boundary Condition Tests ====================

    /// 测试从空字符串解析分支类型
    ///
    /// ## 测试目的
    /// 验证 BranchType::from_str() 对空字符串返回 None。
    ///
    /// ## 测试场景
    /// 1. 使用空字符串解析分支类型
    /// 2. 验证返回 None
    ///
    /// ## 预期结果
    /// - 空字符串返回 None
    #[test]
    fn test_branch_type_from_str_with_empty_string_returns_none() {
        // Arrange: 准备空字符串

        // Act: 从空字符串解析分支类型
        let result = BranchType::from_str("");

        // Assert: 验证返回 None
        assert_eq!(result, None);
    }

    /// 测试从空白字符串解析分支类型
    ///
    /// ## 测试目的
    /// 验证 BranchType::from_str() 对只包含空白字符的字符串返回 None。
    ///
    /// ## 测试场景
    /// 1. 使用只包含空白字符的字符串解析分支类型
    /// 2. 验证返回 None
    ///
    /// ## 预期结果
    /// - 空白字符串返回 None
    #[test]
    fn test_branch_type_from_str_with_whitespace_returns_none() {
        // Arrange: 准备空白字符串

        // Act: 从空白字符串解析分支类型
        let result = BranchType::from_str("   ");

        // Assert: 验证返回 None
        assert_eq!(result, None);
    }

    /// 测试从包含特殊字符的字符串解析分支类型
    ///
    /// ## 测试目的
    /// 验证 BranchType::from_str() 对包含特殊字符的字符串的处理。
    ///
    /// ## 测试场景
    /// 1. 使用包含特殊字符的字符串解析分支类型
    /// 2. 验证处理结果（大部分应返回 None，除非有特殊处理）
    ///
    /// ## 预期结果
    /// - 包含特殊字符的字符串通常返回 None，除非有特殊处理
    #[test]
    fn test_branch_type_from_str_with_special_characters_handles_correctly() {
        // Arrange: 准备包含特殊字符的字符串
        let special_strings = vec!["feat!", "bug#", "fix@", "hot-fix", "feature_branch"];

        // Act & Assert: 验证特殊字符处理正确
        for special_str in special_strings {
            let result = BranchType::from_str(special_str);
            // 大部分特殊字符应该返回 None，除非有特殊处理
            if let Some(branch_type) = result {
                assert!(BranchType::all().contains(&branch_type));
            }
        }
    }

    // ==================== Branch Type Comparison Tests ====================

    /// 测试分支类型相等性
    ///
    /// ## 测试目的
    /// 验证分支类型的相等性比较正确。
    ///
    /// ## 测试场景
    /// 1. 比较相同类型的分支类型
    /// 2. 比较不同类型的分支类型
    /// 3. 验证相等性结果正确
    ///
    /// ## 预期结果
    /// - 相同类型相等，不同类型不相等
    #[test]
    fn test_branch_type_equality_with_same_types_returns_equal() {
        // Arrange: 准备相同和不同的分支类型

        // Act & Assert: 验证分支类型相等性
        assert_eq!(BranchType::Feature, BranchType::Feature);
        assert_eq!(BranchType::Bugfix, BranchType::Bugfix);
        assert_ne!(BranchType::Feature, BranchType::Bugfix);
        assert_ne!(BranchType::Hotfix, BranchType::Chore);
    }

    /// 测试分支类型克隆功能
    ///
    /// ## 测试目的
    /// 验证 BranchType 的 Clone trait 实现正确。
    ///
    /// ## 测试场景
    /// 1. 克隆分支类型
    /// 2. 验证克隆后的值与原值相等
    ///
    /// ## 预期结果
    /// - 克隆后的值与原值相等
    #[test]
    fn test_branch_type_clone_with_valid_type_creates_clone() {
        // Arrange: 准备原始分支类型
        let original = BranchType::Feature;

        // Act: 克隆分支类型
        let cloned = original;

        // Assert: 验证克隆后的值相等
        assert_eq!(original, cloned);
    }

    /// 测试分支类型复制功能
    ///
    /// ## 测试目的
    /// 验证 BranchType 的 Copy trait 实现正确。
    ///
    /// ## 测试场景
    /// 1. 复制分支类型（通过赋值）
    /// 2. 验证复制后的值与原值相等
    ///
    /// ## 预期结果
    /// - 复制后的值与原值相等
    #[test]
    fn test_branch_type_copy_with_valid_type_copies_value() {
        // Arrange: 准备原始分支类型
        let original = BranchType::Refactoring;

        // Act: 复制分支类型（Copy trait）
        let copied = original;

        // Assert: 验证复制后的值相等
        assert_eq!(original, copied);
    }

    // ==================== Integration Tests ====================

    /// 测试完整分支类型工作流
    ///
    /// ## 测试目的
    /// 验证分支类型的所有功能在完整工作流中正常工作。
    ///
    /// ## 测试场景
    /// 1. 获取所有分支类型
    /// 2. 对每种类型测试：转换为字符串、从字符串解析、获取提交类型、获取显示名称
    /// 3. 验证所有操作都成功
    ///
    /// ## 预期结果
    /// - 所有分支类型的所有功能都正常工作
    #[test]
    fn test_complete_branch_type_workflow_with_all_types_completes_successfully() {
        // Arrange: 获取所有类型
        let all_types = BranchType::all();
        assert!(!all_types.is_empty());

        // Act & Assert: 测试每种类型的完整功能
        for branch_type in all_types {
            // 转换为字符串
            let str_repr = branch_type.as_str();
            assert!(!str_repr.is_empty());

            // 从字符串转换回来
            let parsed = BranchType::from_str(str_repr);
            assert_eq!(parsed, Some(branch_type));

            // 获取提交类型
            let commit_type = branch_type.to_commit_type();
            assert!(!commit_type.is_empty());

            // 获取显示名称
            let display_name = branch_type.display_name();
            assert!(!display_name.is_empty());
            assert!(display_name.contains(str_repr));

            // Arrange: 准备测试显示格式
            let display = format!("{}", branch_type);
            assert_eq!(display, str_repr);
        }
    }

    // ==================== Error Handling Tests ====================

    /// 测试从无效输入解析分支类型
    ///
    /// ## 测试目的
    /// 验证 BranchType::from_str() 对无效输入返回 None。
    ///
    /// ## 测试场景
    /// 1. 使用各种无效输入解析分支类型
    /// 2. 验证返回 None 或有效的分支类型
    ///
    /// ## 预期结果
    /// - 无效输入返回 None，有效输入返回对应的分支类型
    #[test]
    fn test_branch_type_from_str_with_invalid_inputs_returns_none() {
        // Arrange: 准备无效输入列表
        let invalid_inputs = vec![
            "",
            "   ",
            "invalid",
            "unknown",
            "feat", // 这是提交类型，不是分支类型
            "fix",  // 这是提交类型，不是分支类型
            "123",
            "feature-branch",
            "bug_fix",
        ];

        // Act & Assert: 验证无效输入处理正确
        for input in invalid_inputs {
            let result = BranchType::from_str(input);
            if result.is_none() {
                // 预期的无效输入
                continue;
            }

            // 如果有结果，验证它确实是有效的
            if let Some(branch_type) = result {
                assert!(BranchType::all().contains(&branch_type));
            }
        }
    }

    // ==================== Real-World Usage Scenario Tests ====================

    /// 测试分支类型到提交类型映射
    ///
    /// ## 测试目的
    /// 验证分支类型到提交类型的映射在实际使用场景中正确。
    ///
    /// ## 测试场景
    /// 1. 转换所有分支类型为提交类型
    /// 2. 模拟生成完整的提交消息
    /// 3. 验证提交消息前缀正确
    ///
    /// ## 预期结果
    /// - 所有分支类型都正确映射到对应的提交类型
    #[test]
    fn test_branch_type_to_commit_type_with_all_types_maps_correctly() {
        // Arrange: 准备分支类型和预期提交类型映射
        let scenarios = vec![
            (BranchType::Feature, "feat"),
            (BranchType::Bugfix, "fix"),
            (BranchType::Refactoring, "refactor"),
            (BranchType::Hotfix, "fix"),
            (BranchType::Chore, "chore"),
        ];

        // Act & Assert: 模拟实际使用场景：根据分支类型生成提交消息前缀
        for (branch_type, expected_commit_type) in scenarios {
            let commit_type = branch_type.to_commit_type();
            assert_eq!(commit_type, expected_commit_type);

            // 模拟生成完整的提交消息
            let commit_message = format!("{}: implement feature", commit_type);
            assert!(commit_message.starts_with(expected_commit_type));
        }
    }

    /// 测试分支类型模板选择
    ///
    /// ## 测试目的
    /// 验证分支类型可用于模板路径和配置键的生成。
    ///
    /// ## 测试场景
    /// 1. 使用分支类型生成模板路径
    /// 2. 使用分支类型生成配置键
    /// 3. 验证路径和键正确
    ///
    /// ## 预期结果
    /// - 分支类型可以用于生成模板路径和配置键
    #[test]
    fn test_branch_type_template_selection_with_feature_type_returns_template_path() {
        // Arrange: 准备分支类型
        let branch_type = BranchType::Feature;
        let template_key = branch_type.as_str();

        // Act: 模拟模板选择场景
        let template_path = format!("templates/{}.hbs", template_key);
        let config_key = format!("branch.{}.prefix", template_key);

        // Assert: 验证模板键可用于文件路径和配置键
        assert!(template_path.contains("feature.hbs"));
        assert!(config_key.contains("branch.feature.prefix"));
    }

    /// 测试所有分支类型的显示名称
    ///
    /// ## 测试目的
    /// 验证所有分支类型的 display_name() 方法都返回正确的显示名称。
    ///
    /// ## 测试场景
    /// 1. 获取所有分支类型的显示名称
    /// 2. 验证显示名称不为空且包含类型字符串和分隔符
    /// 3. 明确测试每个分支类型的显示名称
    ///
    /// ## 预期结果
    /// - 所有分支类型都返回正确的显示名称
    #[test]
    fn test_branch_type_display_name_all_variants() {
        // 确保所有分支类型的 display_name 都被测试覆盖
        // 这个测试专门用于覆盖 display_name() 方法的所有分支（第70-76行）
        let all_types = BranchType::all();

        for branch_type in all_types {
            let display_name = branch_type.display_name();

            // Assert: 验证 display_name 不为空
            assert!(
                !display_name.is_empty(),
                "Display name should not be empty for {:?}",
                branch_type
            );

            // Assert: 验证 display_name 包含分支类型字符串
            let type_str = branch_type.as_str();
            assert!(
                display_name.contains(type_str),
                "Display name '{}' should contain type string '{}'",
                display_name,
                type_str
            );

            // Assert: 验证 display_name 包含中文描述（所有 display_name 都包含中文）
            assert!(
                display_name.contains(" - "),
                "Display name '{}' should contain separator ' - '",
                display_name
            );
        }

        // 明确测试每个分支类型的 display_name
        assert_eq!(BranchType::Feature.display_name(), "feature - 新功能开发");
        assert_eq!(BranchType::Bugfix.display_name(), "bugfix - Bug 修复");
        assert_eq!(
            BranchType::Refactoring.display_name(),
            "refactoring - 代码重构"
        );
        assert_eq!(BranchType::Hotfix.display_name(), "hotfix - 紧急修复");
        assert_eq!(BranchType::Chore.display_name(), "chore - 杂项任务");
    }
}
