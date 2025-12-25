//! 仓库配置集成测试
//!
//! 测试公共配置和私有配置的集成、配置迁移和边界情况。

use pretty_assertions::assert_eq;
use rstest::rstest;
use toml::map::Map;
use toml::Value;
use workflow::repo::config::types::{BranchConfig, PullRequestsConfig};
use workflow::repo::RepoConfig;

// ==================== Configuration Integration Tests ====================

/// 测试公共配置和私有配置的交互
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_public_and_private_config_interaction() {
    // Arrange: 准备测试公共配置和私有配置的交互
    let mut config = RepoConfig::default();

    // 设置公共配置（项目模板）
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));

    // 设置私有配置（个人偏好）
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec!["main".to_string(), "develop".to_string()],
    });
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    // Assert: 验证公共配置
    assert_eq!(config.template_commit.len(), 1);
    assert_eq!(config.template_branch.len(), 1);
    assert_eq!(config.template_pull_requests.len(), 1);

    // Assert: 验证私有配置
    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());
}

/// 测试公共配置和私有配置的独立性
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_independence() {
    // Arrange: 准备测试公共配置和私有配置的独立性
    let mut config = RepoConfig::default();

    // 只设置公共配置
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Assert: 验证私有配置保持默认值
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());

    // 只设置私有配置
    config.configured = true;

    // Assert: 验证公共配置不受影响
    assert_eq!(config.template_commit.len(), 1);
}

/// 测试模板分支前缀和个人分支前缀的共存
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_template_and_personal_branch_prefix() {
    // Arrange: 准备测试模板分支前缀和个人分支前缀的共存
    let mut config = RepoConfig::default();

    // 公共模板：项目标准分支前缀
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));

    // 私有配置：个人偏好分支前缀
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec![],
    });

    // 两者应该独立存在
    assert_eq!(
        config.template_branch.get("prefix"),
        Some(&Value::String("feature".to_string()))
    );
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("my-feature".to_string()));
    }
}

/// 测试模板PR配置和个人PR配置的共存
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_template_and_personal_pr_config() {
    // Arrange: 准备测试模板 PR 配置和个人 PR 配置的共存
    let mut config = RepoConfig::default();

    // 公共模板：项目 PR 标准
    config
        .template_pull_requests
        .insert("require_review".to_string(), Value::Boolean(true));
    config
        .template_pull_requests
        .insert("min_reviewers".to_string(), Value::Integer(2));

    // 私有配置：个人 PR 偏好
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    // 两者应该独立存在
    assert_eq!(config.template_pull_requests.len(), 2);
    assert!(config.pr.is_some());
}

// ==================== Configuration Migration Tests ====================

/// 测试从旧格式迁移配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_migration_from_old_format() {
    // Arrange: 准备测试从旧格式迁移配置
    let mut old_config = RepoConfig::default();

    // 模拟旧格式配置
    old_config
        .template_commit
        .insert("type".to_string(), Value::String("old_type".to_string()));

    // 迁移到新格式
    let mut new_config = RepoConfig::default();
    new_config.template_commit = old_config.template_commit.clone();
    new_config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Assert: 验证迁移结果
    assert_eq!(
        new_config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );
}

/// 测试迁移时添加新字段
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_migration_add_new_fields() {
    // Arrange: 准备测试迁移时添加新字段
    let mut config = RepoConfig::default();

    // 原有配置
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // 添加新字段（模拟版本升级）
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));
    config.template_commit.insert("max_length".to_string(), Value::Integer(72));

    // Assert: 验证所有字段都存在
    assert_eq!(config.template_commit.len(), 3);
    assert!(config.template_commit.contains_key("type"));
    assert!(config.template_commit.contains_key("scope_required"));
    assert!(config.template_commit.contains_key("max_length"));
}

/// 测试迁移时移除废弃字段
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_migration_remove_deprecated_fields() {
    // Arrange: 准备测试迁移时移除废弃字段
    let mut config = RepoConfig::default();

    // 原有配置（包含废弃字段）
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config.template_commit.insert(
        "deprecated_field".to_string(),
        Value::String("old".to_string()),
    );

    // 移除废弃字段
    config.template_commit.remove("deprecated_field");

    // Assert: 验证废弃字段已移除
    assert_eq!(config.template_commit.len(), 1);
    assert!(!config.template_commit.contains_key("deprecated_field"));
}

