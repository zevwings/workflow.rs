//! PrivateRepoConfig 完整测试
//!
//! 包含数据结构测试、文件系统集成测试和错误场景测试

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use workflow::repo::config::private::PrivateRepoConfig;
use workflow::repo::config::types::{BranchConfig, PullRequestsConfig};

use crate::common::helpers::CurrentDirGuard;

// ==================== 测试辅助函数和结构 ====================

/// 测试环境管理器（RAII 模式）
///
/// ## 功能说明
/// 自动处理临时目录的创建和清理，以及工作目录和环境变量的切换和恢复。
/// 使用 RAII（Resource Acquisition Is Initialization）模式确保资源的正确清理。
///
/// ## 设计原理
/// 1. **自动清理**：利用 Rust 的 Drop trait，在对象销毁时自动恢复环境
/// 2. **环境隔离**：每个测试在独立的临时目录和环境变量中运行
/// 3. **Git 集成**：支持在临时目录中初始化 Git 仓库
/// 4. **配置管理**：提供创建和管理配置文件的辅助方法
///
/// ## 使用示例
/// ```rust
/// let env = TestEnv::new()?;
/// env.init_git_repo()?;
/// env.create_config("key = value")?;
/// // 测试代码...
/// // 自动清理：当 env 离开作用域时，Drop trait 会恢复所有环境
/// ```
///
/// ## 管理的资源
/// - `temp_dir`: 临时目录（自动清理）
/// - `original_dir`: 原始工作目录（自动恢复）
/// - `original_home`: 原始 HOME 环境变量（自动恢复）
/// - `original_xdg_config_home`: 原始 XDG_CONFIG_HOME 环境变量（自动恢复）
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

    /// 创建配置文件
    fn create_config(&self, content: &str) -> Result<PathBuf> {
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

    /// 获取配置文件路径
    fn config_path(&self) -> PathBuf {
        self.temp_dir.path().join(".workflow").join("config").join("repository.toml")
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
fn test_private_config_default() {
    // Arrange: 准备测试私有配置的默认值
    let config = PrivateRepoConfig::default();

    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
}

// ==================== 配置字段测试 ====================

#[test]
fn test_private_config_with_configured() {
    // Arrange: 准备测试设置 configured 字段
    let mut config = PrivateRepoConfig::default();
    config.configured = true;

    assert!(config.configured);
}

#[test]
fn test_private_config_with_branch() {
    // Arrange: 准备测试设置 branch 配置
    let mut config = PrivateRepoConfig::default();
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
fn test_private_config_with_pr() {
    // Arrange: 准备测试设置 PR 配置
    let mut config = PrivateRepoConfig::default();
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });

    assert!(config.pr.is_some());
    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, Some(true));
    }
}

#[test]
fn test_private_config_with_all_fields() {
    // Arrange: 准备测试设置所有字段
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec!["main".to_string()],
        }),
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: Some(false),
        }),
    };

    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());
}

// ==================== 仓库 ID 生成测试 ====================

#[test]
fn test_generate_repo_id_format() {
    // Arrange: 准备测试仓库 ID 的格式
    // 注意：这个测试需要在 Git 仓库中运行
    if let Ok(repo_id) = PrivateRepoConfig::generate_repo_id() {
        // 验证格式：{repo_name}_{hash}
        assert!(repo_id.contains('_'));

        let parts: Vec<&str> = repo_id.split('_').collect();
        assert!(parts.len() >= 2);

        // 验证 hash 部分是 8 个字符的十六进制
        if let Some(hash_part) = parts.last() {
            assert_eq!(hash_part.len(), 8);
            assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}

#[test]
fn test_generate_repo_id_consistency() {
    // Arrange: 准备测试同一仓库生成的 ID 应该一致
    if let (Ok(id1), Ok(id2)) = (
        PrivateRepoConfig::generate_repo_id(),
        PrivateRepoConfig::generate_repo_id(),
    ) {
        assert_eq!(id1, id2);
    }
}

// ==================== Clone 和 Debug 测试 ====================

#[test]
fn test_private_config_clone() {
    // Arrange: 准备测试配置的克隆功能
    let original = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec!["main".to_string()],
        }),
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: Some(true),
        }),
    };

    let cloned = original.clone();

    assert_eq!(cloned.configured, original.configured);
    assert_eq!(
        cloned.branch.as_ref().and_then(|b| b.prefix.clone()),
        original.branch.as_ref().and_then(|b| b.prefix.clone())
    );
    assert_eq!(
        cloned.pr.as_ref().and_then(|p| p.auto_accept_change_type),
        original.pr.as_ref().and_then(|p| p.auto_accept_change_type)
    );
}

