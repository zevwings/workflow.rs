//! PublicRepoConfig 完整测试
//!
//! 包含数据结构测试和文件系统集成测试

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use toml::map::Map;
use toml::Value;
use workflow::repo::config::public::PublicRepoConfig;

use crate::common::environments::CliTestEnv;
use crate::common::helpers::CurrentDirGuard;

// ==================== 测试辅助函数 ====================

/// 创建公共配置文件（.workflow/config.toml）
fn create_public_config(env: &CliTestEnv, content: &str) -> Result<PathBuf> {
    let config_dir = env.path().join(".workflow");
    fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("config.toml");
    fs::write(&config_file, content)?;
    Ok(config_file)
}

// ==================== PublicRepoConfig Load 测试 ====================

/// 测试默认配置加载功能
///
/// ## 测试目的
/// 验证当没有配置文件时，PublicRepoConfig 能够返回默认配置。
///
/// ## 测试场景
/// 1. 创建默认配置实例
/// 2. 验证所有模板字段为空
///
/// ## 预期结果
/// - 所有模板字段（template_commit、template_branch、template_pull_requests）都为空
#[test]
fn test_load_public_config_default_with_no_config_returns_default_config() {
    // Arrange: 准备测试（无需额外准备）
    // 注意：这个测试依赖于当前目录没有 .workflow/config.toml
    // 由于 PublicRepoConfig::load() 依赖于 Paths::project_config()，
    // 我们无法轻易模拟不存在的情况，这里我们测试默认值的创建

    // Act: 创建默认配置
    let config = PublicRepoConfig::default();

    // Assert: 验证所有模板字段为空
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
}

/// 测试提交模板配置加载功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够正确设置和读取 commit 模板配置。
///
/// ## 测试场景
/// 1. 创建配置并设置 commit 模板字段
/// 2. 验证字段值正确保存和读取
///
/// ## 预期结果
/// - commit 模板字段能够正确设置和读取
#[test]
fn test_load_public_config_with_commit_template_returns_config_with_commit_template() {
    // Arrange: 准备 commit 模板配置
    // 由于 PublicRepoConfig::load() 使用 Paths::project_config()，
    // 我们直接测试配置结构的创建和字段设置
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));

    // Act & Assert: 验证 commit 模板配置
    assert_eq!(config.template_commit.len(), 2);
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );
    assert_eq!(
        config.template_commit.get("scope_required"),
        Some(&Value::Boolean(true))
    );
}

/// 测试分支模板配置加载功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够正确设置和读取 branch 模板配置。
///
/// ## 测试场景
/// 1. 创建配置并设置 branch 模板字段
/// 2. 验证字段值正确保存和读取
///
/// ## 预期结果
/// - branch 模板字段能够正确设置和读取
#[test]
fn test_load_public_config_with_branch_template_returns_config_with_branch_template() {
    // Arrange: 准备 branch 模板配置
    let mut config = PublicRepoConfig::default();
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config
        .template_branch
        .insert("separator".to_string(), Value::String("/".to_string()));

    // Act & Assert: 验证 branch 模板配置
    assert_eq!(config.template_branch.len(), 2);
    assert_eq!(
        config.template_branch.get("prefix"),
        Some(&Value::String("feature".to_string()))
    );
    assert_eq!(
        config.template_branch.get("separator"),
        Some(&Value::String("/".to_string()))
    );
}

/// 测试 PR 模板配置加载功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够正确设置和读取 PR 模板配置。
///
/// ## 测试场景
/// 1. 创建配置并设置 PR 模板字段
/// 2. 验证字段值正确保存和读取
///
/// ## 预期结果
/// - PR 模板字段能够正确设置和读取
#[test]
fn test_load_public_config_with_pr_template_returns_config_with_pr_template() {
    // Arrange: 准备 PR 模板配置
    let mut config = PublicRepoConfig::default();
    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));
    config
        .template_pull_requests
        .insert("require_review".to_string(), Value::Boolean(true));

    // Act & Assert: 验证 PR 模板配置
    assert_eq!(config.template_pull_requests.len(), 2);
    assert_eq!(
        config.template_pull_requests.get("auto_merge"),
        Some(&Value::Boolean(false))
    );
    assert_eq!(
        config.template_pull_requests.get("require_review"),
        Some(&Value::Boolean(true))
    );
}

