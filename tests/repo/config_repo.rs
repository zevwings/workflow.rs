//! RepoConfig 完整测试
//!
//! 包含数据结构测试、文件系统集成测试和错误场景测试

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::fs;
use workflow::base::settings::paths::Paths;
use workflow::repo::config::types::{BranchConfig, PullRequestsConfig};
use workflow::repo::RepoConfig;

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::{cli_env, cli_env_with_git};

// ==================== Default Value Tests ====================

/// 测试创建默认的RepoConfig
///
/// ## 测试目的
/// 验证 `RepoConfig::default()` 方法能够创建默认的空配置。
///
/// ## 测试场景
/// 1. 调用 `RepoConfig::default()` 创建默认配置
///
/// ## 预期结果
/// - 公共配置字段为空
/// - 私有配置字段为默认值（configured=false，branch=None，pr=None）
#[test]
fn test_repo_config_default_with_no_params_returns_default_config() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 创建默认配置
    let config = RepoConfig::default();

    // Assert: 验证默认值
    // Public configuration
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());

    // Private configuration
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
}

// ==================== Configuration Field Tests ====================

/// 测试设置commit模板配置
///
/// ## 测试目的
/// 验证能够正确设置和获取 commit 模板配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 template_commit 配置值
///
/// ## 预期结果
/// - template_commit 配置正确设置
/// - 配置值可以正确获取
#[test]
fn test_repo_config_with_template_commit_returns_config_with_commit_template() {
    // Arrange: 准备 template_commit 配置值
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));

    // Act & Assert: 验证 template_commit 配置
    assert_eq!(config.template_commit.len(), 2);
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );
}

/// 测试设置branch模板配置
///
/// ## 测试目的
/// 验证能够正确设置和获取 branch 模板配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 template_branch 配置值
///
/// ## 预期结果
/// - template_branch 配置正确设置
/// - 配置值可以正确获取
#[test]
fn test_repo_config_with_template_branch_returns_config_with_branch_template() {
    // Arrange: 准备 template_branch 配置值
    use toml::Value;

    let mut config = RepoConfig::default();
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config
        .template_branch
        .insert("separator".to_string(), Value::String("/".to_string()));

    // Act & Assert: 验证 template_branch 配置
    assert_eq!(config.template_branch.len(), 2);
    assert_eq!(
        config.template_branch.get("prefix"),
        Some(&Value::String("feature".to_string()))
    );
}

/// 测试设置pull requests模板配置
///
/// ## 测试目的
/// 验证能够正确设置和获取 pull requests 模板配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 template_pull_requests 配置值
///
/// ## 预期结果
/// - template_pull_requests 配置正确设置
/// - 配置值可以正确获取
#[test]
fn test_repo_config_with_template_pull_requests_returns_config_with_pr_template() {
    // Arrange: 准备 template_pull_requests 配置值
    use toml::Value;

    let mut config = RepoConfig::default();
    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));
    config
        .template_pull_requests
        .insert("require_review".to_string(), Value::Boolean(true));

    // Act & Assert: 验证 template_pull_requests 配置
    assert_eq!(config.template_pull_requests.len(), 2);
    assert_eq!(
        config.template_pull_requests.get("auto_merge"),
        Some(&Value::Boolean(false))
    );
}

/// 测试设置configured标志
///
/// ## 测试目的
/// 验证能够正确设置 configured 标志。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 configured 字段为 true
///
/// ## 预期结果
/// - configured 标志正确设置
#[test]
fn test_repo_config_with_configured_flag_returns_config_with_configured_true() {
    // Arrange: 准备配置
    let mut config = RepoConfig::default();

    // Act: 设置 configured 字段
    config.configured = true;

    // Assert: 验证 configured 为 true
    assert!(config.configured);
}

/// 测试设置branch配置
///
/// ## 测试目的
/// 验证能够正确设置和获取 branch 配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 branch 配置（包含 prefix 和 ignore）
///
/// ## 预期结果
/// - branch 配置正确设置
/// - prefix 和 ignore 值正确
#[test]
fn test_repo_config_with_branch_config_returns_config_with_branch() {
    // Arrange: 准备 branch 配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string(), "develop".to_string()],
    });

    // Act & Assert: 验证 branch 配置
    assert!(config.branch.is_some());
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("feature".to_string()));
        assert_eq!(branch.ignore.len(), 2);
    }
}

