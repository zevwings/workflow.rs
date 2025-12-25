//! PrivateRepoConfig 完整测试
//!
//! 包含数据结构测试、文件系统集成测试和错误场景测试

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use workflow::repo::config::private::PrivateRepoConfig;
use workflow::repo::config::types::{BranchConfig, PullRequestsConfig};

use crate::common::environments::CliTestEnv;
use crate::common::helpers::CurrentDirGuard;

// ==================== 测试辅助函数 ====================

/// 创建私有配置文件（~/.workflow/config/repository.toml）
///
/// 注意：需要先设置 HOME 环境变量指向 CliTestEnv 的路径
fn create_private_config(env: &CliTestEnv, content: &str) -> Result<PathBuf> {
    let config_dir = env.path().join(".workflow").join("config");
    fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("repository.toml");
    fs::write(&config_file, content)?;
    Ok(config_file)
}

// ==================== 默认值测试 ====================

/// 测试私有配置默认值
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 的默认值正确。
///
/// ## 测试场景
/// 1. 创建默认的 PrivateRepoConfig
/// 2. 验证默认值（configured 为 false，branch 和 pr 为 None）
///
/// ## 预期结果
/// - configured 为 false，branch 和 pr 为 None
#[test]
fn test_private_config_default() {
    // Arrange: 准备测试私有配置的默认值
    let config = PrivateRepoConfig::default();

    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
}

// ==================== 配置字段测试 ====================

/// 测试设置 configured 字段
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够设置 configured 字段。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 configured 为 true
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - configured 字段被正确设置
#[test]
fn test_private_config_with_configured() {
    // Arrange: 准备测试设置 configured 字段
    let mut config = PrivateRepoConfig::default();
    config.configured = true;

    assert!(config.configured);
}

/// 测试设置 branch 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够设置 branch 配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 branch 配置（包含 prefix 和 ignore）
/// 3. 验证配置值正确
///
/// ## 预期结果
/// - branch 配置被正确设置
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

/// 测试设置 PR 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够设置 PR 配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 PR 配置
/// 3. 验证配置值正确
///
/// ## 预期结果
/// - PR 配置被正确设置
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

/// 测试设置所有字段
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够同时设置所有字段。
///
/// ## 测试场景
/// 1. 创建包含所有字段的配置
/// 2. 验证所有字段都被正确设置
///
/// ## 预期结果
/// - 所有字段都被正确设置
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

/// 测试仓库 ID 格式生成
///
/// ## 测试目的
/// 验证 PrivateRepoConfig::generate_repo_id() 生成的仓库 ID 格式正确。
///
/// ## 测试场景
/// 1. 在 Git 仓库中生成 repo_id
/// 2. 验证格式：{repo_name}_{hash}
/// 3. 验证 hash 部分是 8 个字符的十六进制
///
/// ## 预期结果
/// - 仓库 ID 格式正确
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

/// 测试仓库 ID 生成一致性
///
/// ## 测试目的
/// 验证同一仓库多次生成的 ID 应该一致。
///
/// ## 测试场景
/// 1. 在同一仓库中生成两次 repo_id
/// 2. 验证两次生成的 ID 相同
///
/// ## 预期结果
/// - 两次生成的 ID 相同
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

/// 测试配置克隆功能
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 的 Clone trait 实现正确。
///
/// ## 测试场景
/// 1. 创建配置实例
/// 2. 克隆配置
/// 3. 验证克隆后的配置与原配置一致
///
/// ## 预期结果
/// - 克隆后的配置与原配置字段值一致
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

/// 测试配置 Debug 输出
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 的 Debug trait 实现正确。
///
/// ## 测试场景
/// 1. 创建配置实例
/// 2. 格式化 Debug 输出
/// 3. 验证输出包含配置类型名和字段名
///
/// ## 预期结果
/// - Debug 输出包含 "PrivateRepoConfig" 和 "configured"
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

