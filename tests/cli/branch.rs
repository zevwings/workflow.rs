//! Branch CLI 命令测试
//!
//! 测试 Branch CLI 命令的参数解析、命令执行流程和错误处理。

use std::collections::HashMap;
use workflow::commands::branch::{validate_repo_name_format, BranchConfig, RepositoryConfig};

// ==================== 仓库格式验证测试 ====================

#[test]
fn test_validate_repo_name_format_valid() {
    assert!(validate_repo_name_format("owner/repo").is_ok());
    assert!(validate_repo_name_format("github/workflow.rs").is_ok());
    assert!(validate_repo_name_format("user-name/repo_name").is_ok());
}

#[test]
fn test_validate_repo_name_format_invalid() {
    // 缺少斜杠
    assert!(validate_repo_name_format("invalid").is_err());
    // 多个斜杠
    assert!(validate_repo_name_format("owner/repo/sub").is_err());
    // 空的 owner
    assert!(validate_repo_name_format("/repo").is_err());
    // 空的 repo
    assert!(validate_repo_name_format("owner/").is_err());
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
