//! 仓库配置管理测试
//!
//! 测试仓库配置的创建、验证、序列化和管理功能。

use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use serde_json;
use std::fs;
use tempfile::TempDir;
use workflow::repo::{BranchConfig, PullRequestsConfig, RepoConfig};

// ==================== Fixtures ====================

/// 创建临时目录用于测试
#[fixture]
fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// 创建测试用的分支配置
#[fixture]
fn sample_branch_config() -> BranchConfig {
    BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string(), "develop".to_string()],
    }
}

/// 创建测试用的 PR 配置
#[fixture]
fn sample_pr_config() -> PullRequestsConfig {
    PullRequestsConfig {
        auto_accept_change_type: Some(true),
    }
}

// ==================== BranchConfig 测试 ====================

#[test]
fn test_branch_config_default() {
    /// 测试分支配置的默认值
    let config = BranchConfig::default();

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
}

#[test]
fn test_branch_config_with_values() {
    /// 测试带值的分支配置创建
    let config = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string(), "develop".to_string()],
    };

    assert_eq!(config.prefix, Some("feature".to_string()));
    assert_eq!(config.ignore, vec!["main", "develop"]);
}

#[test]
fn test_branch_config_serialization() {
    /// 测试分支配置的序列化
    let config = BranchConfig {
        prefix: Some("hotfix".to_string()),
        ignore: vec!["master".to_string()],
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let expected = r#"{"prefix":"hotfix","ignore":["master"]}"#;

    assert_eq!(json, expected);
}

#[test]
fn test_branch_config_deserialization() {
    /// 测试分支配置的反序列化
    let json = r#"{"prefix":"feature","ignore":["main","develop"]}"#;
    let config: BranchConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.prefix, Some("feature".to_string()));
    assert_eq!(config.ignore, vec!["main", "develop"]);
}

#[test]
fn test_branch_config_partial_deserialization() {
    /// 测试部分字段的反序列化
    let json = r#"{"prefix":"feature"}"#;
    let config: BranchConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.prefix, Some("feature".to_string()));
    assert!(config.ignore.is_empty());
}

#[test]
fn test_branch_config_empty_deserialization() {
    /// 测试空配置的反序列化
    let json = r#"{}"#;
    let config: BranchConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
}

#[rstest]
#[case(None, vec![])]
#[case(Some("feature".to_string()), vec![])]
#[case(Some("hotfix".to_string()), vec!["main".to_string()])]
#[case(None, vec!["main".to_string(), "develop".to_string()])]
fn test_branch_config_parametrized(
    #[case] prefix: Option<String>,
    #[case] ignore: Vec<String>
) {
    /// 参数化测试分支配置的各种组合
    let config = BranchConfig { prefix, ignore: ignore.clone() };

    // 测试序列化和反序列化的一致性
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: BranchConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.prefix, config.prefix);
    assert_eq!(deserialized.ignore, ignore);
}

// ==================== PullRequestsConfig 测试 ====================

#[test]
fn test_pr_config_default() {
    /// 测试 PR 配置的默认值
    let config = PullRequestsConfig::default();

    assert_eq!(config.auto_accept_change_type, None);
}

#[test]
fn test_pr_config_with_values() {
    /// 测试带值的 PR 配置创建
    let config = PullRequestsConfig {
        auto_accept_change_type: Some(true),
    };

    assert_eq!(config.auto_accept_change_type, Some(true));
}

#[test]
fn test_pr_config_serialization() {
    /// 测试 PR 配置的序列化
    let config = PullRequestsConfig {
        auto_accept_change_type: Some(false),
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let expected = r#"{"auto_accept_change_type":false}"#;

    assert_eq!(json, expected);
}

#[test]
fn test_pr_config_deserialization() {
    /// 测试 PR 配置的反序列化
    let json = r#"{"auto_accept_change_type":true}"#;
    let config: PullRequestsConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.auto_accept_change_type, Some(true));
}

#[test]
fn test_pr_config_empty_deserialization() {
    /// 测试空 PR 配置的反序列化
    let json = r#"{}"#;
    let config: PullRequestsConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.auto_accept_change_type, None);
}

#[rstest]
#[case(None)]
#[case(Some(true))]
#[case(Some(false))]
fn test_pr_config_parametrized(#[case] auto_accept: Option<bool>) {
    /// 参数化测试 PR 配置的各种值
    let config = PullRequestsConfig {
        auto_accept_change_type: auto_accept,
    };

    // 测试序列化和反序列化的一致性
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: PullRequestsConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.auto_accept_change_type, auto_accept);
}

// ==================== 边界条件和错误处理测试 ====================

#[test]
fn test_branch_config_invalid_json() {
    /// 测试无效 JSON 的处理
    let invalid_json = r#"{"prefix": invalid}"#;
    let result = serde_json::from_str::<BranchConfig>(invalid_json);

    assert!(result.is_err());
}

#[test]
fn test_pr_config_invalid_json() {
    /// 测试无效 JSON 的处理
    let invalid_json = r#"{"auto_accept_change_type": "not_boolean"}"#;
    let result = serde_json::from_str::<PullRequestsConfig>(invalid_json);

    assert!(result.is_err());
}

#[test]
fn test_branch_config_with_null_values() {
    /// 测试包含 null 值的 JSON
    let json = r#"{"prefix":null,"ignore":[]}"#;
    let config: BranchConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
}

#[test]
fn test_branch_config_empty_ignore_list() {
    /// 测试空的忽略列表
    let config = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");

    // 空数组应该被序列化
    assert!(json.contains(r#""ignore":[]"#));
}

#[test]
fn test_branch_config_special_characters() {
    /// 测试特殊字符的处理
    let config = BranchConfig {
        prefix: Some("feature/test-123".to_string()),
        ignore: vec!["release/v1.0".to_string(), "hotfix-urgent".to_string()],
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: BranchConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.prefix, config.prefix);
    assert_eq!(deserialized.ignore, config.ignore);
}

#[test]
fn test_config_clone() {
    /// 测试配置的克隆功能
    let original_branch = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    };

    let original_pr = PullRequestsConfig {
        auto_accept_change_type: Some(true),
    };

    let cloned_branch = original_branch.clone();
    let cloned_pr = original_pr.clone();

    assert_eq!(cloned_branch.prefix, original_branch.prefix);
    assert_eq!(cloned_branch.ignore, original_branch.ignore);
    assert_eq!(cloned_pr.auto_accept_change_type, original_pr.auto_accept_change_type);
}

#[test]
fn test_config_debug() {
    /// 测试配置的 Debug 输出
    let branch_config = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string()],
    };

    let pr_config = PullRequestsConfig {
        auto_accept_change_type: Some(true),
    };

    let branch_debug = format!("{:?}", branch_config);
    let pr_debug = format!("{:?}", pr_config);

    assert!(branch_debug.contains("BranchConfig"));
    assert!(branch_debug.contains("feature"));
    assert!(pr_debug.contains("PullRequestsConfig"));
    assert!(pr_debug.contains("true"));
}