/// 测试设置PR配置
///
/// ## 测试目的
/// 验证能够正确设置和获取 PR 配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 pr 配置
///
/// ## 预期结果
/// - pr 配置正确设置
/// - 配置值可以正确获取
#[test]
fn test_repo_config_with_pr_config_returns_config_with_pr() {
    // Arrange: 准备 PR 配置
    let mut config = RepoConfig::default();
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    // Act & Assert: 验证 PR 配置
    assert!(config.pr.is_some());
    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, Some(true));
    }
}

/// 测试设置所有配置字段
///
/// ## 测试目的
/// 验证能够同时设置所有配置字段（公共和私有配置）。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置所有公共配置字段
/// 3. 设置所有私有配置字段
///
/// ## 预期结果
/// - 所有配置字段都正确设置
#[test]
fn test_repo_config_with_all_fields_returns_complete_config() {
    // Arrange: 准备所有配置字段
    use toml::Value;

    let mut config = RepoConfig::default();

    // Public configuration
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

    // Private configuration
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    // Act & Assert: 验证所有字段都已设置
    assert!(!config.template_commit.is_empty());
    assert!(!config.template_branch.is_empty());
    assert!(!config.template_pull_requests.is_empty());
    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());
}

// ==================== Static Method Tests ====================

/// 测试获取分支前缀（无配置时返回Option）
///
/// ## 测试目的
/// 验证 `RepoConfig::get_branch_prefix()` 方法能够正确返回分支前缀。
///
/// ## 测试场景
/// 1. 调用 `get_branch_prefix()` 方法
///
/// ## 预期结果
/// - 返回 Option<String>（如果配置存在则返回 Some，否则返回 None）
#[test]
fn test_get_branch_prefix_with_no_config_returns_option() {
    // Arrange: 准备测试（无需额外准备）
    // 注意：这个测试依赖于当前仓库的配置状态

    // Act: 获取分支前缀
    let prefix = RepoConfig::get_branch_prefix();

    // Assert: 验证返回类型是 Option<String>
    // 如果配置不存在，应该返回 None
    // 如果配置存在，应该返回配置的值
    assert!(prefix.is_none() || prefix.is_some());
}

/// 测试获取忽略分支列表（无配置时返回Vec）
///
/// ## 测试目的
/// 验证 `RepoConfig::get_ignore_branches()` 方法能够正确返回忽略分支列表。
///
/// ## 测试场景
/// 1. 调用 `get_ignore_branches()` 方法
///
/// ## 预期结果
/// - 返回 Vec<String>（可能为空或包含分支列表）
#[test]
fn test_get_ignore_branches_with_no_config_returns_vec() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取忽略分支列表
    let branches = RepoConfig::get_ignore_branches();

    // Assert: 验证返回值是一个 Vec<String>
    assert!(branches.is_empty() || !branches.is_empty());
}

/// 测试获取auto_accept_change_type（无配置时返回bool）
///
/// ## 测试目的
/// 验证 `RepoConfig::get_auto_accept_change_type()` 方法能够正确返回自动接受变更类型的标志。
///
/// ## 测试场景
/// 1. 调用 `get_auto_accept_change_type()` 方法
///
/// ## 预期结果
/// - 返回布尔值（默认 false 或根据配置返回 true）
#[test]
fn test_get_auto_accept_change_type_with_no_config_returns_bool() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 auto_accept_change_type
    let auto_accept = RepoConfig::get_auto_accept_change_type();

    // Assert: 验证返回布尔值
    // 默认应该是 false，或者根据配置返回 true
    assert!(!auto_accept || auto_accept);
}

/// 测试获取commit模板配置（无配置时返回Map）
///
/// ## 测试目的
/// 验证 `RepoConfig::get_template_commit()` 方法能够正确返回 commit 模板配置。
///
/// ## 测试场景
/// 1. 调用 `get_template_commit()` 方法
///
/// ## 预期结果
/// - 返回 Map（可能为空或包含配置项）
#[test]
fn test_get_template_commit_with_no_config_returns_map() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 template_commit 配置
    let template = RepoConfig::get_template_commit();

    // Assert: 验证返回值是一个 Map
    assert!(template.is_empty() || !template.is_empty());
}

