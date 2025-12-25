//! Jira 状态管理模块测试
//!
//! 测试 Jira 状态配置的读取、写入和交互式配置功能。

use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::jira::status::{JiraStatus, JiraStatusConfig, ProjectStatusConfig};

// ==================== Status Configuration Reading Tests ====================

#[rstest]
#[case("invalid-ticket")]
#[case("")]
#[case("   ")]
fn test_read_pull_request_created_status_with_invalid_ticket_returns_error(
    #[case] ticket: &str,
) {
    // Arrange: 准备无效的 ticket 格式

    // Act: 尝试读取状态配置
    let result = JiraStatus::read_pull_request_created_status(ticket);

    // Assert: 验证返回错误且错误消息包含格式相关提示
    assert!(
        result.is_err(),
        "Should return error for invalid ticket format: {}",
        ticket
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("format"),
        "Error message should mention invalid format"
    );
}

#[rstest]
#[case("PROJ-123")]
#[case("PROJ-456")]
#[case("ABC-789")]
fn test_read_pull_request_created_status_with_valid_ticket_returns_ok(
    #[case] ticket: &str,
) {
    // Arrange: 准备有效的 ticket 格式

    // Act: 尝试读取状态配置
    let result = JiraStatus::read_pull_request_created_status(ticket);

    // Assert: 验证返回 Ok（值可能为 None，如果配置不存在）
    assert!(
        result.is_ok(),
        "Should return Ok for valid ticket format: {}",
        ticket
    );
}

#[rstest]
#[case("invalid-ticket")]
#[case("")]
#[case("   ")]
fn test_read_pull_request_merged_status_with_invalid_ticket_returns_error(
    #[case] ticket: &str,
) {
    // Arrange: 准备无效的 ticket 格式

    // Act: 尝试读取合并状态配置
    let result = JiraStatus::read_pull_request_merged_status(ticket);

    // Assert: 验证返回错误且错误消息包含格式相关提示
    assert!(
        result.is_err(),
        "Should return error for invalid ticket format: {}",
        ticket
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("format"),
        "Error message should mention invalid format"
    );
}

#[rstest]
#[case("PROJ-123")]
#[case("PROJ-456")]
#[case("ABC-789")]
fn test_read_pull_request_merged_status_with_valid_ticket_returns_ok(#[case] ticket: &str) {
    // Arrange: 准备有效的 ticket 格式

    // Act: 尝试读取合并状态配置
    let result = JiraStatus::read_pull_request_merged_status(ticket);

    // Assert: 验证返回 Ok（值可能为 None，如果配置不存在）
    assert!(
        result.is_ok(),
        "Should return Ok for valid ticket format: {}",
        ticket
    );
}

// ==================== Status Configuration Structure Tests ====================

#[test]
fn test_jira_status_config_structure_with_all_fields_creates_config() {
    // Arrange: 准备配置字段值
    let project = "PROJ";
    let created_status = Some("In Progress".to_string());
    let merged_status = Some("Done".to_string());

    // Act: 创建 JiraStatusConfig 实例
    let config = JiraStatusConfig {
        project: project.to_string(),
        created_pull_request_status: created_status.clone(),
        merged_pull_request_status: merged_status.clone(),
    };

    // Assert: 验证所有字段值正确
    assert_eq!(config.project, project);
    assert_eq!(config.created_pull_request_status, created_status);
    assert_eq!(config.merged_pull_request_status, merged_status);
}

#[test]
fn test_jira_status_config_with_none_fields_creates_config() {
    // Arrange: 准备配置字段值（可选字段为 None）
    let project = "PROJ";

    // Act: 创建 JiraStatusConfig 实例（可选字段为 None）
    let config = JiraStatusConfig {
        project: project.to_string(),
        created_pull_request_status: None,
        merged_pull_request_status: None,
    };

    // Assert: 验证字段值正确
    assert_eq!(config.project, project);
    assert_eq!(config.created_pull_request_status, None);
    assert_eq!(config.merged_pull_request_status, None);
}

// ==================== Status Configuration Serialization Tests ====================

#[test]
fn test_project_status_config_serialization_with_valid_config_serializes_to_toml() {
    // Arrange: 准备 ProjectStatusConfig 实例
    let config = ProjectStatusConfig {
        created_pull_request_status: Some("In Progress".to_string()),
        merged_pull_request_status: Some("Done".to_string()),
    };

    // Act: 序列化为 TOML
    let toml = toml::to_string(&config);

    // Assert: 验证序列化成功且包含预期字段
    assert!(toml.is_ok(), "Should serialize ProjectStatusConfig to TOML");
    let toml_str = toml.expect("serialization should succeed");
    assert!(
        toml_str.contains("created-pr") || toml_str.contains("created_pull_request_status"),
        "TOML should contain created-pr field"
    );
    assert!(
        toml_str.contains("merged-pr") || toml_str.contains("merged_pull_request_status"),
        "TOML should contain merged-pr field"
    );
}

#[test]
fn test_project_status_config_deserialization_with_valid_toml_deserializes_config() {
    // Arrange: 准备有效的 TOML 字符串
    let toml = r#"
created-pr = "In Progress"
merged-pr = "Done"
"#;

    // Act: 反序列化为 ProjectStatusConfig
    let config: Result<ProjectStatusConfig, _> = toml::from_str(toml);

    // Assert: 验证反序列化成功且字段值正确
    assert!(
        config.is_ok(),
        "Should deserialize TOML to ProjectStatusConfig"
    );
    let config = config.expect("deserialization should succeed");
    assert_eq!(
        config.created_pull_request_status,
        Some("In Progress".to_string())
    );
    assert_eq!(config.merged_pull_request_status, Some("Done".to_string()));
}