/// 测试完整配置加载功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够同时设置所有模板配置。
///
/// ## 测试场景
/// 1. 创建配置并设置所有模板字段（commit、branch、PR）
/// 2. 验证所有模板都已正确设置
///
/// ## 预期结果
/// - 所有模板字段都被正确设置
#[test]
fn test_load_public_config_with_all_templates_returns_complete_config() {
    // Arrange: 准备所有模板配置
    let mut config = PublicRepoConfig::default();

    // 添加 commit 模板
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // 添加 branch 模板
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));

    // 添加 PR 模板
    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));

    // Act & Assert: 验证所有模板都已设置
    assert_eq!(config.template_commit.len(), 1);
    assert_eq!(config.template_branch.len(), 1);
    assert_eq!(config.template_pull_requests.len(), 1);
}

// ==================== PublicRepoConfig Save 测试 ====================

/// 测试配置保存结构完整性
///
/// ## 测试目的
/// 验证 PublicRepoConfig 保存时能够保持所有字段的完整性。
///
/// ## 测试场景
/// 1. 创建包含所有字段的配置
/// 2. 验证数据结构完整
///
/// ## 预期结果
/// - 所有字段都存在于配置结构中
#[test]
fn test_save_public_config_structure_with_all_fields_returns_complete_structure() {
    // Arrange: 准备包含所有字段的配置
    let mut config = PublicRepoConfig::default();

    // 添加测试数据
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

    // Act & Assert: 验证数据结构完整
    assert!(!config.template_commit.is_empty());
    assert!(!config.template_branch.is_empty());
    assert!(!config.template_pull_requests.is_empty());
}

// ==================== 配置字段测试 ====================

/// 测试提交模板字段类型支持
///
/// ## 测试目的
/// 验证 commit 模板字段能够支持多种 TOML 值类型（字符串、布尔、整数、数组）。
///
/// ## 测试场景
/// 1. 创建配置并添加不同类型的字段值
/// 2. 验证所有类型都能正确保存
///
/// ## 预期结果
/// - 字符串、布尔、整数、数组类型都能正确保存
#[test]
fn test_template_commit_fields_with_various_types_returns_config_with_fields() {
    // Arrange: 准备不同类型的字段值
    let mut config = PublicRepoConfig::default();

    // 字符串类型
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // 布尔类型
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));

    // 整数类型
    config.template_commit.insert("max_length".to_string(), Value::Integer(72));

    // 数组类型
    let types = vec![
        Value::String("feat".to_string()),
        Value::String("fix".to_string()),
        Value::String("docs".to_string()),
    ];
    config.template_commit.insert("allowed_types".to_string(), Value::Array(types));

    // Act & Assert: 验证所有字段都已添加
    assert_eq!(config.template_commit.len(), 4);
}

/// 测试分支模板字段类型支持
///
/// ## 测试目的
/// 验证 branch 模板字段能够支持多种 TOML 值类型。
///
/// ## 测试场景
/// 1. 创建配置并添加不同类型的字段值
/// 2. 验证所有类型都能正确保存
///
/// ## 预期结果
/// - 字符串、布尔类型都能正确保存
#[test]
fn test_template_branch_fields_with_various_types_returns_config_with_fields() {
    // Arrange: 准备不同类型的字段值
    let mut config = PublicRepoConfig::default();

    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config
        .template_branch
        .insert("separator".to_string(), Value::String("/".to_string()));
    config.template_branch.insert("use_jira_key".to_string(), Value::Boolean(true));

    // Act & Assert: 验证所有字段都已添加
    assert_eq!(config.template_branch.len(), 3);
}

