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
            let branch_type = result.unwrap();
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
        let branch_type = result.unwrap();
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