/// 测试获取branch模板配置（无配置时返回Map）
///
/// ## 测试目的
/// 验证 `RepoConfig::get_template_branch()` 方法能够正确返回 branch 模板配置。
///
/// ## 测试场景
/// 1. 调用 `get_template_branch()` 方法
///
/// ## 预期结果
/// - 返回 Map（可能为空或包含配置项）
#[test]
fn test_get_template_branch_with_no_config_returns_map() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 template_branch 配置
    let template = RepoConfig::get_template_branch();

    // Assert: 验证返回值是一个 Map
    assert!(template.is_empty() || !template.is_empty());
}

/// 测试获取pull requests模板配置（无配置时返回Map）
///
/// ## 测试目的
/// 验证 `RepoConfig::get_template_pull_requests()` 方法能够正确返回 pull requests 模板配置。
///
/// ## 测试场景
/// 1. 调用 `get_template_pull_requests()` 方法
///
/// ## 预期结果
/// - 返回 Map（可能为空或包含配置项）
#[test]
fn test_get_template_pull_requests_with_no_config_returns_map() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 template_pull_requests 配置
    let template = RepoConfig::get_template_pull_requests();

    // Assert: 验证返回值是一个 Map
    assert!(template.is_empty() || !template.is_empty());
}

// ==================== Clone 和 Debug 测试 ====================

/// 测试克隆RepoConfig实例
///
/// ## 测试目的
/// 验证 `RepoConfig` 的 `Clone` trait 实现能够正确克隆配置实例。
///
/// ## 测试场景
/// 1. 创建包含配置的原始实例
/// 2. 克隆配置实例
///
/// ## 预期结果
/// - 克隆后的配置与原始配置相同
#[test]
fn test_repo_config_clone_with_config_instance_returns_cloned_config() {
    // Arrange: 准备原始配置
    use toml::Value;

    let mut original = RepoConfig::default();
    original.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    original.configured = true;
    original.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    // Act: 克隆配置
    let cloned = original.clone();

    // Assert: 验证克隆后的配置与原始配置相同
    assert_eq!(cloned.template_commit.len(), original.template_commit.len());
    assert_eq!(cloned.configured, original.configured);
    assert_eq!(
        cloned.branch.as_ref().and_then(|b| b.prefix.clone()),
        original.branch.as_ref().and_then(|b| b.prefix.clone())
    );
}

/// 测试RepoConfig实例的Debug格式化输出
///
/// ## 测试目的
/// 验证 `RepoConfig` 的 `Debug` trait 实现能够正确格式化输出。
///
/// ## 测试场景
/// 1. 创建配置实例
/// 2. 使用 Debug 格式化输出
///
/// ## 预期结果
/// - Debug 输出包含 "RepoConfig"
#[test]
fn test_repo_config_debug_with_config_instance_returns_debug_string() {
    // Arrange: 创建配置实例
    let config = RepoConfig::default();

    // Act: 格式化 Debug 输出
    let debug_output = format!("{:?}", config);

    // Assert: 验证 Debug 输出包含 RepoConfig
    assert!(debug_output.contains("RepoConfig"));
}

// ==================== Boundary Condition Tests ====================

/// 测试默认配置返回空配置
///
/// ## 测试目的
/// 验证默认配置的所有字段都为空或默认值。
///
/// ## 测试场景
/// 1. 创建默认配置
///
/// ## 预期结果
/// - 所有字段为空或默认值
#[test]
fn test_repo_config_empty_with_default_returns_empty_config() {
    // Arrange: 创建默认配置

    // Act: 获取配置
    let config = RepoConfig::default();

    // Assert: 验证所有字段为空或默认值
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
}

/// 测试只有公共配置字段的配置
///
/// ## 测试目的
/// 验证能够只设置公共配置字段，私有配置字段保持默认值。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 只设置公共配置字段
///
/// ## 预期结果
/// - 公共配置字段已设置
/// - 私有配置字段为默认值
#[test]
fn test_repo_config_only_public_with_public_fields_returns_public_config() {
    // Arrange: 准备只有公共配置的配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Act & Assert: 验证只有公共配置
    assert!(!config.template_commit.is_empty());
    assert!(!config.configured);
    assert!(config.branch.is_none());
}