/// 测试 PR 模板字段类型支持
///
/// ## 测试目的
/// 验证 PR 模板字段能够支持多种 TOML 值类型。
///
/// ## 测试场景
/// 1. 创建配置并添加不同类型的字段值
/// 2. 验证所有类型都能正确保存
///
/// ## 预期结果
/// - 布尔、整数类型都能正确保存
#[test]
fn test_template_pull_requests_fields_with_various_types_returns_config_with_fields() {
    // Arrange: 准备不同类型的字段值
    let mut config = PublicRepoConfig::default();

    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));
    config
        .template_pull_requests
        .insert("require_review".to_string(), Value::Boolean(true));
    config
        .template_pull_requests
        .insert("min_reviewers".to_string(), Value::Integer(2));

    // Act & Assert: 验证所有字段都已添加
    assert_eq!(config.template_pull_requests.len(), 3);
}

// ==================== 边界情况测试 ====================

/// 测试空配置默认值
///
/// ## 测试目的
/// 验证默认配置的所有字段都为空。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 验证所有字段为空
///
/// ## 预期结果
/// - 所有模板字段都为空
#[test]
fn test_empty_config_with_default_returns_empty_config() {
    // Arrange: 创建默认配置

    // Act: 获取配置
    let config = PublicRepoConfig::default();

    // Assert: 验证所有字段为空
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
}

/// 测试嵌套表格配置
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够正确处理嵌套的 TOML 表格结构。
///
/// ## 测试场景
/// 1. 创建包含嵌套表格的配置
/// 2. 验证嵌套结构正确保存和读取
///
/// ## 预期结果
/// - 嵌套表格结构能够正确保存和读取
#[test]
fn test_config_with_nested_tables_returns_config_with_nested_structure() {
    // Arrange: 准备嵌套表格配置
    let mut config = PublicRepoConfig::default();

    // 创建嵌套表格
    let mut nested_table = Map::new();
    nested_table.insert("enabled".to_string(), Value::Boolean(true));
    nested_table.insert("level".to_string(), Value::String("strict".to_string()));

    config
        .template_commit
        .insert("validation".to_string(), Value::Table(nested_table));

    // Act & Assert: 验证嵌套表格结构
    assert_eq!(config.template_commit.len(), 1);
    if let Some(Value::Table(table)) = config.template_commit.get("validation") {
        assert_eq!(table.len(), 2);
        assert_eq!(table.get("enabled"), Some(&Value::Boolean(true)));
    } else {
        panic!("Expected nested table");
    }
}

/// 测试特殊字符处理
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够正确处理包含特殊字符的配置值。
///
/// ## 测试场景
/// 1. 创建包含特殊字符（连字符、正则表达式）的配置
/// 2. 验证特殊字符被正确保存
///
/// ## 预期结果
/// - 特殊字符能够正确保存和读取
#[test]
fn test_config_with_special_characters_returns_config_with_special_chars() {
    // Arrange: 准备包含特殊字符的配置值
    let mut config = PublicRepoConfig::default();

    config.template_branch.insert(
        "prefix".to_string(),
        Value::String("feature/test-123".to_string()),
    );
    config.template_branch.insert(
        "pattern".to_string(),
        Value::String(r"^[a-z]+/[A-Z]+-\d+".to_string()),
    );

    // Act & Assert: 验证特殊字符被正确保存
    assert_eq!(config.template_branch.len(), 2);
}

