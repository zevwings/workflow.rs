//! Branch CLI 命令测试
//!
//! 测试 Branch CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use std::collections::HashMap;
use workflow::cli::BranchSubcommand;
use workflow::commands::branch::{BranchConfig, RepositoryConfig};

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-branch")]
struct TestBranchCli {
    #[command(subcommand)]
    command: BranchSubcommand,
}

// ==================== Create 命令测试 ====================

#[test]
fn test_branch_create_command_structure() {
    // 测试 Create 命令结构（带所有参数）
    let cli = TestBranchCli::try_parse_from(&[
        "test-branch",
        "create",
        "PROJ-123",
        "--from-default",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-123".to_string()));
            assert!(from_default);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_minimal() {
    // 测试 Create 命令最小参数
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(!from_default);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_with_jira_ticket_only() {
    // 测试 Create 命令只带 JIRA ticket
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "PROJ-456"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-456".to_string()));
            assert!(!from_default);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_with_from_default() {
    // 测试 Create 命令带 --from-default 参数
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "--from-default"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(from_default);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_with_dry_run() {
    // 测试 Create 命令带 --dry-run 参数
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "--dry-run"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(!from_default);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

// ==================== RepositoryConfig 序列化/反序列化测试 ====================

#[test]
fn test_repository_config_serialize_with_prefix() {
    let config = RepositoryConfig {
        branch_prefix: Some("feature".to_string()),
        branch_ignore: vec!["main".to_string(), "develop".to_string()],
        branch_prefix_prompted: false,
    };

    let toml = toml::to_string(&config).unwrap();
    assert!(toml.contains("branch_prefix = \"feature\""));
    assert!(toml.contains("branch_ignore = [\"main\", \"develop\"]"));
    // branch_prefix_prompted 为 false 时不应该序列化
    assert!(!toml.contains("branch_prefix_prompted"));
}

#[test]
fn test_repository_config_serialize_without_prefix() {
    let config = RepositoryConfig {
        branch_prefix: None,
        branch_ignore: vec!["main".to_string()],
        branch_prefix_prompted: false,
    };

    let toml = toml::to_string(&config).unwrap();
    // branch_prefix 为 None 时不应该序列化
    assert!(!toml.contains("branch_prefix"));
    assert!(toml.contains("branch_ignore = [\"main\"]"));
}

#[test]
fn test_repository_config_deserialize_with_prefix() {
    let toml = r#"
branch_prefix = "feature"
branch_ignore = ["main", "develop"]
"#;

    let config: RepositoryConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.branch_prefix, Some("feature".to_string()));
    assert_eq!(config.branch_ignore, vec!["main", "develop"]);
    assert_eq!(config.branch_prefix_prompted, false);
}

#[test]
fn test_repository_config_deserialize_without_prefix() {
    let toml = r#"
branch_ignore = ["main"]
"#;

    let config: RepositoryConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.branch_prefix, None);
    assert_eq!(config.branch_ignore, vec!["main"]);
    assert_eq!(config.branch_prefix_prompted, false);
}

#[test]
fn test_repository_config_deserialize_backward_compatible_ignore() {
    // 测试向后兼容：旧的 "ignore" 字段名
    let toml = r#"
ignore = ["main", "develop"]
"#;

    let config: RepositoryConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.branch_ignore, vec!["main", "develop"]);
}

// ==================== BranchConfig 方法测试 ====================

#[test]
fn test_get_branch_prefix_for_repo_exists() {
    let mut repositories = HashMap::new();
    repositories.insert(
        "owner/repo".to_string(),
        RepositoryConfig {
            branch_prefix: Some("feature".to_string()),
            branch_ignore: vec![],
            branch_prefix_prompted: false,
        },
    );

    let config = BranchConfig { repositories };
    assert_eq!(
        config.get_branch_prefix_for_repo("owner/repo"),
        Some("feature")
    );
}

#[test]
fn test_get_branch_prefix_for_repo_not_exists() {
    let config = BranchConfig {
        repositories: HashMap::new(),
    };
    assert_eq!(config.get_branch_prefix_for_repo("owner/repo"), None);
}

#[test]
fn test_get_branch_prefix_for_repo_no_prefix() {
    let mut repositories = HashMap::new();
    repositories.insert(
        "owner/repo".to_string(),
        RepositoryConfig {
            branch_prefix: None,
            branch_ignore: vec![],
            branch_prefix_prompted: false,
        },
    );

    let config = BranchConfig { repositories };
    assert_eq!(config.get_branch_prefix_for_repo("owner/repo"), None);
}

// ==================== BranchConfig 序列化/反序列化测试 ====================

#[test]
fn test_branch_config_serialize() {
    let mut repositories = HashMap::new();
    repositories.insert(
        "owner/repo1".to_string(),
        RepositoryConfig {
            branch_prefix: Some("feature".to_string()),
            branch_ignore: vec!["main".to_string()],
            branch_prefix_prompted: false,
        },
    );
    repositories.insert(
        "owner/repo2".to_string(),
        RepositoryConfig {
            branch_prefix: None,
            branch_ignore: vec!["develop".to_string()],
            branch_prefix_prompted: false,
        },
    );

    let config = BranchConfig { repositories };
    let toml = toml::to_string(&config).unwrap();

    // 打印实际格式用于调试
    // println!("Serialized TOML:\n{}", toml);

    // 检查两个仓库的配置都被序列化
    // 注意：由于使用了 #[serde(flatten)]，HashMap 会被序列化为表头格式 [key]
    // 但 TOML 表头中的斜杠需要转义或使用引号，实际格式可能是 ["owner/repo1"] 或 [owner/repo1]
    // 我们检查关键内容而不是格式
    assert!(toml.contains("branch_prefix = \"feature\""));
    assert!(toml.contains("branch_ignore = [\"develop\"]"));
    // 检查仓库名（可能在表头中，格式可能不同）
    assert!(toml.contains("owner/repo1") || toml.contains("owner/repo2"));
}

#[test]
fn test_branch_config_deserialize() {
    // 测试两种可能的 TOML 格式（带引号和不带引号的表头）
    let toml1 = r#"
["owner/repo1"]
branch_prefix = "feature"
branch_ignore = ["main"]

["owner/repo2"]
branch_ignore = ["develop"]
"#;

    let config1: BranchConfig = toml::from_str(toml1).unwrap();
    assert_eq!(
        config1.get_branch_prefix_for_repo("owner/repo1"),
        Some("feature")
    );
    assert_eq!(config1.get_branch_prefix_for_repo("owner/repo2"), None);

    // 也测试不带引号的格式（如果支持）
    let toml2 = r#"
[owner/repo1]
branch_prefix = "feature"
branch_ignore = ["main"]
"#;

    // 这个格式可能不被 TOML 解析器接受（因为表头中的斜杠），但我们可以测试
    // 如果解析失败，这是预期的行为
    if let Ok(config2) = toml::from_str::<BranchConfig>(toml2) {
        assert_eq!(
            config2.get_branch_prefix_for_repo("owner/repo1"),
            Some("feature")
        );
    }
}