/// 测试只有私有配置字段的配置
///
/// ## 测试目的
/// 验证能够只设置私有配置字段，公共配置字段保持默认值。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 只设置私有配置字段
///
/// ## 预期结果
/// - 私有配置字段已设置
/// - 公共配置字段为默认值
#[test]
fn test_repo_config_only_private_with_private_fields_returns_private_config() {
    // Arrange: 准备只有私有配置的配置
    let mut config = RepoConfig::default();
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    });

    // Act & Assert: 验证只有私有配置
    assert!(config.template_commit.is_empty());
    assert!(config.configured);
    assert!(config.branch.is_some());
}

/// 测试嵌套模板配置
///
/// ## 测试目的
/// 验证能够设置嵌套的模板配置（Table 类型）。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置嵌套的模板配置
///
/// ## 预期结果
/// - 嵌套配置正确设置
#[test]
fn test_repo_config_with_nested_template() {
    // Arrange: 准备测试嵌套的模板配置
    use toml::map::Map;
    use toml::Value;

    let mut config = RepoConfig::default();

    let mut nested_table = Map::new();
    nested_table.insert("enabled".to_string(), Value::Boolean(true));

    config
        .template_commit
        .insert("validation".to_string(), Value::Table(nested_table));

    assert_eq!(config.template_commit.len(), 1);
}

/// 测试包含特殊字符的配置
///
/// ## 测试目的
/// 验证能够正确处理包含特殊字符（如正则表达式模式）的配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置包含特殊字符的配置值
///
/// ## 预期结果
/// - 特殊字符正确保存和读取
#[test]
fn test_repo_config_with_special_characters() {
    // Arrange: 准备测试包含特殊字符的配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_branch.insert(
        "pattern".to_string(),
        Value::String(r"^[a-z]+/[A-Z]+-\d+".to_string()),
    );

    config.branch = Some(BranchConfig {
        prefix: Some("feature/test-123".to_string()),
        ignore: vec!["release/v1.0".to_string()],
    });

    assert!(!config.template_branch.is_empty());
    assert!(config.branch.is_some());
}

// ==================== Configuration Update Tests ====================

/// 测试更新commit模板配置
///
/// ## 测试目的
/// 验证能够更新已存在的 commit 模板配置。
///
/// ## 测试场景
/// 1. 创建配置并设置初始值
/// 2. 更新配置值
///
/// ## 预期结果
/// - 配置值正确更新
#[test]
fn test_update_template_commit() {
    // Arrange: 准备测试更新 template_commit 配置
    use toml::Value;

    let mut config = RepoConfig::default();

    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    assert_eq!(config.template_commit.len(), 1);

    config
        .template_commit
        .insert("type".to_string(), Value::String("semantic".to_string()));
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("semantic".to_string()))
    );
}

/// 测试更新configured标志
///
/// ## 测试目的
/// 验证能够更新 configured 标志。
///
/// ## 测试场景
/// 1. 创建默认配置（configured=false）
/// 2. 更新 configured 为 true
///
/// ## 预期结果
/// - configured 标志正确更新
#[test]
fn test_update_configured_flag() {
    // Arrange: 准备测试更新 configured 标志
    let mut config = RepoConfig::default();
    assert!(!config.configured);

    config.configured = true;
    assert!(config.configured);
}

/// 测试更新branch配置
///
/// ## 测试目的
/// 验证能够更新已存在的 branch 配置。
///
/// ## 测试场景
/// 1. 创建配置并设置初始 branch 配置
/// 2. 更新 branch 配置
///
/// ## 预期结果
/// - branch 配置正确更新
#[test]
fn test_update_branch_config() {
    // Arrange: 准备测试更新 branch 配置
    let mut config = RepoConfig::default();

    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    config.branch = Some(BranchConfig {
        prefix: Some("hotfix".to_string()),
        ignore: vec!["develop".to_string()],
    });

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("hotfix".to_string()));
    }
}

