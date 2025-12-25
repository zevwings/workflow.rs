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

// ==================== BranchConfig Tests ====================

/// 测试分支配置默认值
///
/// ## 测试目的
/// 验证 BranchConfig 的默认值正确。
///
/// ## 测试场景
/// 1. 创建默认的 BranchConfig
/// 2. 验证默认值（prefix 为 None，ignore 为空）
///
/// ## 预期结果
/// - prefix 为 None，ignore 为空
#[test]
fn test_branch_config_default_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建默认配置

    // Act: 创建默认的 BranchConfig
    let config = BranchConfig::default();

    // Assert: 验证默认值正确
    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
}

/// 测试分支配置创建
///
/// ## 测试目的
/// 验证 BranchConfig 能够使用指定值创建配置。
///
/// ## 测试场景
/// 1. 使用指定值创建 BranchConfig
/// 2. 验证字段值正确
///
/// ## 预期结果
/// - 所有字段值正确设置
#[test]
fn test_branch_config_with_values_creates_config() {
    // Arrange: 准备配置值
    let prefix = Some("feature".to_string());
    let ignore = vec!["main".to_string(), "develop".to_string()];

    // Act: 创建 BranchConfig 实例
    let config = BranchConfig {
        prefix: prefix.clone(),
        ignore: ignore.clone(),
    };

    // Assert: 验证字段值正确
    assert_eq!(config.prefix, prefix);
    assert_eq!(config.ignore, vec!["main", "develop"]);
}

/// 测试分支配置序列化
///
/// ## 测试目的
/// 验证 BranchConfig 能够正确序列化为 JSON。
///
/// ## 测试场景
/// 1. 创建 BranchConfig 实例
/// 2. 序列化为 JSON 字符串
/// 3. 验证 JSON 字符串正确
///
/// ## 预期结果
/// - JSON 字符串与预期一致
#[test]
fn test_branch_config_serialization_with_valid_config_serializes_to_json() -> Result<()> {
    // Arrange: 准备 BranchConfig 实例
    let config = BranchConfig {
        prefix: Some("hotfix".to_string()),
        ignore: vec!["master".to_string()],
    };
    let expected = r#"{"prefix":"hotfix","ignore":["master"]}"#;

    // Act: 序列化为 JSON
    let json = serde_json::to_string(&config)?;

    // Assert: 验证 JSON 字符串正确
    assert_eq!(json, expected);
    Ok(())
}

/// 测试分支配置反序列化
///
/// ## 测试目的
/// 验证 BranchConfig 能够从有效的 JSON 字符串反序列化。
///
/// ## 测试场景
/// 1. 准备有效的 JSON 字符串
/// 2. 反序列化为 BranchConfig
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - 字段值与 JSON 中的值一致
#[test]
fn test_branch_config_deserialization_with_valid_json_deserializes_config() -> Result<()> {
    // Arrange: 准备有效的 JSON 字符串
    let json = r#"{"prefix":"feature","ignore":["main","develop"]}"#;

    // Act: 反序列化为 BranchConfig
    let config: BranchConfig = serde_json::from_str(json)?;

    // Assert: 验证字段值正确
    assert_eq!(config.prefix, Some("feature".to_string()));
    assert_eq!(config.ignore, vec!["main", "develop"]);
    Ok(())
}

/// 测试分支配置部分反序列化
///
/// ## 测试目的
/// 验证 BranchConfig 能够从部分字段的 JSON 反序列化（缺失字段使用默认值）。
///
/// ## 测试场景
/// 1. 准备只包含部分字段的 JSON
/// 2. 反序列化为 BranchConfig
/// 3. 验证缺失字段使用默认值
///
/// ## 预期结果
/// - 存在的字段正确设置，缺失字段使用默认值
#[test]
fn test_branch_config_partial_deserialization_with_partial_json_deserializes_config() -> Result<()> {
    // Arrange: 准备部分字段的 JSON 字符串
    let json = r#"{"prefix":"feature"}"#;

    // Act: 反序列化为 BranchConfig
    let config: BranchConfig = serde_json::from_str(json)?;

    // Assert: 验证字段值正确（ignore 应为空）
    assert_eq!(config.prefix, Some("feature".to_string()));
    assert!(config.ignore.is_empty());
    Ok(())
}

