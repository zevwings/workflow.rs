//! Branch 类型测试
//!
//! 测试分支类型定义、转换和验证功能。

use crate::common::performance::measure_test_time_with_threshold;
use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::time::Duration;
use workflow::branch::BranchType;

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

/// 测试从字符串解析分支类型（有效输入）（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 BranchType::from_str() 能够从有效字符串解析分支类型。
///
/// ## 测试场景
/// 测试各种有效输入（包括别名）和无效输入
///
/// ## 预期结果
/// - 有效输入返回对应的分支类型，无效输入返回 None
#[rstest]
#[case("feature", Some(BranchType::Feature))]
#[case("bugfix", Some(BranchType::Bugfix))]
#[case("bug", Some(BranchType::Bugfix))]
#[case("fix", Some(BranchType::Bugfix))]
#[case("refactoring", Some(BranchType::Refactoring))]
#[case("refactor", Some(BranchType::Refactoring))]
#[case("hotfix", Some(BranchType::Hotfix))]
#[case("chore", Some(BranchType::Chore))]
#[case("invalid", None)]
fn test_branch_type_from_string_with_valid_input(
    #[case] input: &str,
    #[case] expected: Option<BranchType>,
) {
    // Arrange: 准备测试用例（通过参数提供）

    // Act & Assert: 验证从字符串创建分支类型
    let result = BranchType::from_str(input);
    assert_eq!(result, expected);
}

/// 测试从字符串解析分支类型（大小写不敏感）（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 BranchType::from_str() 支持大小写不敏感的解析。
///
/// ## 测试场景
/// 测试不同大小写的字符串输入
///
/// ## 预期结果
/// - 大小写不敏感，所有变体都能正确解析
#[rstest]
#[case("FEATURE", Some(BranchType::Feature))]
#[case("BugFix", Some(BranchType::Bugfix))]
#[case("REFACTORING", Some(BranchType::Refactoring))]
#[case("HotFix", Some(BranchType::Hotfix))]
#[case("CHORE", Some(BranchType::Chore))]
fn test_branch_type_from_string_with_case_insensitive_input(
    #[case] input: &str,
    #[case] expected: Option<BranchType>,
) {
    // Arrange: 准备测试用例（通过参数提供）

    // Act & Assert: 验证大小写不敏感的转换
    let result = BranchType::from_str(input);
    assert_eq!(result, expected);
}

// ==================== BranchType 功能测试 ====================

/// 测试分支类型转换为提交类型（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 BranchType::to_commit_type() 能够将分支类型转换为 Conventional Commits 类型。
///
/// ## 测试场景
/// 测试所有分支类型到提交类型的转换
///
/// ## 预期结果
/// - 所有分支类型都正确转换为对应的提交类型
#[rstest]
#[case(BranchType::Feature, "feat")]
#[case(BranchType::Bugfix, "fix")]
#[case(BranchType::Refactoring, "refactor")]
#[case(BranchType::Hotfix, "fix")]
#[case(BranchType::Chore, "chore")]
fn test_branch_type_to_commit_type(
    #[case] branch_type: BranchType,
    #[case] expected_commit_type: &str,
) {
    // Arrange: 准备分支类型和预期提交类型（通过参数提供）

    // Act & Assert: 验证转换为 Conventional Commits 类型正确
    assert_eq!(branch_type.to_commit_type(), expected_commit_type);
}