#[test]
fn test_private_config_debug() {
    // Arrange: 准备测试配置的 Debug 输出
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec!["main".to_string()],
        }),
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: Some(true),
        }),
    };

    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("PrivateRepoConfig"));
    assert!(debug_output.contains("configured"));
}

// ==================== 边界情况测试 ====================

#[test]
fn test_private_config_with_empty_branch() {
    // Arrange: 准备测试空的 branch 配置
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: None,
            ignore: vec![],
        }),
        pr: None,
    };

    assert!(config.configured);
    assert!(config.branch.is_some());
    if let Some(ref branch) = config.branch {
        assert!(branch.prefix.is_none());
        assert!(branch.ignore.is_empty());
    }
}

#[test]
fn test_private_config_with_empty_pr() {
    // Arrange: 准备测试空的 PR 配置
    let config = PrivateRepoConfig {
        configured: true,
        branch: None,
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: None,
        }),
    };

    assert!(config.configured);
    assert!(config.pr.is_some());
    if let Some(ref pr) = config.pr {
        assert!(pr.auto_accept_change_type.is_none());
    }
}

#[test]
fn test_private_config_with_multiple_ignore_branches() {
    // Arrange: 准备测试多个忽略分支
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec![
                "main".to_string(),
                "develop".to_string(),
                "staging".to_string(),
                "production".to_string(),
            ],
        }),
        pr: None,
    };

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.ignore.len(), 4);
        assert!(branch.ignore.contains(&"main".to_string()));
        assert!(branch.ignore.contains(&"develop".to_string()));
        assert!(branch.ignore.contains(&"staging".to_string()));
        assert!(branch.ignore.contains(&"production".to_string()));
    }
}

#[test]
fn test_private_config_with_special_branch_prefix() {
    // Arrange: 准备测试特殊字符的分支前缀
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature/test-123".to_string()),
            ignore: vec![],
        }),
        pr: None,
    };

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("feature/test-123".to_string()));
    }
}

// ==================== 参数化测试 ====================

#[rstest]
#[case(true, Some("feature".to_string()), vec!["main".to_string()])]
#[case(false, None, vec![])]
#[case(true, Some("hotfix".to_string()), vec![])]
#[case(false, Some("bugfix".to_string()), vec!["main".to_string(), "develop".to_string()])]
fn test_private_config_parametrized(
    #[case] configured: bool,
    #[case] prefix: Option<String>,
    #[case] ignore: Vec<String>,
) {
    // 参数化测试私有配置的各种组合
    let config = PrivateRepoConfig {
        configured,
        branch: Some(BranchConfig {
            prefix: prefix.clone(),
            ignore: ignore.clone(),
        }),
        pr: None,
    };

    assert_eq!(config.configured, configured);
    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, prefix);
        assert_eq!(branch.ignore, ignore);
    }
}

#[rstest]
#[case(Some(true))]
#[case(Some(false))]
#[case(None)]
fn test_private_config_pr_parametrized(#[case] auto_accept: Option<bool>) {
    // 参数化测试 PR 配置的各种值
    let config = PrivateRepoConfig {
        configured: true,
        branch: None,
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: auto_accept,
        }),
    };

    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, auto_accept);
    }
}

// ==================== 配置更新测试 ====================

#[test]
fn test_update_configured_flag() {
    // Arrange: 准备测试更新 configured 标志
    let mut config = PrivateRepoConfig::default();
    assert!(!config.configured);

    config.configured = true;
    assert!(config.configured);

    config.configured = false;
    assert!(!config.configured);
}

#[test]
fn test_update_branch_config() {
    // Arrange: 准备测试更新 branch 配置
    let mut config = PrivateRepoConfig::default();

    // 初始为 None
    assert!(config.branch.is_none());

    // 设置配置
    config.branch = Some(BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    });
    assert!(config.branch.is_some());

    // 更新配置
    config.branch = Some(BranchConfig {
        prefix: Some("hotfix".to_string()),
        ignore: vec!["develop".to_string()],
    });

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("hotfix".to_string()));
        assert_eq!(branch.ignore, vec!["develop".to_string()]);
    }
}

#[test]
fn test_update_pr_config() {
    // Arrange: 准备测试更新 PR 配置
    let mut config = PrivateRepoConfig::default();

    // 初始为 None
    assert!(config.pr.is_none());

    // 设置配置
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(true),
    });
    assert!(config.pr.is_some());

    // 更新配置
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(false),
    });

    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, Some(false));
    }
}

#[test]
fn test_clear_branch_config() {
    // Arrange: 准备测试清空 branch 配置
    let mut config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec!["main".to_string()],
        }),
        pr: None,
    };

    assert!(config.branch.is_some());

    config.branch = None;
    assert!(config.branch.is_none());
}