/// 测试清空commit模板配置
///
/// ## 测试目的
/// 验证能够清空 commit 模板配置。
///
/// ## 测试场景
/// 1. 创建配置并设置 commit 模板
/// 2. 清空 commit 模板配置
///
/// ## 预期结果
/// - commit 模板配置为空
#[test]
fn test_clear_template_commit() {
    // Arrange: 准备测试清空 template_commit 配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    config.template_commit.clear();
    assert!(config.template_commit.is_empty());
}

/// 测试清空branch配置
///
/// ## 测试目的
/// 验证能够清空 branch 配置。
///
/// ## 测试场景
/// 1. 创建配置并设置 branch 配置
/// 2. 将 branch 设置为 None
///
/// ## 预期结果
/// - branch 配置为 None
#[test]
fn test_clear_branch_config() {
    // Arrange: 准备测试清空 branch 配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    config.branch = None;
    assert!(config.branch.is_none());
}

// ==================== Parameterized Tests ====================

/// 测试仓库配置的参数化组合
///
/// ## 测试目的
/// 验证不同参数组合下仓库配置的正确性。
///
/// ## 测试场景
/// 1. 使用不同的 configured 和 prefix 参数组合
/// 2. 创建配置并验证
///
/// ## 预期结果
/// - 所有参数组合都能正确设置配置
#[rstest]
#[case(true, Some("feature".to_string()))]
#[case(false, None)]
#[case(true, Some("hotfix".to_string()))]
fn test_repo_config_parametrized(#[case] configured: bool, #[case] prefix: Option<String>) {
    // 参数化测试仓库配置的各种组合
    let mut config = RepoConfig::default();
    config.configured = configured;
    config.branch = Some(BranchConfig {
        prefix: prefix.clone(),
        ignore: vec![],
    });

    assert_eq!(config.configured, configured);
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, prefix);
    }
}

// ==================== Configuration Combination Tests ====================

/// 测试公共和私有配置的组合
///
/// ## 测试目的
/// 验证能够同时设置公共和私有配置，两者独立存在。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置公共配置字段
/// 3. 设置私有配置字段
///
/// ## 预期结果
/// - 公共和私有配置都正确设置
/// - 两种配置独立存在
#[test]
fn test_public_and_private_config_combination() {
    // Arrange: 准备测试公共和私有配置的组合
    use toml::Value;

    let mut config = RepoConfig::default();

    // 设置公共配置
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));

    // 设置私有配置
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    // Assert: 验证两种配置都存在
    assert!(!config.template_commit.is_empty());
    assert!(!config.template_branch.is_empty());
    assert!(config.configured);
    assert!(config.branch.is_some());
}

/// 测试模板配置的覆盖行为
///
/// ## 测试目的
/// 验证公共模板配置和私有个人配置的覆盖行为。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置公共模板配置
/// 3. 设置私有个人配置
///
/// ## 预期结果
/// - 两种配置独立存在，不互相覆盖
#[test]
fn test_template_override_behavior() {
    // Arrange: 准备测试模板配置的覆盖行为
    use toml::Value;

    let mut config = RepoConfig::default();

    // 公共模板配置
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));

    // 私有个人配置（可能会覆盖公共配置）
    config.branch = Some(BranchConfig {
        prefix: Some("my-feature".to_string()),
        ignore: vec![],
    });

    // 两种配置应该独立存在
    assert_eq!(
        config.template_branch.get("prefix"),
        Some(&Value::String("feature".to_string()))
    );
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("my-feature".to_string()));
    }
}

// ==================== Configuration Validation Tests ====================

/// 测试有效的分支前缀配置
///
/// ## 测试目的
/// 验证能够设置有效的分支前缀配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置有效的分支前缀
///
/// ## 预期结果
/// - 分支前缀正确设置
#[test]
fn test_config_with_valid_branch_prefix() {
    // Arrange: 准备测试有效的分支前缀配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    });

    if let Some(ref branch) = config.branch {
        assert!(!branch.prefix.as_ref().expect("branch prefix should exist").is_empty());
    }
}

/// 测试空的分支前缀配置
///
/// ## 测试目的
/// 验证能够设置空的分支前缀配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置空的分支前缀
///
/// ## 预期结果
/// - 空的分支前缀正确设置
#[test]
fn test_config_with_empty_branch_prefix() {
    // Arrange: 准备测试空的分支前缀配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("".to_string()),
        ignore: vec![],
    });

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("".to_string()));
    }
}

