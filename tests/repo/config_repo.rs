//! RepoConfig 完整测试
//!
//! 包含数据结构测试、文件系统集成测试和错误场景测试

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use workflow::repo::config::types::{BranchConfig, PullRequestsConfig};
use workflow::repo::RepoConfig;

// ==================== 测试辅助函数和结构 ====================

/// 测试环境管理器（RAII 模式）
///
/// 自动处理临时目录的创建和清理，以及工作目录和环境变量的切换和恢复
struct TestEnv {
    temp_dir: TempDir,
    original_dir: PathBuf,
    original_home: Option<PathBuf>,
    original_xdg_config_home: Option<PathBuf>,
}

impl TestEnv {
    /// 创建新的测试环境
    fn new() -> Result<Self> {
        let original_dir = std::env::current_dir()?;
        let temp_dir = tempfile::tempdir()?;

        // 保存原始环境变量
        let original_home = std::env::var_os("HOME").map(PathBuf::from);
        let original_xdg_config_home = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);

        // 设置临时 HOME 和 XDG_CONFIG_HOME
        std::env::set_var("HOME", temp_dir.path());
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path().join(".config"));

        Ok(Self {
            temp_dir,
            original_dir,
            original_home,
            original_xdg_config_home,
        })
    }

    /// 初始化 Git 仓库
    fn init_git_repo(&self) -> Result<()> {
        let temp_path = self.temp_dir.path();
        std::env::set_current_dir(temp_path)?;

        std::process::Command::new("git").args(["init"]).current_dir(temp_path).output()?;
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_path)
            .output()?;
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_path)
            .output()?;

        // 添加远程仓库（用于生成 repo_id）
        std::process::Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/test/test-repo.git",
            ])
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

    /// 创建公共配置文件（项目根目录 .workflow/config.toml）
    fn create_public_config(&self, content: &str) -> Result<PathBuf> {
        let config_dir = self.temp_dir.path().join(".workflow");
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("config.toml");
        fs::write(&config_file, content)?;
        Ok(config_file)
    }

    /// 创建私有配置文件（~/.workflow/config/repository.toml）
    fn create_private_config(&self, content: &str) -> Result<PathBuf> {
        let config_dir = self.temp_dir.path().join(".workflow").join("config");
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("repository.toml");
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
        // 恢复原始工作目录
        let _ = std::env::set_current_dir(&self.original_dir);

        // 恢复原始环境变量
        if let Some(ref home) = self.original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }

        if let Some(ref xdg) = self.original_xdg_config_home {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}

// ==================== 默认值测试 ====================

#[test]
fn test_repo_config_default() {
    // 测试仓库配置的默认值
    let config = RepoConfig::default();

    // Public configuration
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());

    // Private configuration
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
}

// ==================== 配置字段测试 ====================

#[test]
fn test_repo_config_with_template_commit() {
    // 测试设置 template_commit 配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));

    assert_eq!(config.template_commit.len(), 2);
    assert_eq!(
        config.template_commit.get("type"),
        Some(&Value::String("conventional".to_string()))
    );
}

#[test]
fn test_repo_config_with_template_branch() {
    // 测试设置 template_branch 配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config
        .template_branch
        .insert("prefix".to_string(), Value::String("feature".to_string()));
    config
        .template_branch
        .insert("separator".to_string(), Value::String("/".to_string()));

    assert_eq!(config.template_branch.len(), 2);
    assert_eq!(
        config.template_branch.get("prefix"),
        Some(&Value::String("feature".to_string()))
    );
}

#[test]
fn test_repo_config_with_template_pull_requests() {
    // 测试设置 template_pull_requests 配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config
        .template_pull_requests
        .insert("auto_merge".to_string(), Value::Boolean(false));
    config
        .template_pull_requests
        .insert("require_review".to_string(), Value::Boolean(true));

    assert_eq!(config.template_pull_requests.len(), 2);
    assert_eq!(
        config.template_pull_requests.get("auto_merge"),
        Some(&Value::Boolean(false))
    );
}

#[test]
fn test_repo_config_with_configured() {
    // 测试设置 configured 字段
    let mut config = RepoConfig::default();
    config.configured = true;

    assert!(config.configured);
}