/// 测试分支配置空 JSON 反序列化
///
/// ## 测试目的
/// 验证 BranchConfig 能够从空 JSON 对象反序列化（使用所有默认值）。
///
/// ## 测试场景
/// 1. 准备空 JSON 对象
/// 2. 反序列化为 BranchConfig
/// 3. 验证所有字段使用默认值
///
/// ## 预期结果
/// - 所有字段使用默认值
#[test]
fn test_branch_config_empty_deserialization_with_empty_json_deserializes_config() -> Result<()> {
    // Arrange: 准备空 JSON 对象
    let json = r#"{}"#;

    // Act: 反序列化为 BranchConfig
    let config: BranchConfig = serde_json::from_str(json)?;

    // Assert: 验证默认值正确
    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
    Ok(())
}

/// 测试分支配置参数化
///
/// ## 测试目的
/// 使用参数化测试验证 BranchConfig 的序列化和反序列化一致性。
///
/// ## 测试场景
/// 1. 使用不同的 prefix 和 ignore 组合创建配置
/// 2. 测试序列化和反序列化的一致性
///
/// ## 预期结果
/// - 序列化和反序列化结果一致
#[rstest]
#[case(None, vec![])]
#[case(Some("feature".to_string()), vec![])]
#[case(Some("hotfix".to_string()), vec!["main".to_string()])]
#[case(None, vec!["main".to_string(), "develop".to_string()])]
fn test_branch_config_parametrized_with_various_combinations_creates_config(
    #[case] prefix: Option<String>,
    #[case] ignore: Vec<String>,
) -> Result<()> {
    // Arrange: 准备参数化测试的配置值
    let config = BranchConfig {
        prefix,
        ignore: ignore.clone(),
    };

    // Act: 测试序列化和反序列化的一致性
    let json = serde_json::to_string(&config)?;
    let deserialized: BranchConfig = serde_json::from_str(&json)?;

    // Assert: 验证序列化和反序列化结果一致
    assert_eq!(deserialized.prefix, config.prefix);
    assert_eq!(deserialized.ignore, ignore);
    Ok(())
}

// ==================== PullRequestsConfig Tests ====================

/// 测试 PR 配置默认值
///
/// ## 测试目的
/// 验证 PullRequestsConfig 的默认值正确。
///
/// ## 测试场景
/// 1. 创建默认的 PullRequestsConfig
/// 2. 验证默认值（auto_accept_change_type 为 None）
///
/// ## 预期结果
/// - auto_accept_change_type 为 None
#[test]
fn test_pr_config_default_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建默认配置

    // Act: 创建默认的 PullRequestsConfig
    let config = PullRequestsConfig::default();

    // Assert: 验证默认值正确
    assert_eq!(config.auto_accept_change_type, None);
}

/// 测试 PR 配置创建
///
/// ## 测试目的
/// 验证 PullRequestsConfig 能够使用指定值创建配置。
///
/// ## 测试场景
/// 1. 使用指定值创建 PullRequestsConfig
/// 2. 验证字段值正确
///
/// ## 预期结果
/// - 字段值正确设置
#[test]
fn test_pr_config_with_values_creates_config() {
    // Arrange: 准备配置值
    let auto_accept = Some(true);

    // Act: 创建 PullRequestsConfig 实例
    let config = PullRequestsConfig {
        auto_accept_change_type: auto_accept,
    };

    // Assert: 验证字段值正确
    assert_eq!(config.auto_accept_change_type, auto_accept);
}

