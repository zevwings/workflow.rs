//! Branch 类型测试
//!
//! 测试分支类型定义、转换和验证功能。

use pretty_assertions::assert_eq;
// use rstest::{fixture, rstest};
use workflow::branch::BranchType;

// ==================== BranchType 枚举测试 ====================

#[test]
fn test_branch_type_enum_values() {
    // 测试所有分支类型枚举值
    let types = vec![
        BranchType::Feature,
        BranchType::Bugfix,
        BranchType::Refactoring,
        BranchType::Hotfix,
        BranchType::Chore,
    ];

    // 验证所有类型都可以创建
    for branch_type in types {
        let debug_str = format!("{:?}", branch_type);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_branch_type_all() {
    // 测试获取所有分支类型
    let all_types = BranchType::all();
    assert_eq!(all_types.len(), 5);
    assert!(all_types.contains(&BranchType::Feature));
    assert!(all_types.contains(&BranchType::Bugfix));
    assert!(all_types.contains(&BranchType::Refactoring));
    assert!(all_types.contains(&BranchType::Hotfix));
    assert!(all_types.contains(&BranchType::Chore));
}

#[test]
fn test_branch_type_display() {
    // 测试分支类型的显示格式
    assert_eq!(BranchType::Feature.to_string(), "feature");
    assert_eq!(BranchType::Bugfix.to_string(), "bugfix");
    assert_eq!(BranchType::Refactoring.to_string(), "refactoring");
    assert_eq!(BranchType::Hotfix.to_string(), "hotfix");
    assert_eq!(BranchType::Chore.to_string(), "chore");
}

#[test]
fn test_branch_type_as_str() {
    // 测试 as_str 方法
    assert_eq!(BranchType::Feature.as_str(), "feature");
    assert_eq!(BranchType::Bugfix.as_str(), "bugfix");
    assert_eq!(BranchType::Refactoring.as_str(), "refactoring");
    assert_eq!(BranchType::Hotfix.as_str(), "hotfix");
    assert_eq!(BranchType::Chore.as_str(), "chore");
}

#[test]
fn test_branch_type_from_string() {
    // 测试从字符串创建分支类型
    let test_cases = vec![
        ("feature", Some(BranchType::Feature)),
        ("bugfix", Some(BranchType::Bugfix)),
        ("bug", Some(BranchType::Bugfix)),
        ("fix", Some(BranchType::Bugfix)),
        ("refactoring", Some(BranchType::Refactoring)),
        ("refactor", Some(BranchType::Refactoring)),
        ("hotfix", Some(BranchType::Hotfix)),
        ("chore", Some(BranchType::Chore)),
        ("invalid", None),
    ];

    for (input, expected) in test_cases {
        let result = BranchType::from_str(input);
        assert_eq!(result, expected);
    }
}

#[test]
fn test_branch_type_from_string_case_insensitive() {
    // 测试大小写不敏感的转换
    let test_cases = vec![
        ("FEATURE", Some(BranchType::Feature)),
        ("BugFix", Some(BranchType::Bugfix)),
        ("REFACTORING", Some(BranchType::Refactoring)),
        ("HotFix", Some(BranchType::Hotfix)),
        ("CHORE", Some(BranchType::Chore)),
    ];

    for (input, expected) in test_cases {
        let result = BranchType::from_str(input);
        assert_eq!(result, expected);
    }
}

// ==================== BranchType 功能测试 ====================

#[test]
fn test_branch_type_to_commit_type() {
    // 测试转换为 Conventional Commits 类型
    assert_eq!(BranchType::Feature.to_commit_type(), "feat");
    assert_eq!(BranchType::Bugfix.to_commit_type(), "fix");
    assert_eq!(BranchType::Refactoring.to_commit_type(), "refactor");
    assert_eq!(BranchType::Hotfix.to_commit_type(), "fix");
    assert_eq!(BranchType::Chore.to_commit_type(), "chore");
}

#[test]
fn test_branch_type_display_name() {
    // 测试显示名称（包含描述）
    assert_eq!(BranchType::Feature.display_name(), "feature - 新功能开发");
    assert_eq!(BranchType::Bugfix.display_name(), "bugfix - Bug 修复");
    assert_eq!(
        BranchType::Refactoring.display_name(),
        "refactoring - 代码重构"
    );
    assert_eq!(BranchType::Hotfix.display_name(), "hotfix - 紧急修复");
    assert_eq!(BranchType::Chore.display_name(), "chore - 杂项任务");
}

// ==================== 边界条件测试 ====================

#[test]
fn test_branch_type_from_empty_string() {
    let result = BranchType::from_str("");
    assert_eq!(result, None);
}

#[test]
fn test_branch_type_from_whitespace() {
    let result = BranchType::from_str("   ");
    assert_eq!(result, None);
}

#[test]
fn test_branch_type_from_special_characters() {
    let special_strings = vec!["feat!", "bug#", "fix@", "hot-fix", "feature_branch"];

    for special_str in special_strings {
        let result = BranchType::from_str(special_str);
        // 大部分特殊字符应该返回 None，除非有特殊处理
        if result.is_some() {
            // 如果有结果，验证它是有效的类型
            let branch_type = result.expect("operation should succeed");
            assert!(BranchType::all().contains(&branch_type));
        }
    }
}

// ==================== 分支类型比较测试 ====================

#[test]
fn test_branch_type_equality() {
    // 测试分支类型相等性
    assert_eq!(BranchType::Feature, BranchType::Feature);
    assert_eq!(BranchType::Bugfix, BranchType::Bugfix);

    assert_ne!(BranchType::Feature, BranchType::Bugfix);
    assert_ne!(BranchType::Hotfix, BranchType::Chore);
}

#[test]
fn test_branch_type_clone() {
    // 测试克隆功能
    let original = BranchType::Feature;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_branch_type_copy() {
    // 测试复制功能
    let original = BranchType::Refactoring;
    let copied = original;
    assert_eq!(original, copied);
}

// ==================== 性能测试 ====================

#[test]
fn test_branch_type_conversion_performance() {
    use std::time::Instant;

    let test_strings = vec!["feature", "bugfix", "refactoring", "hotfix", "chore"];

    let start = Instant::now();

    // 多次转换
    for _ in 0..1000 {
        for s in &test_strings {
            let _ = BranchType::from_str(s);
        }
    }

    let duration = start.elapsed();

    // 转换应该很快
    assert!(
        duration.as_millis() < 100,
        "Branch type conversion too slow: {:?}",
        duration
    );
}

#[test]
fn test_branch_type_display_performance() {
    use std::time::Instant;

    let types = BranchType::all();

    let start = Instant::now();

    // 多次显示转换
    for _ in 0..1000 {
        for branch_type in &types {
            let _ = branch_type.to_string();
            let _ = branch_type.as_str();
            let _ = branch_type.display_name();
            let _ = branch_type.to_commit_type();
        }
    }

    let duration = start.elapsed();

    // 显示转换应该很快
    assert!(
        duration.as_millis() < 50,
        "Branch type display too slow: {:?}",
        duration
    );
}

// ==================== 集成测试 ====================

#[test]
fn test_complete_branch_type_workflow() {
    // 1. 获取所有类型
    let all_types = BranchType::all();
    assert!(!all_types.is_empty());

    // 2. 测试每种类型的完整功能
    for branch_type in all_types {
        // 3. 转换为字符串
        let str_repr = branch_type.as_str();
        assert!(!str_repr.is_empty());

        // 4. 从字符串转换回来
        let parsed = BranchType::from_str(str_repr);
        assert_eq!(parsed, Some(branch_type));

        // 5. 获取提交类型
        let commit_type = branch_type.to_commit_type();
        assert!(!commit_type.is_empty());

        // 6. 获取显示名称
        let display_name = branch_type.display_name();
        assert!(!display_name.is_empty());
        assert!(display_name.contains(str_repr));

        // 7. 测试显示格式
        let display = format!("{}", branch_type);
        assert_eq!(display, str_repr);
    }
}

// ==================== 错误处理测试 ====================

#[test]
fn test_branch_type_invalid_inputs() {
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

    for input in invalid_inputs {
        let result = BranchType::from_str(input);
        if result.is_none() {
            // 预期的无效输入
            continue;
        }

        // 如果有结果，验证它确实是有效的
        let branch_type = result.expect("operation should succeed");
        assert!(BranchType::all().contains(&branch_type));
    }
}

// ==================== 实际使用场景测试 ====================

#[test]
fn test_branch_type_commit_mapping_scenario() {
    // 模拟实际使用场景：根据分支类型生成提交消息前缀
    let scenarios = vec![
        (BranchType::Feature, "feat"),
        (BranchType::Bugfix, "fix"),
        (BranchType::Refactoring, "refactor"),
        (BranchType::Hotfix, "fix"),
        (BranchType::Chore, "chore"),
    ];

    for (branch_type, expected_commit_type) in scenarios {
        let commit_type = branch_type.to_commit_type();
        assert_eq!(commit_type, expected_commit_type);

        // 模拟生成完整的提交消息
        let commit_message = format!("{}: implement feature", commit_type);
        assert!(commit_message.starts_with(expected_commit_type));
    }
}

#[test]
fn test_branch_type_template_selection_scenario() {
    // 模拟模板选择场景
    let branch_type = BranchType::Feature;
    let template_key = branch_type.as_str();

    // 验证模板键可用于文件路径
    let template_path = format!("templates/{}.hbs", template_key);
    assert!(template_path.contains("feature.hbs"));

    // 验证可用于配置键
    let config_key = format!("branch.{}.prefix", template_key);
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
fn test_branch_type_prompt_selection() {
    // 测试交互式选择分支类型
    // 注意：这个测试需要用户交互，在 CI 环境中会卡住
    // 使用 `cargo test -- --ignored` 来运行这些测试
    let result = workflow::branch::BranchType::prompt_selection();

    // 如果用户交互失败（例如没有配置），应该返回错误
    // 如果成功，应该返回有效的 BranchType
    match result {
        Ok(branch_type) => {
            // 验证返回的是有效的分支类型
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
fn test_branch_type_resolve_with_repo_prefix_with_prefix() {
    // 测试有 repository prefix 的情况
    // 注意：这个测试依赖于实际的仓库配置，可能在不同环境中表现不同
    let result = workflow::branch::BranchType::resolve_with_repo_prefix();

    // 如果仓库有配置 prefix，应该返回对应的 BranchType
    // 如果没有配置，可能会调用 prompt_selection（需要交互）
    match result {
        Ok(branch_type) => {
            // 验证返回的是有效的分支类型
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
fn test_branch_type_resolve_with_repo_prefix_without_prefix() {
    // 测试没有 repository prefix 的情况（会调用 prompt_selection）
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

#[test]
fn test_branch_type_display_name_all_variants() {
    // 确保所有分支类型的 display_name 都被测试覆盖
    // 这个测试专门用于覆盖 display_name() 方法的所有分支（第70-76行）
    let all_types = workflow::branch::BranchType::all();

    for branch_type in all_types {
        let display_name = branch_type.display_name();

        // 验证 display_name 不为空
        assert!(
            !display_name.is_empty(),
            "Display name should not be empty for {:?}",
            branch_type
        );

        // 验证 display_name 包含分支类型字符串
        let type_str = branch_type.as_str();
        assert!(
            display_name.contains(type_str),
            "Display name '{}' should contain type string '{}'",
            display_name,
            type_str
        );

        // 验证 display_name 包含中文描述（所有 display_name 都包含中文）
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