/// 测试 Unicode 字符支持
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够正确处理 Unicode 字符（包括 emoji）。
///
/// ## 测试场景
/// 1. 创建包含 Unicode 字符的配置值
/// 2. 验证 Unicode 字符被正确保存和读取
///
/// ## 预期结果
/// - Unicode 字符能够正确保存和读取
#[test]
fn test_config_with_unicode_returns_config_with_unicode_chars() {
    // Arrange: 准备包含 Unicode 字符的配置值
    let mut config = PublicRepoConfig::default();

    config.template_commit.insert(
        "description".to_string(),
        Value::String("功能: 添加新特性 🚀".to_string()),
    );

    // Act & Assert: 验证 Unicode 字符被正确保存
    assert_eq!(config.template_commit.len(), 1);
    assert_eq!(
        config.template_commit.get("description"),
        Some(&Value::String("功能: 添加新特性 🚀".to_string()))
    );
}

// ==================== 配置更新测试 ====================

/// 测试字段更新功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够更新已存在的字段值。
///
/// ## 测试场景
/// 1. 创建配置并设置初始值
/// 2. 更新字段值
/// 3. 验证值已更新
///
/// ## 预期结果
/// - 字段值能够正确更新
#[test]
fn test_update_existing_field_with_new_value_updates_field() {
    // Arrange: 准备配置和初始值
    let mut config = PublicRepoConfig::default();

    // 初始值
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );

    // Act: 更新值
    config
        .template_commit
        .insert("type".to_string(), Value::String("semantic".to_string()));

    // Assert: 验证值已更新
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("semantic".to_string()))
    );
}

/// 测试字段删除功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够删除已存在的字段。
///
/// ## 测试场景
/// 1. 创建配置并添加字段
/// 2. 删除字段
/// 3. 验证字段已删除
///
/// ## 预期结果
/// - 字段能够正确删除
#[test]
fn test_remove_field_with_existing_field_removes_field() {
    // Arrange: 准备配置和字段
    let mut config = PublicRepoConfig::default();

    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    assert_eq!(config.template_commit.len(), 1);

    // Act: 删除字段
    config.template_commit.remove("type");

    // Assert: 验证字段已删除
    assert_eq!(config.template_commit.len(), 0);
}

/// 测试清空所有字段功能
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够清空所有模板字段。
///
/// ## 测试场景
/// 1. 创建包含所有字段的配置
/// 2. 清空所有字段
/// 3. 验证所有字段已清空
///
/// ## 预期结果
/// - 所有字段都被清空
#[test]
fn test_clear_all_fields_with_populated_config_clears_all_fields() {
    // Arrange: 准备包含所有字段的配置
    let mut config = PublicRepoConfig::default();

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

    // Act: 清空所有字段
    config.template_commit.clear();
    config.template_branch.clear();
    config.template_pull_requests.clear();

    // Assert: 验证所有字段已清空
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
}

// ==================== 参数化测试 ====================

/// 测试提交模板字段参数化
///
/// ## 测试目的
/// 使用参数化测试验证 commit 模板的各种字段类型。
///
/// ## 测试场景
/// 1. 使用不同字段名和值类型进行测试
/// 2. 验证字段能够正确插入和读取
///
/// ## 预期结果
/// - 所有字段类型都能正确插入和读取
#[rstest]
#[case("type", Value::String("conventional".to_string()))]
#[case("scope_required", Value::Boolean(true))]
#[case("max_length", Value::Integer(72))]
fn test_template_commit_parametrized_with_various_fields_returns_config_with_field(
    #[case] key: &str,
    #[case] value: Value,
) {
    // Arrange: 准备参数化测试数据
    // 参数化测试 template.commit 的各种字段

    // Act: 插入字段
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(key.to_string(), value.clone());

    // Assert: 验证字段已正确插入
    assert_eq!(config.template_commit.get(key), Some(&value));
}