/// 测试 PR 配置序列化
///
/// ## 测试目的
/// 验证 PullRequestsConfig 能够正确序列化为 JSON。
///
/// ## 测试场景
/// 1. 创建 PullRequestsConfig 实例
/// 2. 序列化为 JSON 字符串
/// 3. 验证 JSON 字符串正确
///
/// ## 预期结果
/// - JSON 字符串与预期一致
#[test]
fn test_pr_config_serialization_with_valid_config_serializes_to_json() -> Result<()> {
    // Arrange: 准备 PullRequestsConfig 实例
    let config = PullRequestsConfig {
        auto_accept_change_type: Some(false),
    };
    let expected = r#"{"auto_accept_change_type":false}"#;

    // Act: 序列化为 JSON
    let json = serde_json::to_string(&config)?;

    // Assert: 验证 JSON 字符串正确
    assert_eq!(json, expected);
    Ok(())
}

/// 测试 PR 配置反序列化
///
/// ## 测试目的
/// 验证 PullRequestsConfig 能够从有效的 JSON 字符串反序列化。
///
/// ## 测试场景
/// 1. 准备有效的 JSON 字符串
/// 2. 反序列化为 PullRequestsConfig
/// 3. 验证字段值正确
///
/// ## 预期结果
/// - 字段值与 JSON 中的值一致
#[test]
fn test_pr_config_deserialization_with_valid_json_deserializes_config() -> Result<()> {
    // Arrange: 准备有效的 JSON 字符串
    let json = r#"{"auto_accept_change_type":true}"#;

    // Act: 反序列化为 PullRequestsConfig
    let config: PullRequestsConfig = serde_json::from_str(json)?;

    // Assert: 验证字段值正确
    assert_eq!(config.auto_accept_change_type, Some(true));
    Ok(())
}

/// 测试 PR 配置空 JSON 反序列化
///
/// ## 测试目的
/// 验证 PullRequestsConfig 能够从空 JSON 对象反序列化（使用默认值）。
///
/// ## 测试场景
/// 1. 准备空 JSON 对象
/// 2. 反序列化为 PullRequestsConfig
/// 3. 验证字段使用默认值
///
/// ## 预期结果
/// - 字段使用默认值（None）
#[test]
fn test_pr_config_empty_deserialization_with_empty_json_deserializes_config() -> Result<()> {
    // Arrange: 准备空 JSON 对象
    let json = r#"{}"#;

    // Act: 反序列化为 PullRequestsConfig
    let config: PullRequestsConfig = serde_json::from_str(json)?;

    // Assert: 验证默认值正确
    assert_eq!(config.auto_accept_change_type, None);
    Ok(())
}

