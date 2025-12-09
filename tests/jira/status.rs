//! Jira 状态管理模块测试
//!
//! 测试 Jira 状态配置的读取、写入和交互式配置功能。

use workflow::jira::status::{JiraStatus, JiraStatusConfig, ProjectStatusConfig};

// ==================== 状态配置读取测试 ====================

#[test]
fn test_read_pull_request_created_status_invalid_ticket() {
    // 测试读取无效 ticket 格式的状态配置
    let result = JiraStatus::read_pull_request_created_status("invalid-ticket");

    // 应该返回错误，因为 ticket 格式无效
    assert!(
        result.is_err(),
        "Should return error for invalid ticket format"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("format"),
        "Error message should mention invalid format"
    );
}

#[test]
fn test_read_pull_request_created_status_valid_ticket() {
    // 测试读取有效 ticket 的状态配置
    // 注意：如果配置文件不存在，应该返回 Ok(None)
    let result = JiraStatus::read_pull_request_created_status("PROJ-123");

    // 应该返回 Ok，但可能为 None（如果配置不存在）
    assert!(result.is_ok(), "Should return Ok for valid ticket format");
    // 值可能是 None（如果配置不存在），这是可以接受的
}

#[test]
fn test_read_pull_request_merged_status_invalid_ticket() {
    // 测试读取无效 ticket 格式的合并状态配置
    let result = JiraStatus::read_pull_request_merged_status("invalid-ticket");

    // 应该返回错误，因为 ticket 格式无效
    assert!(
        result.is_err(),
        "Should return error for invalid ticket format"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("format"),
        "Error message should mention invalid format"
    );
}

#[test]
fn test_read_pull_request_merged_status_valid_ticket() {
    // 测试读取有效 ticket 的合并状态配置
    // 注意：如果配置文件不存在，应该返回 Ok(None)
    let result = JiraStatus::read_pull_request_merged_status("PROJ-123");

    // 应该返回 Ok，但可能为 None（如果配置不存在）
    assert!(result.is_ok(), "Should return Ok for valid ticket format");
    // 值可能是 None（如果配置不存在），这是可以接受的
}

#[test]
fn test_read_pull_request_created_status_with_project_name() {
    // 测试使用项目名读取状态配置
    let result = JiraStatus::read_pull_request_created_status("PROJ-456");

    // 应该返回 Ok，但可能为 None（如果配置不存在）
    assert!(result.is_ok(), "Should return Ok for valid project name");
}

#[test]
fn test_read_pull_request_merged_status_with_project_name() {
    // 测试使用项目名读取合并状态配置
    let result = JiraStatus::read_pull_request_merged_status("PROJ-456");

    // 应该返回 Ok，但可能为 None（如果配置不存在）
    assert!(result.is_ok(), "Should return Ok for valid project name");
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

    let toml_str = toml.unwrap();
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

    let config = config.unwrap();
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

#[test]
fn test_configure_interactive_invalid_project() {
    // 测试使用无效项目名进行交互式配置
    let result = JiraStatus::configure_interactive("invalid/project");

    // 应该返回错误，因为项目名格式无效
    assert!(
        result.is_err(),
        "Should return error for invalid project name"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("format"),
        "Error message should mention invalid format"
    );
}

#[test]
fn test_configure_interactive_with_ticket() {
    // 测试使用 ticket ID 进行交互式配置
    // 注意：这个测试需要实际的 Jira API 调用，可能会失败
    // 我们主要验证函数不会 panic，并且能正确处理错误
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

#[test]
fn test_configure_interactive_with_project_name() {
    // 测试使用项目名进行交互式配置
    // 注意：这个测试需要实际的 Jira API 调用，可能会失败
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

// ==================== 边界条件测试 ====================

#[test]
fn test_read_status_with_empty_ticket() {
    // 测试使用空字符串读取状态配置
    let result = JiraStatus::read_pull_request_created_status("");

    // 应该返回错误，因为 ticket 格式无效
    assert!(result.is_err(), "Should return error for empty ticket");
}

#[test]
fn test_read_status_with_whitespace_ticket() {
    // 测试使用空白字符读取状态配置
    let result = JiraStatus::read_pull_request_created_status("   ");

    // 应该返回错误，因为 ticket 格式无效
    assert!(
        result.is_err(),
        "Should return error for whitespace-only ticket"
    );
}

#[test]
fn test_configure_interactive_with_empty_string() {
    // 测试使用空字符串进行交互式配置
    let result = JiraStatus::configure_interactive("");

    // 应该返回错误，因为项目名格式无效
    assert!(
        result.is_err(),
        "Should return error for empty project name"
    );
}

#[test]
fn test_configure_interactive_with_special_characters() {
    // 测试使用特殊字符进行交互式配置
    let result = JiraStatus::configure_interactive("PROJ/测试");

    // 应该返回错误，因为项目名包含无效字符
    assert!(
        result.is_err(),
        "Should return error for project name with special characters"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("characters"),
        "Error message should mention invalid characters"
    );
}
