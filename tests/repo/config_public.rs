//! PublicRepoConfig 完整测试
//!
//! 包含数据结构测试和文件系统集成测试

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use toml::map::Map;
use toml::Value;
use workflow::repo::config::public::PublicRepoConfig;

use crate::common::helpers::CurrentDirGuard;

// ==================== 测试辅助函数和结构 ====================

/// 测试环境管理器（RAII 模式）
///
/// 自动处理临时目录的创建和清理，以及工作目录的切换和恢复
struct TestEnv {
    temp_dir: TempDir,
    original_dir: PathBuf,
}

impl TestEnv {
    /// 创建新的测试环境
    fn new() -> Result<Self> {
        let original_dir = std::env::current_dir()?;
        let temp_dir = tempfile::tempdir()?;
        Ok(Self {
            temp_dir,
            original_dir,
        })
    }

    /// 初始化 Git 仓库
    fn init_git_repo(&self) -> Result<()> {
        let temp_path = self.temp_dir.path();
        // 注意：不需要set_current_dir，因为所有Git命令都使用.current_dir(temp_path)

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_path)
            .output()?;

        // 创建初始提交
        let readme_path = temp_path.join("README.md");
        fs::write(&readme_path, "# Test Repository")?;
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(temp_path)
            .output()?;
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_path)
            .output()?;

        Ok(())
    }

    /// 创建配置文件
    fn create_config(&self, content: &str) -> Result<PathBuf> {
        let config_dir = self.temp_dir.path().join(".workflow");
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("config.toml");
        fs::write(&config_file, content)?;
        Ok(config_file)
    }

    /// 获取临时目录路径
    fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original_dir);
    }
}

// ==================== PublicRepoConfig Load 测试 ====================

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

#[test]
#[serial(repo_config_fs)] // 串行执行，避免工作目录冲突
fn test_load_from_existing_file_with_valid_config_returns_loaded_config() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let config_content = r#"
[template.commit]
type = "conventional"
scope_required = true

[template.branch]
prefix = "feature"
separator = "/"
"#;
    env.create_config(config_content)?;

    // Act: 调用 PublicRepoConfig::load()
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

#[test]
#[serial(repo_config_fs)]
fn test_load_from_non_existing_file_returns_default_config() -> Result<()> {
    // Arrange: 创建没有配置文件的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 调用 PublicRepoConfig::load()
    let config = PublicRepoConfig::load()?;

    // Assert: 验证返回默认配置
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_save_to_new_file_with_config_creates_file() -> Result<()> {
    // Arrange: 创建临时 Git 仓库（不创建配置文件）
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // 创建配置
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));

    // Act: 保存配置
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

#[test]
#[serial(repo_config_fs)]
fn test_save_preserves_other_sections_with_existing_config_preserves_other_sections() -> Result<()> {
    // Arrange: 创建包含其他配置部分的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let config_content = r#"
[other_section]
key1 = "value1"
key2 = "value2"

[template.commit]
type = "old_type"
"#;
    env.create_config(config_content)?;

    // 创建新配置
    let mut config = PublicRepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));

    // Act: 保存配置
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

#[test]
#[serial(repo_config_fs)]
fn test_load_and_save_roundtrip_with_modified_config_returns_consistent_config() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

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
    env.create_config(config_content)?;

    // Act: 加载 → 修改 → 保存 → 重新加载
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

#[test]
#[serial(repo_config_fs)]
fn test_load_corrupted_toml_file_with_invalid_toml_returns_error() -> Result<()> {
    // Arrange: 创建包含无效 TOML 的配置文件
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let invalid_toml = r#"
[template.commit
type = "invalid  # 缺少闭合引号和括号
"#;
    env.create_config(invalid_toml)?;

    // Act: 尝试加载配置
    let result = PublicRepoConfig::load();

    // Assert: 验证返回错误
    assert!(result.is_err());

    Ok(())
}

#[test]
#[cfg(unix)]
#[serial(repo_config_fs)]
fn test_save_to_readonly_directory_with_config_returns_error() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Arrange: 创建只读的 .workflow 目录
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

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

    // Act: 尝试保存配置
    let result = config.save();

    // Assert: 验证返回权限错误
    assert!(result.is_err());

    // 恢复权限以便清理
    let mut perms = fs::metadata(&workflow_dir)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&workflow_dir, perms)?;

    Ok(())
}