/// 测试 PR 配置参数化
///
/// ## 测试目的
/// 使用参数化测试验证 PullRequestsConfig 的序列化和反序列化一致性。
///
/// ## 测试场景
/// 1. 使用不同的 auto_accept_change_type 值创建配置
/// 2. 测试序列化和反序列化的一致性
///
/// ## 预期结果
/// - 序列化和反序列化结果一致
#[rstest]
#[case(None)]
#[case(Some(true))]
#[case(Some(false))]
fn test_pr_config_parametrized_with_various_values_creates_config(
    #[case] auto_accept: Option<bool>,
) -> Result<()> {
    // Arrange: 准备参数化测试的配置值
    let config = PullRequestsConfig {
        auto_accept_change_type: auto_accept,
    };

    // Arrange: 准备测试序列化和反序列化的一致性
    let json = serde_json::to_string(&config)?;
    let deserialized: PullRequestsConfig = serde_json::from_str(&json)?;

    assert_eq!(deserialized.auto_accept_change_type, auto_accept);
    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试分支配置无效 JSON 处理
///
/// ## 测试目的
/// 验证当 JSON 格式无效时，BranchConfig 反序列化返回错误。
///
/// ## 测试场景
/// 1. 准备无效的 JSON 字符串
/// 2. 尝试反序列化
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回 JSON 解析错误
#[test]
fn test_branch_config_invalid_json_with_invalid_json_returns_error() {
    // Arrange: 准备无效的 JSON 字符串
    let invalid_json = r#"{"prefix": invalid}"#;

    // Act: 尝试反序列化
    let result = serde_json::from_str::<BranchConfig>(invalid_json);

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试 PR 配置无效 JSON 处理
///
/// ## 测试目的
/// 验证当 JSON 格式无效时，PullRequestsConfig 反序列化返回错误。
///
/// ## 测试场景
/// 1. 准备无效的 JSON 字符串（类型不匹配）
/// 2. 尝试反序列化
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回 JSON 解析错误
#[test]
fn test_pr_config_invalid_json_with_invalid_json_returns_error() {
    // Arrange: 准备无效的 JSON 字符串
    let invalid_json = r#"{"auto_accept_change_type": "not_boolean"}"#;
    let result = serde_json::from_str::<PullRequestsConfig>(invalid_json);

    assert!(result.is_err());
}

/// 测试分支配置 null 值处理
///
/// ## 测试目的
/// 验证 BranchConfig 能够正确处理 JSON 中的 null 值。
///
/// ## 测试场景
/// 1. 准备包含 null 值的 JSON
/// 2. 反序列化为 BranchConfig
/// 3. 验证 null 值被转换为 None
///
/// ## 预期结果
/// - null 值被正确转换为 None
#[test]
fn test_branch_config_with_null_values() -> Result<()> {
    // Arrange: 准备测试包含 null 值的 JSON
    let json = r#"{"prefix":null,"ignore":[]}"#;
    let config: BranchConfig = serde_json::from_str(json)?;

    assert_eq!(config.prefix, None);
    assert!(config.ignore.is_empty());
    Ok(())
}

/// 测试分支配置空忽略列表
///
/// ## 测试目的
/// 验证当 ignore 列表为空时，序列化时会被跳过（skip_serializing_if）。
///
/// ## 测试场景
/// 1. 创建包含空 ignore 列表的配置
/// 2. 序列化为 JSON
/// 3. 验证 ignore 字段不在 JSON 中
///
/// ## 预期结果
/// - 空 ignore 列表不会被序列化
#[test]
fn test_branch_config_empty_ignore_list() -> Result<()> {
    // Arrange: 准备测试空的忽略列表
    let config = BranchConfig {
        prefix: Some("feature".to_string()),
        ignore: vec![],
    };

    let json = serde_json::to_string(&config)?;

    // 由于 skip_serializing_if = "Vec::is_empty"，空数组不会被序列化
    assert!(!json.contains(r#""ignore""#));
    Ok(())
}

/// 测试分支配置特殊字符处理
///
/// ## 测试目的
/// 验证 BranchConfig 能够正确处理包含特殊字符的字符串值。
///
/// ## 测试场景
/// 1. 创建包含特殊字符的配置（连字符、斜杠、点）
/// 2. 测试序列化和反序列化
/// 3. 验证特殊字符被正确保存
///
/// ## 预期结果
/// - 特殊字符被正确保存和读取
#[test]
fn test_branch_config_special_characters() -> Result<()> {
    // Arrange: 准备测试特殊字符的处理
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

/// 测试配置克隆功能
///
/// ## 测试目的
/// 验证 BranchConfig 和 PullRequestsConfig 的 Clone trait 实现正确。
///
/// ## 测试场景
/// 1. 创建配置实例
/// 2. 克隆配置
/// 3. 验证克隆后的配置与原配置一致
///
/// ## 预期结果
/// - 克隆后的配置与原配置字段值一致
#[test]
fn test_config_clone() {
    // Arrange: 准备测试配置的克隆功能
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

/// 测试配置 Debug 输出
///
/// ## 测试目的
/// 验证 BranchConfig 和 PullRequestsConfig 的 Debug trait 实现正确。
///
/// ## 测试场景
/// 1. 创建配置实例
/// 2. 格式化 Debug 输出
/// 3. 验证输出包含配置类型名和字段值
///
/// ## 预期结果
/// - Debug 输出包含配置类型名和字段值
#[test]
fn test_config_debug() {
    // Arrange: 准备测试配置的 Debug 输出
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