#[test]
fn test_clear_pr_config() {
    // Arrange: 准备测试清空 PR 配置
    let mut config = PrivateRepoConfig {
        configured: true,
        branch: None,
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: Some(true),
        }),
    };

    assert!(config.pr.is_some());

    config.pr = None;
    assert!(config.pr.is_none());
}

// ==================== 文件系统集成测试 ====================

#[test]
#[serial(repo_config_fs)]
fn test_load_from_existing_file() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // 生成 repo_id
    let repo_id = PrivateRepoConfig::generate_repo_id()?;

    let config_content = format!(
        r#"
["{repo_id}"]
configured = true

["{repo_id}.branch"]
prefix = "feature"
ignore = ["main", "develop"]

["{repo_id}.pr"]
auto_accept_change_type = true
"#
    );
    env.create_config(&config_content)?;

    // Act: 调用 PrivateRepoConfig::load()
    let config = PrivateRepoConfig::load()?;

    // Assert: 配置正确加载
    assert!(config.configured);
    assert!(config.branch.is_some());
    assert!(config.pr.is_some());

    if let Some(ref branch) = config.branch {
        assert_eq!(branch.prefix, Some("feature".to_string()));
        assert_eq!(branch.ignore.len(), 2);
        assert!(branch.ignore.contains(&"main".to_string()));
        assert!(branch.ignore.contains(&"develop".to_string()));
    }

    if let Some(ref pr) = config.pr {
        assert_eq!(pr.auto_accept_change_type, Some(true));
    }

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_load_from_non_existing_file() -> Result<()> {
    // Arrange: 创建没有配置文件的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 调用 PrivateRepoConfig::load()
    let config = PrivateRepoConfig::load()?;

    // Assert: 返回默认配置
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_save_to_new_file() -> Result<()> {
    // Arrange: 创建临时 Git 仓库（不创建配置文件）
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 创建配置并保存
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec!["main".to_string()],
        }),
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: Some(true),
        }),
    };
    config.save()?;

    // Assert: 文件创建成功
    let config_path = env.config_path();
    assert!(config_path.exists());

    // Assert: 内容正确
    let content = fs::read_to_string(&config_path)?;
    let _repo_id = PrivateRepoConfig::generate_repo_id()?;
    // Note: TOML section names with special chars might be quoted
    assert!(content.contains("configured = true"));
    assert!(content.contains(r#"prefix = "feature""#));
    assert!(content.contains("auto_accept_change_type = true"));

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_save_preserves_other_repos() -> Result<()> {
    // Arrange: 创建包含其他仓库配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let config_content = r#"
[other_repo_12345678]
configured = true

[other_repo_12345678.branch]
prefix = "hotfix"
"#;
    env.create_config(config_content)?;

    // Act: 保存当前仓库的配置
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec![],
        }),
        pr: None,
    };
    config.save()?;

    // Assert: 其他仓库配置未被覆盖
    let content = fs::read_to_string(env.config_path())?;
    assert!(content.contains("[other_repo_12345678]"));
    assert!(content.contains("[other_repo_12345678.branch]"));
    assert!(content.contains(r#"prefix = "hotfix""#));

    // Assert: 当前仓库配置已添加
    assert!(content.contains(r#"prefix = "feature""#));

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_load_and_save_roundtrip() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let repo_id = PrivateRepoConfig::generate_repo_id()?;
    let config_content = format!(
        r#"
["{repo_id}"]
configured = true

["{repo_id}.branch"]
prefix = "feature"
ignore = ["main"]
"#
    );
    env.create_config(&config_content)?;

    // Act: 加载 → 修改 → 保存 → 重新加载
    let mut config = PrivateRepoConfig::load()?;
    assert!(config.configured);

    // 修改配置
    config.pr = Some(PullRequestsConfig {
        auto_accept_change_type: Some(false),
    });
    if let Some(ref mut branch) = config.branch {
        branch.ignore.push("develop".to_string());
    }
    config.save()?;

    // 重新加载
    let reloaded_config = PrivateRepoConfig::load()?;

    // Assert: 数据一致性
    assert_eq!(reloaded_config.configured, config.configured);
    assert_eq!(
        reloaded_config.branch.as_ref().and_then(|b| b.prefix.clone()),
        config.branch.as_ref().and_then(|b| b.prefix.clone())
    );
    assert_eq!(
        reloaded_config.branch.as_ref().map(|b| b.ignore.len()),
        Some(2)
    );
    assert_eq!(
        reloaded_config.pr.as_ref().and_then(|p| p.auto_accept_change_type),
        Some(false)
    );

    Ok(())
}

// ==================== 错误场景测试 ====================

#[test]
#[serial(repo_config_fs)]
fn test_load_corrupted_toml_file() -> Result<()> {
    // Arrange: 创建包含无效 TOML 的配置文件
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let invalid_toml = r#"
[test_repo
configured = "invalid  # 缺少闭合引号和括号
"#;
    env.create_config(invalid_toml)?;

    // Act: 尝试加载配置
    let result = PrivateRepoConfig::load();

    // Assert: 返回错误
    assert!(result.is_err());

    Ok(())
}

/// 测试向只读目录保存配置的错误处理
///
/// ## 测试目的
/// 验证`PrivateRepoConfig::save()`在目标目录只读时能够正确返回错误，而不是panic。
///
/// ## 为什么被忽略
/// - **权限模型差异**: 不同系统（Linux/macOS）的权限模型可能不同
/// - **Unix特定**: 使用Unix特定的权限API，仅在Unix系统上运行
/// - **文件系统依赖**: 依赖文件系统正确处理只读权限
/// - **可能失败**: 在某些系统配置下可能因权限处理不同而失败
/// - **需要#[serial]**: 避免并发文件系统操作干扰
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_save_to_readonly_directory -- --ignored
/// ```
/// 注意：此测试仅在Unix系统（Linux/macOS）上运行
///
/// ## 测试场景
/// 1. 创建临时Git仓库和.workflow目录
/// 2. 将.workflow目录设置为只读（权限0o555）
/// 3. 尝试保存PrivateRepoConfig到只读目录
/// 4. 验证操作失败并返回适当的错误
/// 5. 恢复目录权限进行清理
///
/// ## 预期行为
/// - 保存操作失败（不能创建config子目录）
/// - 返回Err而不是panic
/// - 错误消息清晰说明权限问题
/// - 清理过程正确恢复权限
/// - 不会留下部分创建的文件
#[test]
#[serial(repo_config_fs)]
#[cfg(unix)]
#[ignore] // 这个测试在某些系统上可能因权限模型不同而失败
fn test_save_to_readonly_directory() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Arrange: 创建只读的 .workflow 目录（阻止创建 config 子目录）
    let env = TestEnv::new()?;
    env.init_git_repo()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    let workflow_dir = env.path().join(".workflow");
    fs::create_dir_all(&workflow_dir)?;

    // 设置 .workflow 目录为只读（无法创建 config 子目录）
    let mut perms = fs::metadata(&workflow_dir)?.permissions();
    perms.set_mode(0o555); // r-xr-xr-x (只读+执行)
    fs::set_permissions(&workflow_dir, perms)?;

    // Act: 尝试保存配置
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec![],
        }),
        pr: None,
    };
    let result = config.save();

    // Assert: 返回权限错误
    // 注意：在某些系统上，root 用户或特定权限配置下这个测试可能会失败
    assert!(
        result.is_err(),
        "Expected error when saving to readonly directory"
    );

    // 恢复权限以便清理
    let mut perms = fs::metadata(&workflow_dir)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&workflow_dir, perms)?;

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_generate_repo_id_outside_git_repo() -> Result<()> {
    use crate::common::helpers::CurrentDirGuard;

    // Arrange: 创建非 Git 仓库的临时目录
    let env = TestEnv::new()?;
    let temp_path = env.path();
    let _dir_guard = CurrentDirGuard::new(temp_path)?;

    // Act: 尝试生成 repo_id
    let result = PrivateRepoConfig::generate_repo_id();

    // Assert: 返回错误（因为不在 Git 仓库中）
    assert!(result.is_err());

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_save_with_empty_branch_config() -> Result<()> {
    // Arrange: 准备测试保存空的 branch 配置（prefix 为 None 且 ignore 为空）
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: None,
            ignore: vec![],
        }),
        pr: None,
    };
    config.save()?;

    // Assert: 文件创建成功但不包含 branch 部分（因为是空的）
    let content = fs::read_to_string(env.config_path())?;
    assert!(content.contains("configured = true"));
    // 空的 branch 配置不应该被保存
    assert!(!content.contains(".branch"));

    Ok(())
}

#[test]
#[serial(repo_config_fs)]
fn test_save_with_empty_pr_config() -> Result<()> {
    // Arrange: 准备测试保存空的 PR 配置
    let env = TestEnv::new()?;
    env.init_git_repo()?;

    let config = PrivateRepoConfig {
        configured: true,
        branch: None,
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: None,
        }),
    };
    config.save()?;

    // Assert: 文件创建成功但不包含 pr 部分（因为是空的）
    let content = fs::read_to_string(env.config_path())?;
    assert!(content.contains("configured = true"));
    // 空的 pr 配置不应该被保存
    assert!(!content.contains(".pr"));

    Ok(())
}
