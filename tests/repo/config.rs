//! 仓库配置管理测试
//!
//! 测试仓库配置的创建、验证、序列化和管理功能。
//!
//! ## 测试策略
//!
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理序列化错误
//! - 使用 `rstest` 进行参数化测试
//! - 测试各种配置组合和边界情况

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json;
use workflow::repo::{BranchConfig, PullRequestsConfig};

// ==================== Fixtures ====================
// (Removed unused fixtures)

// ==================== BranchConfig 测试 ====================

#[test]
fn test_branch_config_default() {
    // 测试分支配置的默认值
    let config = BranchConfig::default();

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
}

#[test]
fn test_branch_config_with_values() {
    // 测试带值的分支配置创建
    let config = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec!["main".to_string(), "develop".to_string()],
    };

    assert_eq!(config.prefix, Some("feature".to_string()));
    assert_eq!(config.ignore, vec!["main", "develop"]);
}

#[test]
fn test_branch_config_serialization() -> Result<()> {
    // 测试分支配置的序列化
    let config = BranchConfig {
        prefix: Some("hotfix".to_string()),
        ignore: vec!["master".to_string()],
    };

    let json = serde_json::to_string(&config)?;
    let expected = r#"{"prefix":"hotfix","ignore":["master"]}"#;

    assert_eq!(json, expected);
    Ok(())
}

#[test]
fn test_branch_config_deserialization() -> Result<()> {
    // 测试分支配置的反序列化
    let json = r#"{"prefix":"feature","ignore":["main","develop"]}"#;
    let config: BranchConfig = serde_json::from_str(json)?;

    assert_eq!(config.prefix, Some("feature".to_string()));
    assert_eq!(config.ignore, vec!["main", "develop"]);
    Ok(())
}

#[test]
fn test_branch_config_partial_deserialization() -> Result<()> {
    // 测试部分字段的反序列化
    let json = r#"{"prefix":"feature"}"#;
    let config: BranchConfig = serde_json::from_str(json)?;

    assert_eq!(config.prefix, Some("feature".to_string()));
    assert!(config.ignore.is_empty());
    Ok(())
}

#[test]
fn test_branch_config_empty_deserialization() -> Result<()> {
    // 测试空配置的反序列化
    let json = r#"{}"#;
    let config: BranchConfig = serde_json::from_str(json)?;

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
    Ok(())
}

#[rstest]
#[case(None, vec![])]
#[case(Some("feature".to_string()), vec![])]
#[case(Some("hotfix".to_string()), vec!["main".to_string()])]
#[case(None, vec!["main".to_string(), "develop".to_string()])]
fn test_branch_config_parametrized(#[case] prefix: Option<String>, #[case] ignore: Vec<String>) -> Result<()> {
    // 参数化测试分支配置的各种组合
    let config = BranchConfig {
        prefix,
        ignore: ignore.clone(),
    };

    // 测试序列化和反序列化的一致性
    let json = serde_json::to_string(&config)?;
    let deserialized: BranchConfig = serde_json::from_str(&json)?;

    assert_eq!(deserialized.prefix, config.prefix);
    assert_eq!(deserialized.ignore, ignore);
    Ok(())
}

// ==================== PullRequestsConfig 测试 ====================

#[test]
fn test_pr_config_default() {
    // 测试 PR 配置的默认值
    let config = PullRequestsConfig::default();

    assert_eq!(config.auto_accept_change_type, None);
}

#[test]
fn test_pr_config_with_values() {
    // 测试带值的 PR 配置创建
    let config = PullRequestsConfig {
        auto_accept_change_type: Some(true),
    };

    assert_eq!(config.auto_accept_change_type, Some(true));
}

#[test]
fn test_pr_config_serialization() -> Result<()> {
    // 测试 PR 配置的序列化
    let config = PullRequestsConfig {
        auto_accept_change_type: Some(false),
    };

    let json = serde_json::to_string(&config)?;
    let expected = r#"{"auto_accept_change_type":false}"#;

    assert_eq!(json, expected);
    Ok(())
}

#[test]
fn test_pr_config_deserialization() -> Result<()> {
    // 测试 PR 配置的反序列化
    let json = r#"{"auto_accept_change_type":true}"#;
    let config: PullRequestsConfig = serde_json::from_str(json)?;

    assert_eq!(config.auto_accept_change_type, Some(true));
    Ok(())
}

#[test]
fn test_pr_config_empty_deserialization() -> Result<()> {
    // 测试空 PR 配置的反序列化
    let json = r#"{}"#;
    let config: PullRequestsConfig = serde_json::from_str(json)?;

    assert_eq!(config.auto_accept_change_type, None);
    Ok(())
}

#[rstest]
#[case(None)]
#[case(Some(true))]
#[case(Some(false))]
fn test_pr_config_parametrized(#[case] auto_accept: Option<bool>) -> Result<()> {
    // 参数化测试 PR 配置的各种值
    let config = PullRequestsConfig {
        auto_accept_change_type: auto_accept,
    };

    // 测试序列化和反序列化的一致性
    let json = serde_json::to_string(&config)?;
    let deserialized: PullRequestsConfig = serde_json::from_str(&json)?;

    assert_eq!(deserialized.auto_accept_change_type, auto_accept);
    Ok(())
}

// ==================== 边界条件和错误处理测试 ====================

#[test]
fn test_branch_config_invalid_json() {
    // 测试无效 JSON 的处理
    let invalid_json = r#"{"prefix": invalid}"#;
    let result = serde_json::from_str::<BranchConfig>(invalid_json);

    assert!(result.is_err());
}

#[test]
fn test_pr_config_invalid_json() {
    // 测试无效 JSON 的处理
    let invalid_json = r#"{"auto_accept_change_type": "not_boolean"}"#;
    let result = serde_json::from_str::<PullRequestsConfig>(invalid_json);

    assert!(result.is_err());
}

#[test]
fn test_branch_config_with_null_values() -> Result<()> {
    // 测试包含 null 值的 JSON
    let json = r#"{"prefix":null,"ignore":[]}"#;
    let config: BranchConfig = serde_json::from_str(json)?;

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
    Ok(())
}

#[test]
fn test_branch_config_empty_ignore_list() -> Result<()> {
    // 测试空的忽略列表
    let config = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    };

    let json = serde_json::to_string(&config)?;

    // 由于 skip_serializing_if = "Vec::is_empty"，空数组不会被序列化
    assert!(!json.contains(r#""ignore""#));
    Ok(())
}

#[test]
fn test_branch_config_special_characters() -> Result<()> {
    // 测试特殊字符的处理
    let config = BranchConfig {
        prefix: Some("feature/test-123".to_string()),
        ignore: vec!["release/v1.0".to_string(), "hotfix-urgent".to_string()],
    };

    let json = serde_json::to_string(&config)?;
    let deserialized: BranchConfig = serde_json::from_str(&json)?;

    assert_eq!(deserialized.prefix, config.prefix);
    assert_eq!(deserialized.ignore, config.ignore);
    Ok(())
}

#[test]
fn test_config_clone() {
    // 测试配置的克隆功能
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
    assert_eq!(
        cloned_pr.auto_accept_change_type,
        original_pr.auto_accept_change_type
    );
}

#[test]
fn test_config_debug() {
    // 测试配置的 Debug 输出
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