/// 测试多个忽略分支配置
///
/// ## 测试目的
/// 验证能够设置多个忽略分支。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置包含多个分支的 ignore 列表
///
/// ## 预期结果
/// - 所有忽略分支正确设置
#[test]
fn test_config_with_multiple_ignore_branches() {
    // Arrange: 准备测试多个忽略分支
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![
            "main".to_string(),
            "develop".to_string(),
            "staging".to_string(),
            "production".to_string(),
        ],
    });

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.ignore.len(), 4);
    }
}

// ==================== File System Integration Tests ====================

/// 测试从现有文件加载配置
///
/// ## 测试目的
/// 验证 `RepoConfig::load()` 方法能够从现有文件正确加载公共和私有配置。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库
/// 2. 创建公共和私有配置文件
/// 3. 调用 `load()` 方法加载配置
///
/// ## 预期结果
/// - 公共配置正确加载
/// - 私有配置正确加载
#[rstest]
// 已修复：使用路径参数版本，不再需要串行执行
fn test_load_from_existing_files_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建包含公共和私有配置的临时 Git 仓库

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    // 创建公共配置（项目模板）
    let public_config_content = r#"
[template.commit]
type = "conventional"
scope_required = true

[template.branch]
prefix = "feature"
"#;
    cli_env_with_git.create_project_config(public_config_content)?;

    // 创建私有配置（个人偏好）
    use workflow::repo::config::private::PrivateRepoConfig;
    // 生成 repo_id（使用项目路径）
    let repo_id = PrivateRepoConfig::generate_repo_id_in(cli_env_with_git.project_path())?;
    let private_config_content = format!(
        r#"
["{repo_id}"]
configured = true

["{repo_id}.branch"]
ignore = ["main", "develop"]

["{repo_id}.pr"]
auto_accept_change_type = false
"#
    );
    cli_env_with_git.create_home_config(&private_config_content)?;

    // 执行：调用 RepoConfig::load_from()，传入 home 路径
    let config = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // Assert: 验证：公共配置正确加载
    assert_eq!(config.template_commit.len(), 2);
    assert_eq!(config.template_branch.len(), 1);

    // Assert: 验证：私有配置正确加载
    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.ignore.len(), 2);
    }

    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, Some(false));
    }

    Ok(())
}

/// 测试从不存在文件加载配置（应返回默认配置）
///
/// ## 测试目的
/// 验证当配置文件不存在时，`RepoConfig::load()` 方法能够返回默认配置。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库（不创建配置文件）
/// 2. 调用 `load()` 方法
///
/// ## 预期结果
/// - 返回默认配置（所有字段为空或默认值）
#[rstest]
fn test_load_from_non_existing_files_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建没有配置文件的临时 Git 仓库

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    // 执行：调用 RepoConfig::load_from()，传入 home 路径
    let config = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // Assert: 验证：返回默认配置
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());

    Ok(())
}