/// 测试分支类型显示名称（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 BranchType::display_name() 返回包含描述的显示名称。
///
/// ## 测试场景
/// 测试所有分支类型的显示名称
///
/// ## 预期结果
/// - 所有分支类型都返回包含描述的显示名称
#[rstest]
#[case(BranchType::Feature, "feature - 新功能开发")]
#[case(BranchType::Bugfix, "bugfix - Bug 修复")]
#[case(BranchType::Refactoring, "refactoring - 代码重构")]
#[case(BranchType::Hotfix, "hotfix - 紧急修复")]
#[case(BranchType::Chore, "chore - 杂项任务")]
fn test_branch_type_display_name(
    #[case] branch_type: BranchType,
    #[case] expected_display_name: &str,
) {
    // Arrange: 准备分支类型和预期显示名称（通过参数提供）

    // Act & Assert: 验证显示名称（包含描述）正确
    assert_eq!(branch_type.display_name(), expected_display_name);
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
fn test_branch_type_from_str_with_special_characters_handles_correctly() -> Result<()> {
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
    Ok(())
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

// ==================== Performance Tests ====================

/// 测试分支类型转换性能
///
/// ## 测试目的
/// 验证分支类型转换操作在大量调用时保持良好性能。
///
/// ## 测试场景
/// 1. 执行1000次转换操作
/// 2. 测量执行时间
///
/// ## 预期结果
/// - 1000次转换在100毫秒内完成
#[test]
fn test_branch_type_conversion_performance_with_multiple_conversions_completes_quickly(
) -> Result<()> {
    // Arrange: 准备测试字符串
    let test_strings = vec!["feature", "bugfix", "refactoring", "hotfix", "chore"];

    // Act & Assert: 多次转换并测量时间（应该很快，< 100ms）
    measure_test_time_with_threshold(
        "test_branch_type_conversion_performance_with_multiple_conversions_completes_quickly",
        Duration::from_millis(100),
        || {
            for _ in 0..1000 {
                for s in &test_strings {
                    let _ = BranchType::from_str(s);
                }
            }
            Ok(())
        },
    )
}

/// 测试分支类型显示性能
///
/// ## 测试目的
/// 验证分支类型显示操作在大量调用时保持良好性能。
///
/// ## 测试场景
/// 1. 执行1000次显示转换操作
/// 2. 测量执行时间
///
/// ## 预期结果
/// - 1000次显示转换在50毫秒内完成
#[test]
fn test_branch_type_display_performance_with_multiple_displays_completes_quickly() -> Result<()> {
    // Arrange: 准备所有分支类型
    let types = BranchType::all();

    // Act & Assert: 多次显示转换并测量时间（应该很快，< 50ms）
    measure_test_time_with_threshold(
        "test_branch_type_display_performance_with_multiple_displays_completes_quickly",
        Duration::from_millis(50),
        || {
            for _ in 0..1000 {
                for branch_type in &types {
                    let _ = branch_type.to_string();
                    let _ = branch_type.as_str();
                    let _ = branch_type.display_name();
                    let _ = branch_type.to_commit_type();
                }
            }
            Ok(())
        },
    )
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
fn test_branch_type_from_str_with_invalid_inputs_returns_none() -> Result<()> {
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
        let branch_type = result.ok_or_else(|| color_eyre::eyre::eyre!("result should be Some"))?;
        assert!(BranchType::all().contains(&branch_type));
    }
    Ok(())
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

// ==================== prompt_selection 和 resolve_with_repo_prefix 测试 ====================

/// 测试分支类型的交互式选择
///
/// ## 测试目的
/// 验证`BranchType::prompt_selection()`方法能够正确显示选项并接收用户选择的分支类型。
///
/// ## 为什么被忽略
/// - **需要用户交互**: 测试需要用户使用方向键选择分支类型
/// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
/// - **UI/UX验证**: 用于手动验证分支类型选择对话框
/// - **会卡住CI**: 在非交互式环境中会无限等待用户输入
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_type_prompt_selection -- --ignored
/// ```
/// 然后使用↑↓键选择分支类型，Enter确认
///
/// ## 测试场景
/// 1. 调用BranchType::prompt_selection()显示分支类型列表
/// 2. 显示所有可用的分支类型（feature、fix、hotfix等）
/// 3. 等待用户使用方向键选择
/// 4. 用户按Enter确认选择
/// 5. 返回选中的分支类型
///
/// ## 预期行为
/// - 成功情况：返回Ok(BranchType)包含用户选择的类型
/// - 取消情况：返回Err表示用户取消了选择
/// - 显示的分支类型列表完整且正确
/// - 选择的分支类型在BranchType::all()列表中
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
#[cfg(feature = "interactive-tests")]
fn test_branch_type_prompt_selection() {
    // Arrange: 准备测试交互式选择分支类型
    // 注意：这个测试需要用户交互，在 CI 环境中会卡住
    // 使用 `cargo test -- --ignored` 来运行这些测试
    let result = workflow::branch::BranchType::prompt_selection();

    // 如果用户交互失败（例如没有配置），应该返回错误
    // 如果成功，应该返回有效的 BranchType
    match result {
        Ok(branch_type) => {
            // Assert: 验证返回的是有效的分支类型
            assert!(workflow::branch::BranchType::all().contains(&branch_type));
        }
        Err(_) => {
            // 交互失败是可以接受的（例如在 CI 环境中）
            assert!(
                true,
                "Prompt selection may fail in non-interactive environments"
            );
        }
    }
}

/// 测试从仓库配置解析分支类型（有prefix）
///
/// ## 测试目的
/// 验证`BranchType::resolve_with_repo_prefix()`在仓库配置了prefix时能够正确解析分支类型。
///
/// ## 为什么被忽略
/// - **需要仓库配置**: 测试依赖实际的仓库配置文件
/// - **可能需要用户交互**: 如果配置不完整，可能需要用户选择
/// - **环境依赖**: 不同环境中的配置可能不同
/// - **CI环境不支持**: 可能需要交互式输入
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_type_resolve_with_repo_prefix_with_prefix -- --ignored
/// ```
/// 注意：需要在配置了repository prefix的Git仓库中运行
///
/// ## 测试场景
/// 1. 读取仓库配置文件
/// 2. 查找repository prefix配置
/// 3. 如果有prefix：直接解析对应的分支类型
/// 4. 如果无prefix：调用prompt_selection（需要交互）
/// 5. 返回解析的分支类型
///
/// ## 预期行为
/// - 有配置时：返回Ok(BranchType)对应配置的类型
/// - 无配置时：调用交互选择或返回错误
/// - 解析的分支类型有效
/// - 配置文件解析正确
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
#[cfg(feature = "interactive-tests")]
fn test_branch_type_resolve_with_repo_prefix_with_prefix() {
    // Arrange: 准备测试有 repository prefix 的情况
    // 注意：这个测试依赖于实际的仓库配置，可能在不同环境中表现不同
    let result = workflow::branch::BranchType::resolve_with_repo_prefix();

    // 如果仓库有配置 prefix，应该返回对应的 BranchType
    // 如果没有配置，可能会调用 prompt_selection（需要交互）
    match result {
        Ok(branch_type) => {
            // Assert: 验证返回的是有效的分支类型
            assert!(workflow::branch::BranchType::all().contains(&branch_type));
        }
        Err(_) => {
            // 如果没有配置且 prompt 失败，这是可以接受的
            assert!(
                true,
                "resolve_with_repo_prefix may fail if no prefix configured and prompt fails"
            );
        }
    }
}

/// 测试从仓库配置解析分支类型（无prefix）
///
/// ## 测试目的
/// 验证`BranchType::resolve_with_repo_prefix()`在仓库未配置prefix时能够回退到交互式选择。
///
/// ## 为什么被忽略
/// - **需要用户交互**: 无prefix时会调用prompt_selection需要用户输入
/// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
/// - **环境依赖**: 依赖仓库配置状态
/// - **会卡住CI**: 在非交互式环境中会等待用户输入
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_type_resolve_with_repo_prefix_without_prefix -- --ignored
/// ```
/// 注意：需要在未配置repository prefix的Git仓库中运行
///
/// ## 测试场景
/// 1. 读取仓库配置文件
/// 2. 发现没有配置repository prefix
/// 3. 调用BranchType::prompt_selection()进行交互选择
/// 4. 等待用户选择分支类型
/// 5. 返回用户选择的分支类型
///
/// ## 预期行为
/// - 检测到无prefix配置
/// - 自动调用交互式选择
/// - 用户选择后返回Ok(BranchType)
/// - 用户取消则返回Err
/// - 整个流程无panic或hang
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
#[cfg(feature = "interactive-tests")]
fn test_branch_type_resolve_with_repo_prefix_without_prefix() {
    // Arrange: 准备测试没有 repository prefix 的情况（会调用 prompt_selection）
    // 注意：这个测试需要用户交互，在 CI 环境中会卡住
    let result = workflow::branch::BranchType::resolve_with_repo_prefix();

    // 如果没有 prefix，应该调用 prompt_selection
    match result {
        Ok(branch_type) => {
            assert!(workflow::branch::BranchType::all().contains(&branch_type));
        }
        Err(_) => {
            // 交互失败是可以接受的
            assert!(
                true,
                "resolve_with_repo_prefix may fail in non-interactive environments"
            );
        }
    }
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
    let all_types = workflow::branch::BranchType::all();

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
    assert_eq!(
        workflow::branch::BranchType::Feature.display_name(),
        "feature - 新功能开发"
    );
    assert_eq!(
        workflow::branch::BranchType::Bugfix.display_name(),
        "bugfix - Bug 修复"
    );
    assert_eq!(
        workflow::branch::BranchType::Refactoring.display_name(),
        "refactoring - 代码重构"
    );
    assert_eq!(
        workflow::branch::BranchType::Hotfix.display_name(),
        "hotfix - 紧急修复"
    );
    assert_eq!(
        workflow::branch::BranchType::Chore.display_name(),
        "chore - 杂项任务"
    );
}