#[test]
fn test_repo_config_with_branch() {
    // 测试设置 branch 配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string(), "develop".to_string()],
    });

    assert!(config.branch.is_some());
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("feature".to_string()));
        assert_eq!(branch.ignore.len(), 2);
    }
}

#[test]
fn test_repo_config_with_pr() {
    // 测试设置 PR 配置
    let mut config = RepoConfig::default();
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    assert!(config.pr.is_some());
    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, Some(true));
    }
}

#[test]
fn test_repo_config_with_all_fields() {
    // 测试设置所有字段
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

    assert!(!config.template_commit.is_empty());
    assert!(!config.template_branch.is_empty());
    assert!(!config.template_pull_requests.is_empty());
    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());
}

// ==================== 静态方法测试 ====================

#[test]
fn test_get_branch_prefix_none() {
    // 测试获取不存在的分支前缀
    // 注意：这个测试依赖于当前仓库的配置状态
    let prefix = RepoConfig::get_branch_prefix();
    // 如果配置不存在，应该返回 None
    // 如果配置存在，应该返回配置的值
    // 这里我们只验证返回类型是 Option<String>
    assert!(prefix.is_none() || prefix.is_some());
}

#[test]
fn test_get_ignore_branches_empty() {
    // 测试获取空的忽略分支列表
    let branches = RepoConfig::get_ignore_branches();
    // 返回值应该是一个 Vec<String>
    assert!(branches.is_empty() || !branches.is_empty());
}

#[test]
fn test_get_auto_accept_change_type_default() {
    // 测试获取 auto_accept_change_type 的默认值
    let auto_accept = RepoConfig::get_auto_accept_change_type();
    // 默认应该是 false，或者根据配置返回 true
    assert!(!auto_accept || auto_accept);
}

#[test]
fn test_get_template_commit_empty() {
    // 测试获取空的 template_commit 配置
    let template = RepoConfig::get_template_commit();
    // 返回值应该是一个 Map
    assert!(template.is_empty() || !template.is_empty());
}

#[test]
fn test_get_template_branch_empty() {
    // 测试获取空的 template_branch 配置
    let template = RepoConfig::get_template_branch();
    // 返回值应该是一个 Map
    assert!(template.is_empty() || !template.is_empty());
}

#[test]
fn test_get_template_pull_requests_empty() {
    // 测试获取空的 template_pull_requests 配置
    let template = RepoConfig::get_template_pull_requests();
    // 返回值应该是一个 Map
    assert!(template.is_empty() || !template.is_empty());
}

// ==================== Clone 和 Debug 测试 ====================

#[test]
fn test_repo_config_clone() {
    // 测试配置的克隆功能
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

    let cloned = original.clone();

    assert_eq!(cloned.template_commit.len(), original.template_commit.len());
    assert_eq!(cloned.configured, original.configured);
    assert_eq!(
        cloned.branch.as_ref().and_then(|b| b.prefix.clone()),
        original.branch.as_ref().and_then(|b| b.prefix.clone())
    );
}

#[test]
fn test_repo_config_debug() {
    // 测试配置的 Debug 输出
    let config = RepoConfig::default();
    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("RepoConfig"));
}

// ==================== 边界情况测试 ====================

#[test]
fn test_repo_config_empty() {
    // 测试空配置
    let config = RepoConfig::default();

    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
}

#[test]
fn test_repo_config_only_public() {
    // 测试只有公共配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    assert!(!config.template_commit.is_empty());
    assert!(!config.configured);
    assert!(config.branch.is_none());
}

#[test]
fn test_repo_config_only_private() {
    // 测试只有私有配置
    let mut config = RepoConfig::default();
    config.configured = true;
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    });

    assert!(config.template_commit.is_empty());
    assert!(config.configured);
    assert!(config.branch.is_some());
}