/// 测试分支模板字段参数化
///
/// ## 测试目的
/// 使用参数化测试验证 branch 模板的各种字段类型。
///
/// ## 测试场景
/// 1. 使用不同字段名和值类型进行测试
/// 2. 验证字段能够正确插入和读取
///
/// ## 预期结果
/// - 所有字段类型都能正确插入和读取
#[rstest]
#[case("prefix", Value::String("feature".to_string()))]
#[case("separator", Value::String("/".to_string()))]
#[case("use_jira_key", Value::Boolean(true))]
fn test_template_branch_parametrized_with_various_fields_returns_config_with_field(
    #[case] key: &str,
    #[case] value: Value,
) {
    // Arrange: 准备参数化测试数据
    // 参数化测试 template.branch 的各种字段

    // Act: 插入字段
    let mut config = PublicRepoConfig::default();
    config.template_branch.insert(key.to_string(), value.clone());

    // Assert: 验证字段已正确插入
    assert_eq!(config.template_branch.get(key), Some(&value));
}

/// 测试 PR 模板字段参数化
///
/// ## 测试目的
/// 使用参数化测试验证 PR 模板的各种字段类型。
///
/// ## 测试场景
/// 1. 使用不同字段名和值类型进行测试
/// 2. 验证字段能够正确插入和读取
///
/// ## 预期结果
/// - 所有字段类型都能正确插入和读取
#[rstest]
#[case("auto_merge", Value::Boolean(false))]
#[case("require_review", Value::Boolean(true))]
#[case("min_reviewers", Value::Integer(2))]
fn test_template_pull_requests_parametrized_with_various_fields_returns_config_with_field(
    #[case] key: &str,
    #[case] value: Value,
) {
    // Arrange: 准备参数化测试数据
    // 参数化测试 template.pull_requests 的各种字段

    // Act: 插入字段
    let mut config = PublicRepoConfig::default();
    config.template_pull_requests.insert(key.to_string(), value.clone());

    // Assert: 验证字段已正确插入
    assert_eq!(config.template_pull_requests.get(key), Some(&value));
}

// ==================== Debug 和 Clone 测试 ====================

/// 测试配置 Debug 输出
///
/// ## 测试目的
/// 验证 PublicRepoConfig 的 Debug trait 实现正确。
///
/// ## 测试场景
/// 1. 创建配置实例
/// 2. 格式化 Debug 输出
/// 3. 验证输出包含配置类型名
///
/// ## 预期结果
/// - Debug 输出包含 "PublicRepoConfig"
#[test]
fn test_config_debug_with_config_instance_returns_debug_string() {
    // Arrange: 准备配置实例
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Act: 格式化 Debug 输出
    let debug_output = format!("{:?}", config);

    // Assert: 验证 Debug 输出包含 PublicRepoConfig
    assert!(debug_output.contains("PublicRepoConfig"));
}

/// 测试默认配置一致性
///
/// ## 测试目的
/// 验证多次调用 default() 返回的配置值一致。
///
/// ## 测试场景
/// 1. 创建多个默认配置实例
/// 2. 验证默认值一致
///
/// ## 预期结果
/// - 所有默认配置实例的值一致
#[test]
fn test_config_default_with_multiple_calls_returns_consistent_defaults() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 创建多个默认配置
    let config1 = PublicRepoConfig::default();
    let config2 = PublicRepoConfig::default();

    // Assert: 验证默认值一致
    assert!(config1.template_commit.is_empty());
    assert!(config2.template_commit.is_empty());
}

// ==================== 文件系统集成测试 ====================

/// 测试从文件加载配置
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够从文件系统加载有效的配置文件。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库和配置文件
/// 2. 调用 load() 加载配置
/// 3. 验证配置正确加载
///
/// ## 预期结果
/// - 配置能够正确从文件加载
#[test]
#[serial(repo_config_fs)] // 串行执行，避免工作目录冲突
fn test_load_from_existing_file_with_valid_config_returns_loaded_config() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    let config_content = r#"
[template.commit]
type = "conventional"
scope_required = true