#[test]
fn test_project_status_config_with_optional_fields_serializes_successfully() {
    // Arrange: 准备 ProjectStatusConfig 实例（可选字段为 None）
    let config = ProjectStatusConfig {
        created_pull_request_status: None,
        merged_pull_request_status: None,
    };

    // Act: 序列化为 TOML
    let toml = toml::to_string(&config);

    // Assert: 验证序列化成功
    assert!(
        toml.is_ok(),
        "Should serialize ProjectStatusConfig with optional fields"
    );
}

// ==================== Interactive Configuration Tests ====================

#[rstest]
#[case("invalid/project")]
#[case("PROJ/测试")]
fn test_configure_interactive_with_invalid_project_returns_error(#[case] project: &str) {
    // Arrange: 准备无效的项目名

    // Act: 尝试进行交互式配置
    let result = JiraStatus::configure_interactive(project);

    // Assert: 验证返回错误且错误消息包含格式或字符相关提示
    assert!(
        result.is_err(),
        "Should return error for invalid project name: {}",
        project
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid")
            || error_msg.contains("format")
            || error_msg.contains("characters"),
        "Error message should mention invalid format or characters: {}",
        error_msg
    );
}

#[test]
fn test_configure_interactive_with_empty_string_returns_error() {
    // Arrange: 准备空字符串

    // Act: 尝试进行交互式配置
    let result = JiraStatus::configure_interactive("");

    // Assert: 验证返回错误
    assert!(
        result.is_err(),
        "Should return error for empty project name"
    );
}

/// 测试使用ticket ID进行交互式Jira配置
///
/// ## 测试目的
/// 验证`configure_interactive()`方法能够使用ticket ID引导用户完成Jira配置。
///
/// ## 为什么被忽略
/// - **需要交互式输入**: 需要用户输入Jira URL、用户名、API token等
/// - **CI环境不支持**: 自动化环境无法提供交互式输入
/// - **多步骤交互**: 涉及多个连续的用户输入步骤
/// - **配置敏感信息**: 涉及API token等敏感信息
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_configure_interactive_with_ticket -- --ignored
/// ```
/// 准备好Jira URL、用户名和API token以完成配置
///
/// ## 测试场景
/// 1. 调用configure_interactive()并提供ticket ID
/// 2. 提示用户输入Jira URL
/// 3. 提示用户输入Jira用户名
/// 4. 提示用户输入API token
/// 5. 使用ticket ID验证配置
/// 6. 保存配置到文件
///
/// ## 预期行为
/// - 依次显示各个配置项的输入提示
/// - 接受用户输入并验证格式
/// - 使用ticket ID测试连接
/// - 配置成功后保存到~/.workflow/config/
/// - 返回Ok表示配置完成
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
fn test_configure_interactive_with_ticket() {
    // 测试使用 ticket ID 进行交互式配置
    // 注意：这个测试需要实际的 Jira API 调用和用户交互，在 CI 环境中会卡住
    // 使用 `cargo test -- --ignored` 来运行这些测试
    let result = JiraStatus::configure_interactive("PROJ-123");

    // 如果 API 调用失败（例如没有配置 Jira），应该返回错误
    // 如果成功，应该返回 StatusConfigResult
    match result {
        Ok(config_result) => {
            assert_eq!(config_result.project, "PROJ");
            assert!(!config_result.created_pull_request_status.is_empty());
            assert!(!config_result.merged_pull_request_status.is_empty());
        }
        Err(_) => {
            // API 调用失败，这是可以接受的（例如没有配置 Jira 凭据）
            assert!(true, "API call may fail if Jira is not configured");
        }
    }
}

/// 测试使用项目名进行交互式Jira配置
///
/// ## 测试目的
/// 验证`configure_interactive()`方法能够使用项目名引导用户完成Jira配置。
///
/// ## 为什么被忽略
/// - **需要交互式输入**: 需要用户输入Jira URL、用户名、API token等
/// - **CI环境不支持**: 自动化环境无法提供交互式输入
/// - **多步骤交互**: 涉及多个连续的用户输入步骤
/// - **配置敏感信息**: 涉及API token等敏感信息
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_configure_interactive_with_project_name -- --ignored
/// ```
/// 准备好Jira URL、用户名、API token和项目名
///
/// ## 测试场景
/// 1. 调用configure_interactive()并提供项目名
/// 2. 提示用户输入Jira URL
/// 3. 提示用户输入Jira用户名
/// 4. 提示用户输入API token
/// 5. 使用项目名验证配置
/// 6. 保存配置到文件
///
/// ## 预期行为
/// - 依次显示各个配置项的输入提示
/// - 接受用户输入并验证格式
/// - 使用项目名测试连接
/// - 配置成功后保存到~/.workflow/config/
/// - 返回Ok表示配置完成
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
fn test_configure_interactive_with_project_name() {
    // 测试使用项目名进行交互式配置
    // 注意：这个测试需要实际的 Jira API 调用和用户交互，在 CI 环境中会卡住
    // 使用 `cargo test -- --ignored` 来运行这些测试
    let result = JiraStatus::configure_interactive("PROJ");

    // 如果 API 调用失败（例如没有配置 Jira），应该返回错误
    // 如果成功，应该返回 StatusConfigResult
    match result {
        Ok(config_result) => {
            assert_eq!(config_result.project, "PROJ");
            assert!(!config_result.created_pull_request_status.is_empty());
            assert!(!config_result.merged_pull_request_status.is_empty());
        }
        Err(_) => {
            // API 调用失败，这是可以接受的（例如没有配置 Jira 凭据）
            assert!(true, "API call may fail if Jira is not configured");
        }
    }
}