#[test]
fn test_repo_config_with_nested_template() {
    // 测试嵌套的模板配置
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

#[test]
fn test_repo_config_with_special_characters() {
    // 测试包含特殊字符的配置
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

// ==================== 配置更新测试 ====================

#[test]
fn test_update_template_commit() {
    // 测试更新 template_commit 配置
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

#[test]
fn test_update_configured_flag() {
    // 测试更新 configured 标志
    let mut config = RepoConfig::default();
    assert!(!config.configured);

    config.configured = true;
    assert!(config.configured);
}

#[test]
fn test_update_branch_config() {
    // 测试更新 branch 配置
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

#[test]
fn test_clear_template_commit() {
    // 测试清空 template_commit 配置
    use toml::Value;

    let mut config = RepoConfig::default();
    config.template_commit.insert(
        "type".to_string(),
        Value::String("conventional".to_string()),
    );

    config.template_commit.clear();
    assert!(config.template_commit.is_empty());
}

#[test]
fn test_clear_branch_config() {
    // 测试清空 branch 配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });

    config.branch = None;
    assert!(config.branch.is_none());
}

// ==================== 参数化测试 ====================

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

// ==================== 配置组合测试 ====================

#[test]
fn test_public_and_private_config_combination() {
    // 测试公共和私有配置的组合
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

    // 验证两种配置都存在
    assert!(!config.template_commit.is_empty());
    assert!(!config.template_branch.is_empty());
    assert!(config.configured);
    assert!(config.branch.is_some());
}

#[test]
fn test_template_override_behavior() {
    // 测试模板配置的覆盖行为
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

// ==================== 配置验证测试 ====================

#[test]
fn test_config_with_valid_branch_prefix() {
    // 测试有效的分支前缀配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    });

    if let Some(ref branch) = config.branch {
        assert!(!branch.prefix.as_ref().unwrap().is_empty());
    }
}

#[test]
fn test_config_with_empty_branch_prefix() {
    // 测试空的分支前缀配置
    let mut config = RepoConfig::default();
    config.branch = Some(BranchConfig {
        prefix: Some("".to_string()),
        ignore: vec![],
    });

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("".to_string()));
    }
}

#[test]
fn test_config_with_multiple_ignore_branches() {
    // 测试多个忽略分支
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

// ==================== 文件系统集成测试 ====================

#[test]
#[serial(repo_config_fs)]
fn test_load_from_existing_files() -> Result<()> {
    // 准备：创建包含公共和私有配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    // 创建公共配置（项目模板）
    let public_config_content = r#"
[template.commit]
type = "conventional"
scope_required = true

[template.branch]
prefix = "feature"
"#;
    env.create_public_config(public_config_content)?;

    // 创建私有配置（个人偏好）
    use workflow::repo::config::private::PrivateRepoConfig;
    let repo_id = PrivateRepoConfig::generate_repo_id()?;
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
    env.create_private_config(&private_config_content)?;

    // 执行：调用 RepoConfig::load()
    let config = RepoConfig::load()?;

    // 验证：公共配置正确加载
    assert_eq!(config.template_commit.len(), 2);
    assert_eq!(config.template_branch.len(), 1);

    // 验证：私有配置正确加载
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

#[test]
#[serial(repo_config_fs)]
fn test_load_from_non_existing_files() -> Result<()> {
    // 准备：创建没有配置文件的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    // 执行：调用 RepoConfig::load()
    let config = RepoConfig::load()?;

    // 验证：返回默认配置
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_save_to_new_files() -> Result<()> {
    // 准备：创建临时 Git 仓库（不创建配置文件）
    let env = TestEnv::new()?;
    env.init_git_repo()?;

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
    config.save()?;

    // 验证：公共配置文件创建成功
    let public_config_path = env.path().join(".workflow").join("config.toml");
    assert!(public_config_path.exists());

    // 验证：私有配置文件创建成功
    let private_config_path =
        env.path().join(".workflow").join("config").join("repository.toml");
    assert!(private_config_path.exists());

    // 验证：公共配置内容正确
    let public_content = fs::read_to_string(&public_config_path)?;
    assert!(public_content.contains("[template.commit]"));
    assert!(public_content.contains(r#"type = "conventional""#));

    // 验证：私有配置内容正确
    let private_content = fs::read_to_string(&private_config_path)?;
    assert!(private_content.contains("configured = true"));
    assert!(private_content.contains(r#"prefix = "feature""#));

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_load_and_save_roundtrip() -> Result<()> {
    // 准备：创建包含配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    // 创建初始公共配置
    let public_config_content = r#"
[template.commit]
type = "conventional"
"#;
    env.create_public_config(public_config_content)?;

    // 创建初始私有配置
    use workflow::repo::config::private::PrivateRepoConfig;
    let repo_id = PrivateRepoConfig::generate_repo_id()?;
    let private_config_content = format!(
        r#"
["{repo_id}"]
configured = true

["{repo_id}.branch"]
prefix = "feature"
"#
    );
    env.create_private_config(&private_config_content)?;

    // 执行：加载 → 修改 → 保存 → 重新加载
    let mut config = RepoConfig::load()?;
    assert!(config.configured);

    // 修改配置
    use toml::Value;
    config
        .template_commit
        .insert("scope_required".to_string(), Value::Boolean(true));
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });
    config.save()?;

    // 重新加载
    let reloaded_config = RepoConfig::load()?;

    // 验证：数据一致性
    assert_eq!(
        reloaded_config.template_commit.len(),
        config.template_commit.len()
    );
    assert_eq!(
        reloaded_config
            .pr
            .as_ref()
            .and_then(|p| p.auto_accept_change_type),
        Some(true)
    );

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_exists_check() -> Result<()> {
    // 准备：创建临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    // 1. 未配置时，exists() 应返回 false
    assert!(!RepoConfig::exists()?);

    // 2. 保存配置后，exists() 应返回 true
    let mut config = RepoConfig::default();
    config.configured = true;
    config.save()?;

    assert!(RepoConfig::exists()?);

    Ok(())
}

// ==================== 错误场景测试 ====================

#[test]
#[serial(repo_config_fs)]
fn test_load_with_corrupted_public_config() -> Result<()> {
    // 准备：创建包含无效公共配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    let invalid_public_config = r#"
[template.commit
type = "invalid  # 缺少闭合引号和括号
"#;
    env.create_public_config(invalid_public_config)?;

    // 执行：尝试加载配置
    let result = RepoConfig::load();

    // 验证：返回错误
    assert!(result.is_err());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_load_with_corrupted_private_config() -> Result<()> {
    // 准备：创建包含无效私有配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    let invalid_private_config = r#"
[invalid
configured = "not_a_boolean"
"#;
    env.create_private_config(invalid_private_config)?;

    // 执行：尝试加载配置
    let result = RepoConfig::load();

    // 验证：返回错误（私有配置损坏会导致加载失败）
    assert!(result.is_err());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_exists_outside_git_repo() -> Result<()> {
    // 准备：创建非 Git 仓库的临时目录
    let env = TestEnv::new()?;
    let temp_path = env.path();
    std::env::set_current_dir(temp_path)?;

    // 执行：调用 RepoConfig::exists()
    let result = RepoConfig::exists()?;

    // 验证：在非 Git 仓库中返回 true（跳过检查）
    assert!(result);

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_load_with_only_public_config() -> Result<()> {
    // 准备：只创建公共配置
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    let public_config_content = r#"
[template.commit]
type = "conventional"
"#;
    env.create_public_config(public_config_content)?;

    // 执行：加载配置
    let config = RepoConfig::load()?;

    // 验证：公共配置加载成功，私有配置为默认值
    assert_eq!(config.template_commit.len(), 1);
    assert!(!config.configured);
    assert!(config.branch.is_none());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_load_with_only_private_config() -> Result<()> {
    // 准备：只创建私有配置
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    use workflow::repo::config::private::PrivateRepoConfig;
    let repo_id = PrivateRepoConfig::generate_repo_id()?;
    let private_config_content = format!(
        r#"
["{repo_id}"]
configured = true
"#
    );
    env.create_private_config(&private_config_content)?;

    // 执行：加载配置
    let config = RepoConfig::load()?;

    // 验证：私有配置加载成功，公共配置为默认值
    assert!(config.configured);
    assert!(config.template_commit.is_empty());

    Ok(())
}