[template.branch]
prefix = "feature"
separator = "/"
"#;
    create_public_config(&env, config_content)?;

    // Act: 切换到测试目录，然后调用 PublicRepoConfig::load()
    let _guard = CurrentDirGuard::new(env.path())?;
    let config = PublicRepoConfig::load()?;

    // Assert: 验证配置正确加载
    assert_eq!(config.template_commit.len(), 2);
    assert_eq!(config.template_branch.len(), 2);
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );
    assert_eq!(
        config.template_commit.get("scope_required"),
        Some(&Value::Boolean(true))
    );
    assert_eq!(
        config.template_branch.get("prefix"),
        Some(&Value::String("feature".to_string()))
    );

    Ok(())
}

/// 测试从不存在文件加载配置
///
/// ## 测试目的
/// 验证当配置文件不存在时，PublicRepoConfig 返回默认配置。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库（不创建配置文件）
/// 2. 调用 load() 加载配置
/// 3. 验证返回默认配置
///
/// ## 预期结果
/// - 返回默认配置（所有字段为空）
#[test]
#[serial(repo_config_fs)]
fn test_load_from_non_existing_file_returns_default_config() -> Result<()> {
    // Arrange: 创建没有配置文件的临时 Git 仓库
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // Act: 切换到测试目录，然后调用 PublicRepoConfig::load()
    let _guard = CurrentDirGuard::new(env.path())?;
    let config = PublicRepoConfig::load()?;

    // Assert: 验证返回默认配置
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());

    Ok(())
}

