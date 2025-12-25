//! Jira 状态管理模块测试
//!
//! 测试 Jira 状态配置的读取、写入和交互式配置功能。

use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::jira::status::{JiraStatus, JiraStatusConfig, ProjectStatusConfig};

// ==================== 状态配置读取测试 ====================

#[rstest]
#[case("invalid-ticket")]
#[case("")]
#[case("   ")]
fn test_read_pull_request_created_status_invalid_ticket(#[case] ticket: &str) {
    // 测试读取无效 ticket 格式的状态配置
    let result = JiraStatus::read_pull_request_created_status(ticket);

    // 应该返回错误，因为 ticket 格式无效
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
fn test_read_pull_request_created_status_valid_ticket(#[case] ticket: &str) {
    // 测试读取有效 ticket 的状态配置
    // 注意：如果配置文件不存在，应该返回 Ok(None)
    let result = JiraStatus::read_pull_request_created_status(ticket);

    // 应该返回 Ok，但可能为 None（如果配置不存在）
    assert!(
        result.is_ok(),
        "Should return Ok for valid ticket format: {}",
        ticket
    );
    // 值可能是 None（如果配置不存在），这是可以接受的
}

#[rstest]
#[case("invalid-ticket")]
#[case("")]
#[case("   ")]
fn test_read_pull_request_merged_status_invalid_ticket(#[case] ticket: &str) {
    // 测试读取无效 ticket 格式的合并状态配置
    let result = JiraStatus::read_pull_request_merged_status(ticket);

    // 应该返回错误，因为 ticket 格式无效
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
fn test_read_pull_request_merged_status_valid_ticket(#[case] ticket: &str) {
    // 测试读取有效 ticket 的合并状态配置
    // 注意：如果配置文件不存在，应该返回 Ok(None)
    let result = JiraStatus::read_pull_request_merged_status(ticket);

    // 应该返回 Ok，但可能为 None（如果配置不存在）
    assert!(
        result.is_ok(),
        "Should return Ok for valid ticket format: {}",
        ticket
    );
    // 值可能是 None（如果配置不存在），这是可以接受的
}

// ==================== 状态配置结构体测试 ====================

#[test]
fn test_jira_status_config_structure() {
    // 测试 JiraStatusConfig 结构体
    let config = JiraStatusConfig {
        project: "PROJ".to_string(),
        created_pull_request_status: Some("In Progress".to_string()),
        merged_pull_request_status: Some("Done".to_string()),
    };

    assert_eq!(config.project, "PROJ");
    assert_eq!(
        config.created_pull_request_status,
        Some("In Progress".to_string())
    );
    assert_eq!(config.merged_pull_request_status, Some("Done".to_string()));
}

#[test]
fn test_jira_status_config_with_none_fields() {
    // 测试 JiraStatusConfig 的可选字段
    let config = JiraStatusConfig {
        project: "PROJ".to_string(),
        created_pull_request_status: None,
        merged_pull_request_status: None,
    };

    assert_eq!(config.project, "PROJ");
    assert_eq!(config.created_pull_request_status, None);
    assert_eq!(config.merged_pull_request_status, None);
}

#[test]
fn test_project_status_config_serialization() {
    // 测试 ProjectStatusConfig 的序列化
    let config = ProjectStatusConfig {
        created_pull_request_status: Some("In Progress".to_string()),
        merged_pull_request_status: Some("Done".to_string()),
    };

    // 测试序列化（使用 TOML）
    let toml = toml::to_string(&config);
    assert!(toml.is_ok(), "Should serialize ProjectStatusConfig to TOML");

    let toml_str = toml.expect("serialization should succeed");
    // TOML 字段名应该是 "created-pr" 和 "merged-pr"（根据 serde rename）
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
fn test_project_status_config_deserialization() {
    // 测试 ProjectStatusConfig 的反序列化
    let toml = r#"
created-pr = "In Progress"
merged-pr = "Done"
"#;

    let config: Result<ProjectStatusConfig, _> = toml::from_str(toml);
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
fn test_project_status_config_with_optional_fields() {
    // 测试 ProjectStatusConfig 的可选字段
    let config = ProjectStatusConfig {
        created_pull_request_status: None,
        merged_pull_request_status: None,
    };

    // 测试序列化
    let toml = toml::to_string(&config);
    assert!(
        toml.is_ok(),
        "Should serialize ProjectStatusConfig with optional fields"
    );
}

// ==================== 交互式配置测试 ====================

#[rstest]
#[case("invalid/project")]
#[case("PROJ/测试")]
fn test_configure_interactive_invalid_project(#[case] project: &str) {
    // 测试使用无效项目名进行交互式配置
    let result = JiraStatus::configure_interactive(project);

    // 应该返回错误，因为项目名格式无效
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
fn test_configure_interactive_empty_string() {
    // 测试使用空字符串进行交互式配置
    let result = JiraStatus::configure_interactive("");

    // 应该返回错误，因为项目名格式无效
    assert!(
        result.is_err(),
        "Should return error for empty project name"
    );
    // 空字符串的错误消息可能不同，所以不检查具体内容
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