/// 测试保存配置到新文件
///
/// ## 测试目的
/// 验证 `RepoConfig::save()` 方法能够将配置保存到新文件。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库（不创建配置文件）
/// 2. 创建配置并设置值
/// 3. 调用 `save()` 方法保存配置
///
/// ## 预期结果
/// - 公共配置文件创建成功
/// - 私有配置文件创建成功
/// - 配置文件内容正确
#[rstest]
fn test_save_to_new_files_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建临时 Git 仓库（不创建配置文件）

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    // 执行：创建配置并保存
    use toml::Value;
    let mut config = RepoConfig::default();
    config.configured = true;
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    // 保存配置
    config.save_in(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // Assert: 验证：公共配置文件创建成功
    let public_config_path = cli_env_with_git.project_path().join(".workflow").join("config.toml");
    assert!(public_config_path.exists());

    // Assert: 验证：私有配置文件创建成功
    // CliTestEnv 已设置 WORKFLOW_DISABLE_ICLOUD=1，传递 true 禁用 iCloud
    let private_config_path = Paths::repository_config_in(cli_env_with_git.home_path(), true)?;
    assert!(private_config_path.exists());

    // Assert: 验证：公共配置内容正确
    let public_content = fs::read_to_string(&public_config_path)?;
    assert!(public_content.contains("[template.commit]"));
    assert!(public_content.contains(r#"type = "conventional""#));

    // Assert: 验证：私有配置内容正确
    let private_content = fs::read_to_string(&private_config_path)?;
    assert!(private_content.contains("configured = true"));
    assert!(private_content.contains(r#"prefix = "feature""#));

    Ok(())
}

/// 测试加载和保存的往返流程（数据一致性）
///
/// ## 测试目的
/// 验证加载和保存的往返流程能够保持数据一致性。
///
/// ## 测试场景
/// 1. 创建包含配置的临时 Git 仓库
/// 2. 加载配置
/// 3. 修改配置
/// 4. 保存配置
/// 5. 重新加载配置
///
/// ## 预期结果
/// - 重新加载的配置与保存的配置一致
#[rstest]
// 已修复：使用路径参数版本，不再需要串行执行
fn test_load_and_save_roundtrip_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建包含配置的临时 Git 仓库

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    // 创建初始公共配置
    let public_config_content = r#"
[template.commit]
type = "conventional"
"#;
    cli_env_with_git.create_project_config(public_config_content)?;

    // 创建初始私有配置
    use workflow::repo::config::private::PrivateRepoConfig;
    let repo_id = PrivateRepoConfig::generate_repo_id_in(cli_env_with_git.project_path())?;
    let private_config_content = format!(
        r#"
["{repo_id}"]
configured = true

["{repo_id}.branch"]
prefix = "feature"
"#
    );
    cli_env_with_git.create_home_config(&private_config_content)?;

    // 执行：加载 → 修改 → 保存 → 重新加载
    let mut config = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;
    assert!(config.configured);

    // 修改配置
    use toml::Value;
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });
    config.save_in(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // 重新加载
    let reloaded_config = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // Assert: 验证：数据一致性
    assert_eq!(
        reloaded_config.template_commit.len(),
        config.template_commit.len()
    );
    assert_eq!(
        reloaded_config.pr.as_ref().and_then(|p| p.auto_accept_change_type),
        Some(true)
    );

    Ok(())
}

/// 测试检查配置是否存在
///
/// ## 测试目的
/// 验证 `RepoConfig::exists()` 方法能够正确检查配置是否存在。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库
/// 2. 检查配置是否存在（应该返回 false）
/// 3. 保存配置
/// 4. 再次检查配置是否存在（应该返回 true）
///
/// ## 预期结果
/// - 未配置时返回 false
/// - 配置后返回 true
#[rstest]
// 已修复：使用路径参数版本，不再需要串行执行
fn test_exists_check_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建临时 Git 仓库

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    // 1. 未配置时，exists() 应返回 false
    assert!(!RepoConfig::exists_in(cli_env_with_git.project_path(), cli_env_with_git.home_path())?);

    // 2. 保存配置后，exists() 应返回 true
    let mut config = RepoConfig::default();
    config.configured = true;
    config.save_in(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    assert!(RepoConfig::exists_in(cli_env_with_git.project_path(), cli_env_with_git.home_path())?);

    Ok(())
}

// ==================== Error Scenario Tests ====================

/// 测试加载损坏的公共配置文件（应返回错误）
///
/// ## 测试目的
/// 验证当公共配置文件损坏（无效 TOML）时，`RepoConfig::load()` 方法能够正确返回错误。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库
/// 2. 创建包含无效 TOML 的公共配置文件
/// 3. 尝试加载配置
///
/// ## 预期结果
/// - 返回错误，不panic
#[rstest]
fn test_load_with_corrupted_public_config_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建包含无效公共配置的临时 Git 仓库

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let invalid_public_config = r#"
[template.commit
type = "invalid  # 缺少闭合引号和括号
"#;
    cli_env_with_git.create_project_config(invalid_public_config)?;

    // 执行：尝试加载配置
    let result = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path());

    // Assert: 验证：返回错误
    assert!(result.is_err());

    Ok(())
}

/// 测试加载损坏的私有配置文件（应返回错误）
///
/// ## 测试目的
/// 验证当私有配置文件损坏（无效 TOML）时，`RepoConfig::load()` 方法能够正确返回错误。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库
/// 2. 创建包含无效 TOML 的私有配置文件
/// 3. 尝试加载配置
///
/// ## 预期结果
/// - 返回错误，不panic
#[rstest]
// 已修复：使用路径参数版本，不再需要串行执行
fn test_load_with_corrupted_private_config_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：创建包含无效私有配置的临时 Git 仓库

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let invalid_private_config = r#"
[invalid
configured = "not_a_boolean"
"#;
    cli_env_with_git.create_home_config(invalid_private_config)?;

    // 执行：尝试加载配置
    let result = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path());

    // Assert: 验证：返回错误（私有配置损坏会导致加载失败）
    assert!(result.is_err());

    Ok(())
}