/// 测试空 branch 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够处理空的 branch 配置（prefix 为 None，ignore 为空）。
///
/// ## 测试场景
/// 1. 创建包含空 branch 配置的配置
/// 2. 验证配置值正确
///
/// ## 预期结果
/// - branch 配置存在但字段为空
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

/// 测试空 PR 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够处理空的 PR 配置（auto_accept_change_type 为 None）。
///
/// ## 测试场景
/// 1. 创建包含空 PR 配置的配置
/// 2. 验证配置值正确
///
/// ## 预期结果
/// - PR 配置存在但字段为空
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

/// 测试多个忽略分支
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够处理包含多个忽略分支的配置。
///
/// ## 测试场景
/// 1. 创建包含多个忽略分支的配置
/// 2. 验证所有分支都被正确保存
///
/// ## 预期结果
/// - 所有忽略分支都被正确保存
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

/// 测试特殊字符分支前缀
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够处理包含特殊字符的分支前缀。
///
/// ## 测试场景
/// 1. 创建包含特殊字符分支前缀的配置
/// 2. 验证特殊字符被正确保存
///
/// ## 预期结果
/// - 特殊字符被正确保存
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

/// 测试私有配置参数化
///
/// ## 测试目的
/// 使用参数化测试验证 PrivateRepoConfig 的各种配置组合。
///
/// ## 测试场景
/// 1. 使用不同的 configured、prefix 和 ignore 组合创建配置
/// 2. 验证配置值正确
///
/// ## 预期结果
/// - 所有配置组合都被正确处理
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

/// 测试 PR 配置参数化
///
/// ## 测试目的
/// 使用参数化测试验证 PrivateRepoConfig 的 PR 配置各种值。
///
/// ## 测试场景
/// 1. 使用不同的 auto_accept_change_type 值创建配置
/// 2. 验证配置值正确
///
/// ## 预期结果
/// - 所有 PR 配置值都被正确处理
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

/// 测试更新 configured 标志
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够更新 configured 标志。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 更新 configured 为 true
/// 3. 更新 configured 为 false
/// 4. 验证更新成功
///
/// ## 预期结果
/// - configured 标志能够正确更新
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

/// 测试更新 branch 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够更新 branch 配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 branch 配置
/// 3. 更新 branch 配置
/// 4. 验证更新成功
///
/// ## 预期结果
/// - branch 配置能够正确更新
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

/// 测试更新 PR 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够更新 PR 配置。
///
/// ## 测试场景
/// 1. 创建默认配置
/// 2. 设置 PR 配置
/// 3. 更新 PR 配置
/// 4. 验证更新成功
///
/// ## 预期结果
/// - PR 配置能够正确更新
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

/// 测试清空 branch 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够清空 branch 配置。
///
/// ## 测试场景
/// 1. 创建包含 branch 配置的配置
/// 2. 将 branch 设置为 None
/// 3. 验证配置已清空
///
/// ## 预期结果
/// - branch 配置被清空（为 None）
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

/// 测试清空 PR 配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够清空 PR 配置。
///
/// ## 测试场景
/// 1. 创建包含 PR 配置的配置
/// 2. 将 PR 设置为 None
/// 3. 验证配置已清空
///
/// ## 预期结果
/// - PR 配置被清空（为 None）
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

/// 测试从文件加载配置
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够从文件系统加载有效的配置文件。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库和配置文件
/// 2. 设置环境变量（HOME、XDG_CONFIG_HOME）
/// 3. 调用 load() 加载配置
/// 4. 验证配置正确加载
///
/// ## 预期结果
/// - 配置能够正确从文件加载
#[test]
#[serial(repo_config_fs)]
fn test_load_from_existing_file() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量（PrivateRepoConfig 需要）
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

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
    create_private_config(&env, &config_content)?;

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