/// 测试保存配置到新文件
///
/// ## 测试目的
/// 验证 PublicRepoConfig 能够将配置保存到新文件。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库（不创建配置文件）
/// 2. 创建配置并保存
/// 3. 验证文件创建成功且内容正确
///
/// ## 预期结果
/// - 配置文件被创建且内容正确
#[test]
#[serial(repo_config_fs)]
fn test_save_to_new_file_with_config_creates_file() -> Result<()> {
    // Arrange: 创建临时 Git 仓库（不创建配置文件）
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 创建配置
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));

    // Act: 切换到测试目录，然后保存配置
    let _guard = CurrentDirGuard::new(env.path())?;
    config.save()?;

    // Assert: 验证文件创建成功，内容正确
    let config_path = env.path().join(".workflow/config.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("[template.commit]"));
    assert!(content.contains(r#"type = "conventional""#));
    assert!(content.contains("[template.branch]"));
    assert!(content.contains(r#"prefix = "feature""#));

    Ok(())
}

/// 测试保存配置时保留其他部分
///
/// ## 测试目的
/// 验证保存配置时不会覆盖配置文件中的其他部分。
///
/// ## 测试场景
/// 1. 创建包含其他配置部分的文件
/// 2. 保存 PublicRepoConfig
/// 3. 验证其他部分未被覆盖
///
/// ## 预期结果
/// - 其他配置部分被保留，模板配置已更新
#[test]
#[serial(repo_config_fs)]
fn test_save_preserves_other_sections_with_existing_config_preserves_other_sections() -> Result<()> {
    // Arrange: 创建包含其他配置部分的临时 Git 仓库
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    let config_content = r#"
[other_section]
key1 = "value1"
key2 = "value2"

[template.commit]
type = "old_type"
"#;
    create_public_config(&env, config_content)?;

    // 创建新配置
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));

    // Act: 切换到测试目录，然后保存配置
    let _guard = CurrentDirGuard::new(env.path())?;
    config.save()?;

    // Assert: 验证其他配置部分未被覆盖，模板配置已更新
    let content = fs::read_to_string(env.path().join(".workflow/config.toml"))?;
    assert!(content.contains("[other_section]"));
    assert!(content.contains(r#"key1 = "value1""#));
    assert!(content.contains(r#"key2 = "value2""#));
    assert!(content.contains("[template.commit]"));
    assert!(content.contains(r#"type = "conventional""#));
    assert!(content.contains("scope_required = true"));

    Ok(())
}

/// 测试配置加载和保存往返一致性
///
/// ## 测试目的
/// 验证配置的加载、修改、保存、重新加载过程保持数据一致性。
///
/// ## 测试场景
/// 1. 加载配置
/// 2. 修改配置
/// 3. 保存配置
/// 4. 重新加载配置
/// 5. 验证数据一致性
///
/// ## 预期结果
/// - 修改后的配置能够正确保存和重新加载
#[test]
#[serial(repo_config_fs)]
fn test_load_and_save_roundtrip_with_modified_config_returns_consistent_config() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    let config_content = r#"
[template.commit]
type = "conventional"
scope_required = true

[template.branch]
prefix = "feature"
separator = "/"

[template.pull_requests]
auto_merge = false
require_review = true
"#;
    create_public_config(&env, config_content)?;

    // Act: 切换到测试目录，然后加载 → 修改 → 保存 → 重新加载
    let _guard = CurrentDirGuard::new(env.path())?;
    let mut config = PublicRepoConfig::load()?;
    config.template_commit.insert("max_length".to_string(), Value::Integer(72));
    config.template_branch.insert("use_jira_key".to_string(), Value::Boolean(true));
    config.save()?;

    let reloaded_config = PublicRepoConfig::load()?;

    // Assert: 验证数据一致性
    assert_eq!(
        config.template_commit.len(),
        reloaded_config.template_commit.len()
    );
    assert_eq!(
        config.template_branch.len(),
        reloaded_config.template_branch.len()
    );
    assert_eq!(
        config.template_pull_requests.len(),
        reloaded_config.template_pull_requests.len()
    );
    assert_eq!(
        reloaded_config.template_commit.get("max_length"),
        Some(&Value::Integer(72))
    );
    assert_eq!(
        reloaded_config.template_branch.get("use_jira_key"),
        Some(&Value::Boolean(true))
    );

    Ok(())
}

// ==================== 错误场景测试 ====================

/// 测试加载损坏的 TOML 文件
///
/// ## 测试目的
/// 验证当配置文件包含无效 TOML 时，PublicRepoConfig 返回错误。
///
/// ## 测试场景
/// 1. 创建包含无效 TOML 的配置文件
/// 2. 尝试加载配置
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回 TOML 解析错误
#[test]
#[serial(repo_config_fs)]
fn test_load_corrupted_toml_file_with_invalid_toml_returns_error() -> Result<()> {
    // Arrange: 创建包含无效 TOML 的配置文件
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    let invalid_toml = r#"
[template.commit
type = "invalid  # 缺少闭合引号和括号
"#;
    create_public_config(&env, invalid_toml)?;

    // Act: 切换到测试目录，然后尝试加载配置
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = PublicRepoConfig::load();

    // Assert: 验证返回错误
    assert!(result.is_err());

    Ok(())
}

/// 测试保存到只读目录
///
/// ## 测试目的
/// 验证当目录为只读时，PublicRepoConfig 返回权限错误。
///
/// ## 测试场景
/// 1. 创建只读的 .workflow 目录
/// 2. 尝试保存配置
/// 3. 验证返回权限错误
///
/// ## 预期结果
/// - 返回文件系统权限错误
#[test]
#[cfg(unix)]
#[serial(repo_config_fs)]
fn test_save_to_readonly_directory_with_config_returns_error() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Arrange: 创建只读的 .workflow 目录
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    let workflow_dir = env.path().join(".workflow");
    fs::create_dir_all(&workflow_dir)?;

    // 设置目录为只读
    let mut perms = fs::metadata(&workflow_dir)?.permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&workflow_dir, perms)?;

    // 准备配置
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    // Act: 切换到测试目录，然后尝试保存配置
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = config.save();

    // Assert: 验证返回权限错误
    assert!(result.is_err());

    // 恢复权限以便清理
    let mut perms = fs::metadata(&workflow_dir)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&workflow_dir, perms)?;

    Ok(())
}