/// 测试迁移时保留用户数据
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_migration_preserve_user_data() {
    // Arrange: 准备测试迁移时保留用户数据
    let mut config = RepoConfig::default();

    // 用户配置
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    // 模拟配置迁移（更新模板配置）
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Assert: 验证用户数据未受影响
    assert!(config.configured);
    assert!(config.branch.is_some());
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("my-feature".to_string()));
    }
}

// ==================== Boundary Condition Tests ====================

/// 测试包含特殊字符的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_special_characters() {
    // Arrange: 准备测试包含特殊字符的配置
    let mut config = RepoConfig::default();

    // 特殊字符在模板配置中
    config.template_branch.insert(
        "pattern".to_string(),
        Value::String(r"^[a-z]+/[A-Z]+-\d+$".to_string()),
    );
    config
        .template_commit
        .insert("emoji".to_string(), Value::String("✨ 🚀 🎉".to_string()));

    // 特殊字符在私有配置中
    config.branch = Some(BranchConfig {
        prefix: Some("feature/test-123".to_string()),
        ignore: vec!["release/v1.0".to_string(), "hotfix-urgent".to_string()],
    });

    // Assert: 验证特殊字符正确处理
    assert!(config.template_branch.contains_key("pattern"));
    assert!(config.template_commit.contains_key("emoji"));
    assert!(config.branch.is_some());
}

/// 测试包含很长值的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_very_long_values() {
    // Arrange: 准备测试包含很长值的配置
    let mut config = RepoConfig::default();

    let long_string = "a".repeat(1000);
    config
        .template_commit
        .insert("long_field".to_string(), Value::String(long_string.clone()));

    assert_eq!(
        config.template_commit.get("long_field"),
        Some(&Value::String(long_string))
    );
}

/// 测试包含Unicode字符的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_unicode() {
    // Arrange: 准备测试包含 Unicode 字符的配置
    let mut config = RepoConfig::default();

    config.template_commit.insert(
        "description".to_string(),
        Value::String("功能: 添加新特性 🚀".to_string()),
    );
    config
        .template_branch
        .insert("中文键".to_string(), Value::String("中文值".to_string()));

    assert!(config.template_commit.contains_key("description"));
    assert!(config.template_branch.contains_key("中文键"));
}

/// 测试包含空字符串的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_empty_strings() {
    // Arrange: 准备测试包含空字符串的配置
    let mut config = RepoConfig::default();

    config
        .template_commit
        .insert("empty".to_string(), Value::String("".to_string()));
    config.branch = Some(BranchConfig {
        prefix: Some("".to_string()),
        ignore: vec![],
    });

    assert_eq!(
        config.template_commit.get("empty"),
        Some(&Value::String("".to_string()))
    );
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("".to_string()));
    }
}

/// 测试包含null值的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_null_values() {
    // Arrange: 准备测试包含 null 值的配置
    let mut config = RepoConfig::default();

    // TOML 中的 null 值通常不存在，但我们可以测试 None
    config.branch = Some(BranchConfig {
        prefix: None,
        ignore: vec![],
    });
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: None,
    });

    assert!(config.branch.is_some());
    assert!(config.pr.is_some());
    if let Some(ref branch) = config.branch {
        assert!(branch.prefix.is_none());
    }
    if let Some(ref pr) = config.pr {
        assert!(pr.auto_accept_change_type.is_none());
    }
}

/// 测试包含大量忽略分支的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_many_ignore_branches() {
    // Arrange: 准备测试包含大量忽略分支的配置
    let mut config = RepoConfig::default();

    let ignore_branches: Vec<String> = (0..100).map(|i| format!("branch-{}", i)).collect();

    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: ignore_branches.clone(),
    });

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.ignore.len(), 100);
    }
}

/// 测试包含嵌套表格的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_nested_tables() {
    // Arrange: 准备测试包含嵌套表格的配置
    let mut config = RepoConfig::default();

    let mut nested_table = Map::new();
    nested_table.insert("enabled".to_string(), Value::Boolean(true));
    nested_table.insert("level".to_string(), Value::String("strict".to_string()));

    let mut inner_table = Map::new();
    inner_table.insert("max_length".to_string(), Value::Integer(72));
    nested_table.insert("rules".to_string(), Value::Table(inner_table));

    config
        .template_commit
        .insert("validation".to_string(), Value::Table(nested_table));

    assert!(config.template_commit.contains_key("validation"));
}