/// 测试从不存在文件加载配置
///
/// ## 测试目的
/// 验证当配置文件不存在时，PrivateRepoConfig 返回默认配置。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库（不创建配置文件）
/// 2. 设置环境变量
/// 3. 调用 load() 加载配置
/// 4. 验证返回默认配置
///
/// ## 预期结果
/// - 返回默认配置（configured 为 false，branch 和 pr 为 None）
#[test]
#[serial(repo_config_fs)]
fn test_load_from_non_existing_file() -> Result<()> {
    // Arrange: 创建没有配置文件的临时 Git 仓库
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    // Act: 调用 PrivateRepoConfig::load()
    let config = PrivateRepoConfig::load()?;

    // Assert: 返回默认配置
    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());

    Ok(())
}

/// 测试保存配置到新文件
///
/// ## 测试目的
/// 验证 PrivateRepoConfig 能够将配置保存到新文件。
///
/// ## 测试场景
/// 1. 创建临时 Git 仓库（不创建配置文件）
/// 2. 设置环境变量
/// 3. 创建配置并保存
/// 4. 验证文件创建成功且内容正确
///
/// ## 预期结果
/// - 配置文件被创建且内容正确
#[test]
#[serial(repo_config_fs)]
fn test_save_to_new_file() -> Result<()> {
    // Arrange: 创建临时 Git 仓库（不创建配置文件）
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

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

    // 切换到测试目录，让 generate_repo_id() 能找到 Git 仓库
    let _guard = CurrentDirGuard::new(env.path())?;
    config.save()?;

    // Assert: 文件创建成功
    let config_path = env.path().join(".workflow").join("config").join("repository.toml");
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

/// 测试保存配置时保留其他仓库配置
///
/// ## 测试目的
/// 验证保存配置时不会覆盖配置文件中的其他仓库配置。
///
/// ## 测试场景
/// 1. 创建包含其他仓库配置的文件
/// 2. 保存当前仓库的配置
/// 3. 验证其他仓库配置未被覆盖
///
/// ## 预期结果
/// - 其他仓库配置被保留，当前仓库配置已添加
#[test]
#[serial(repo_config_fs)]
fn test_save_preserves_other_repos() -> Result<()> {
    // Arrange: 创建包含其他仓库配置的临时 Git 仓库
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let config_content = r#"
[other_repo_12345678]
configured = true

[other_repo_12345678.branch]
prefix = "hotfix"
"#;
    create_private_config(&env, config_content)?;

    // Act: 保存当前仓库的配置
    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: Some("feature".to_string()),
            ignore: vec![],
        }),
        pr: None,
    };

    // 切换到测试目录，让 generate_repo_id() 能找到 Git 仓库
    let _guard = CurrentDirGuard::new(env.path())?;
    config.save()?;

    // Assert: 其他仓库配置未被覆盖
    let config_path = env.path().join(".workflow").join("config").join("repository.toml");
    let content = fs::read_to_string(config_path)?;
    assert!(content.contains("[other_repo_12345678]"));
    assert!(content.contains("[other_repo_12345678.branch]"));
    assert!(content.contains(r#"prefix = "hotfix""#));

    // Assert: 当前仓库配置已添加
    assert!(content.contains(r#"prefix = "feature""#));

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
fn test_load_and_save_roundtrip() -> Result<()> {
    // Arrange: 创建包含配置的临时 Git 仓库
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

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
    create_private_config(&env, &config_content)?;

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

/// 测试加载损坏的 TOML 文件
///
/// ## 测试目的
/// 验证当配置文件包含无效 TOML 时，PrivateRepoConfig 返回错误。
///
/// ## 测试场景
/// 1. 创建包含无效 TOML 的配置文件
/// 2. 设置环境变量
/// 3. 尝试加载配置
/// 4. 验证返回错误
///
/// ## 预期结果
/// - 返回 TOML 解析错误
#[test]
#[serial(repo_config_fs)]
fn test_load_corrupted_toml_file() -> Result<()> {
    // Arrange: 创建包含无效 TOML 的配置文件
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let invalid_toml = r#"
[test_repo
configured = "invalid  # 缺少闭合引号和括号
"#;
    create_private_config(&env, invalid_toml)?;

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
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

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

/// 测试在非 Git 仓库中生成仓库 ID
///
/// ## 测试目的
/// 验证当不在 Git 仓库中时，generate_repo_id() 返回错误。
///
/// ## 测试场景
/// 1. 创建非 Git 仓库的临时目录
/// 2. 尝试生成 repo_id
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回错误（因为不在 Git 仓库中）
#[test]
#[serial(repo_config_fs)]
fn test_generate_repo_id_outside_git_repo() -> Result<()> {
    use crate::common::helpers::CurrentDirGuard;

    // Arrange: 创建非 Git 仓库的临时目录
    let env = CliTestEnv::new()?;
    // 注意：不调用 init_git_repo()，因为我们需要测试非 Git 仓库的情况

    // Act: 切换到非 Git 仓库目录，然后尝试生成 repo_id
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = PrivateRepoConfig::generate_repo_id();

    // Assert: 返回错误（因为不在 Git 仓库中）
    assert!(result.is_err());

    Ok(())
}

/// 测试保存空 branch 配置
///
/// ## 测试目的
/// 验证当 branch 配置为空（prefix 为 None 且 ignore 为空）时，保存时不会包含 branch 部分。
///
/// ## 测试场景
/// 1. 创建包含空 branch 配置的配置
/// 2. 保存配置
/// 3. 验证文件不包含 branch 部分
///
/// ## 预期结果
/// - 文件创建成功但不包含 branch 部分
#[test]
#[serial(repo_config_fs)]
fn test_save_with_empty_branch_config() -> Result<()> {
    // Arrange: 准备测试保存空的 branch 配置（prefix 为 None 且 ignore 为空）
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let config = PrivateRepoConfig {
        configured: true,
        branch: Some(BranchConfig {
            prefix: None,
            ignore: vec![],
        }),
        pr: None,
    };

    // 切换到测试目录，让 generate_repo_id() 能找到 Git 仓库
    let _guard = CurrentDirGuard::new(env.path())?;
    config.save()?;

    // Assert: 文件创建成功但不包含 branch 部分（因为是空的）
    let config_path = env.path().join(".workflow").join("config").join("repository.toml");
    let content = fs::read_to_string(config_path)?;
    assert!(content.contains("configured = true"));
    // 空的 branch 配置不应该被保存
    assert!(!content.contains(".branch"));

    Ok(())
}

/// 测试保存空 PR 配置
///
/// ## 测试目的
/// 验证当 PR 配置为空（auto_accept_change_type 为 None）时，保存时不会包含 PR 部分。
///
/// ## 测试场景
/// 1. 创建包含空 PR 配置的配置
/// 2. 保存配置
/// 3. 验证文件不包含 PR 部分
///
/// ## 预期结果
/// - 文件创建成功但不包含 PR 部分
#[test]
#[serial(repo_config_fs)]
fn test_save_with_empty_pr_config() -> Result<()> {
    // Arrange: 准备测试保存空的 PR 配置
    let mut env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 设置 HOME 和 XDG_CONFIG_HOME 环境变量
    let home_path = env.path().to_string_lossy().to_string();
    let xdg_path = env.path().join(".config").to_string_lossy().to_string();
    env.env_guard().set("HOME", &home_path);
    env.env_guard().set("XDG_CONFIG_HOME", &xdg_path);

    let config = PrivateRepoConfig {
        configured: true,
        branch: None,
        pr: Some(PullRequestsConfig {
            auto_accept_change_type: None,
        }),
    };

    // 切换到测试目录，让 generate_repo_id() 能找到 Git 仓库
    let _guard = CurrentDirGuard::new(env.path())?;
    config.save()?;

    // Assert: 文件创建成功但不包含 pr 部分（因为是空的）
    let config_path = env.path().join(".workflow").join("config").join("repository.toml");
    let content = fs::read_to_string(config_path)?;
    assert!(content.contains("configured = true"));
    // 空的 pr 配置不应该被保存
    assert!(!content.contains(".pr"));

    Ok(())
}