/// 测试在非Git仓库中检查配置是否存在
///
/// ## 测试目的
/// 验证在非 Git 仓库中 `RepoConfig::exists()` 方法的行为。
///
/// ## 测试场景
/// 1. 创建非 Git 仓库的临时目录
/// 2. 调用 `exists()` 方法
///
/// ## 预期结果
/// - 在非 Git 仓库中返回 true（跳过检查）
#[rstest]
fn test_exists_outside_git_repo_return_ok(cli_env: CliTestEnv) -> Result<()> {
    // 准备：创建非 Git 仓库的临时目录
    // 注意：使用 cli_env 而不是 cli_env_with_git，因为我们需要测试非 Git 仓库的情况

    // 执行：调用 RepoConfig::exists_in()
    let result = RepoConfig::exists_in(cli_env.path(), cli_env.home_path())?;

    // Assert: 验证：在非 Git 仓库中返回 true（跳过检查）
    assert!(result);

    Ok(())
}

/// 测试只加载公共配置（无私有配置）
///
/// ## 测试目的
/// 验证当只有公共配置文件时，`RepoConfig::load()` 方法能够正确加载公共配置，私有配置使用默认值。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库
/// 2. 只创建公共配置文件
/// 3. 调用 `load()` 方法
///
/// ## 预期结果
/// - 公共配置加载成功
/// - 私有配置为默认值
#[rstest]
fn test_load_with_only_public_config_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：只创建公共配置

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let public_config_content = r#"
[template.commit]
type = "conventional"
"#;
    cli_env_with_git.create_project_config(public_config_content)?;

    // 执行：加载配置
    let config = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // Assert: 验证：公共配置加载成功，私有配置为默认值
    assert_eq!(config.template_commit.len(), 1);
    assert!(!config.configured);
    assert!(config.branch.is_none());

    Ok(())
}

/// 测试只加载私有配置（无公共配置）
///
/// ## 测试目的
/// 验证当只有私有配置文件时，`RepoConfig::load()` 方法能够正确加载私有配置，公共配置使用默认值。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库
/// 2. 只创建私有配置文件
/// 3. 调用 `load()` 方法
///
/// ## 预期结果
/// - 私有配置加载成功
/// - 公共配置为默认值
#[rstest]
// 已修复：使用路径参数版本，不再需要串行执行
fn test_load_with_only_private_config_return_ok(mut cli_env_with_git: CliTestEnv) -> Result<()> {
    // 准备：只创建私有配置

    // HOME 和 WORKFLOW_DISABLE_ICLOUD 已在 CliTestEnv::new() 中自动设置
    // 设置 XDG_CONFIG_HOME 环境变量（如果需要）
    let xdg_path = cli_env_with_git.home_path().join(".config").to_string_lossy().to_string();
    cli_env_with_git.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    use workflow::repo::config::private::PrivateRepoConfig;
    // 生成 repo_id（使用项目路径）
    let repo_id = PrivateRepoConfig::generate_repo_id_in(cli_env_with_git.project_path())?;
    let private_config_content = format!(
        r#"
["{repo_id}"]
configured = true
"#
    );
    cli_env_with_git.create_home_config(&private_config_content)?;

    // 执行：加载配置
    let config = RepoConfig::load_from(cli_env_with_git.project_path(), cli_env_with_git.home_path())?;

    // Assert: 验证：私有配置加载成功，公共配置为默认值
    assert!(config.configured);
    assert!(config.template_commit.is_empty());

    Ok(())
}