/// 测试包含数组的配置
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_with_arrays() {
    // Arrange: 准备测试包含数组的配置
    let mut config = RepoConfig::default();

    let types = vec![
        Value::String("feat".to_string()),
        Value::String("fix".to_string()),
        Value::String("docs".to_string()),
        Value::String("style".to_string()),
        Value::String("refactor".to_string()),
    ];

    config
        .template_commit
        .insert("allowed_types".to_string(), Value::Array(types.clone()));

    assert_eq!(
        config.template_commit.get("allowed_types"),
        Some(&Value::Array(types))
    );
}

// ==================== Configuration Consistency Tests ====================

/// 测试多次更新后配置的一致性
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_consistency_after_multiple_updates() {
    // Arrange: 准备测试多次更新后配置的一致性
    let mut config = RepoConfig::default();

    // 第一次更新
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config.configured = true;

    // 第二次更新
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec![],
    });

    // 第三次更新
    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    // Assert: 验证所有配置都存在且正确
    assert_eq!(config.template_commit.len(), 1);
    assert_eq!(config.template_branch.len(), 1);
    assert_eq!(config.template_pull_requests.len(), 1);
    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());
}

/// 测试部分清空后配置的一致性
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_consistency_after_partial_clear() {
    // Arrange: 准备测试部分清空后配置的一致性
    let mut config = RepoConfig::default();

    // 设置所有配置
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec![],
    });

    // 清空部分配置
    config.template_commit.clear();
    config.branch = None;

    // Assert: 验证剩余配置正确
    assert!(config.template_commit.is_empty());
    assert_eq!(config.template_branch.len(), 1);
    assert!(config.configured);
    assert!(config.branch.is_none());
}

// ==================== Parameterized Tests ====================

/// 测试配置的各种组合情况（参数化测试）
///
/// ## 测试目的
/// 验证 `RepoConfig` 在不同配置组合下的行为是否正确。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下组合：
/// - `has_public=true, has_private=true, configured=true`
/// - `has_public=true, has_private=false, configured=false`
/// - `has_public=false, has_private=true, configured=false`
/// - `has_public=false, has_private=false, configured=false`
///
/// ## 预期结果
/// - 所有配置组合都能正确处理
/// - 配置状态与预期一致
#[rstest]
#[case(true, true, true)]
#[case(true, false, false)]
#[case(false, true, false)]
#[case(false, false, false)]
fn test_config_combinations(
    #[case] has_public: bool,
    #[case] has_private: bool,
    #[case] configured: bool,
) {
    // 参数化测试配置的各种组合
    let mut config = RepoConfig::default();

    if has_public {
        config.template_commit.insert(
            "type".to_string(),
            Value::String("conventional".to_string()),
        );
    }

    if has_private {
        config.configured = configured;
        config.branch = Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec![],
        });
    }

    assert_eq!(!config.template_commit.is_empty(), has_public);
    assert_eq!(config.configured, has_private && configured);
}

// ==================== Error Recovery Tests ====================

/// 测试无效更新后的配置恢复
#[test]
fn test_config_recovery_after_invalid_update() {
    // Arrange: 准备测试无效更新后的配置恢复
    let mut config = RepoConfig::default();

    // 设置有效配置
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // 尝试设置无效配置（这里我们只是模拟，实际上 Rust 类型系统会阻止大部分无效操作）
    // 例如：清空然后重新设置
    config.template_commit.clear();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Assert: 验证配置已恢复
    assert_eq!(config.template_commit.len(), 1);
}

/// 测试配置回滚
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_rollback() {
    // Arrange: 准备测试配置回滚
    let mut config = RepoConfig::default();

    // 原始配置
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    let original_commit = config.template_commit.clone();

    // 修改配置
    config
        .template_commit
        .insert("type".to_string(), Value::String("semantic".to_string()));

    // 回滚配置
    config.template_commit = original_commit;

    // Assert: 验证已回滚
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );
}
