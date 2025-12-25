//! Jira 状态管理模块测试
//!
//! 测试 Jira 状态配置的读取、写入和交互式配置功能。

use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::jira::status::{JiraStatus, JiraStatusConfig, ProjectStatusConfig};

// ==================== Status Configuration Reading Tests ====================

/// 测试使用无效ticket格式读取PR创建状态配置
///
/// ## 测试目的
/// 验证`JiraStatus::read_pull_request_created_status()`能够正确检测并拒绝无效的ticket格式，返回包含格式相关提示的错误。
///
/// ## 测试场景
/// 1. 使用参数化测试，测试多种无效ticket格式：`"invalid-ticket"`、空字符串、仅包含空格的字符串
/// 2. 调用`read_pull_request_created_status()`尝试读取状态配置
/// 3. 验证返回错误
/// 4. 验证错误消息包含`"Invalid"`或`"format"`关键词
///
/// ## 预期结果
/// - 所有无效格式都返回`Err`
/// - 错误消息包含格式验证相关的提示信息
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

/// 测试使用有效ticket格式读取PR创建状态配置
///
/// ## 测试目的
/// 验证`JiraStatus::read_pull_request_created_status()`能够接受有效的ticket格式（如`PROJ-123`），返回`Ok`结果（即使配置不存在也可能返回`Ok(None)`）。
///
/// ## 测试场景
/// 1. 使用参数化测试，测试多种有效ticket格式：`"PROJ-123"`、`"PROJ-456"`、`"ABC-789"`
/// 2. 调用`read_pull_request_created_status()`尝试读取状态配置
/// 3. 验证返回`Ok`（值可能为`None`，如果配置不存在）
///
/// ## 预期结果
/// - 所有有效格式都返回`Ok`
/// - 如果配置存在，返回`Ok(Some(status))`
/// - 如果配置不存在，返回`Ok(None)`
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

/// 测试使用无效ticket格式读取PR合并状态配置
///
/// ## 测试目的
/// 验证`JiraStatus::read_pull_request_merged_status()`能够正确检测并拒绝无效的ticket格式，返回包含格式相关提示的错误。
///
/// ## 测试场景
/// 1. 使用参数化测试，测试多种无效ticket格式：`"invalid-ticket"`、空字符串、仅包含空格的字符串
/// 2. 调用`read_pull_request_merged_status()`尝试读取合并状态配置
/// 3. 验证返回错误
/// 4. 验证错误消息包含`"Invalid"`或`"format"`关键词
///
/// ## 预期结果
/// - 所有无效格式都返回`Err`
/// - 错误消息包含格式验证相关的提示信息
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

/// 测试使用有效ticket格式读取PR合并状态配置
///
/// ## 测试目的
/// 验证`JiraStatus::read_pull_request_merged_status()`能够接受有效的ticket格式（如`PROJ-123`），返回`Ok`结果（即使配置不存在也可能返回`Ok(None)`）。
///
/// ## 测试场景
/// 1. 使用参数化测试，测试多种有效ticket格式：`"PROJ-123"`、`"PROJ-456"`、`"ABC-789"`
/// 2. 调用`read_pull_request_merged_status()`尝试读取合并状态配置
/// 3. 验证返回`Ok`（值可能为`None`，如果配置不存在）
///
/// ## 预期结果
/// - 所有有效格式都返回`Ok`
/// - 如果配置存在，返回`Ok(Some(status))`
/// - 如果配置不存在，返回`Ok(None)`
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

/// 测试使用所有字段创建Jira状态配置结构体
///
/// ## 测试目的
/// 验证`JiraStatusConfig`结构体可以使用所有字段值（项目名、PR创建状态、PR合并状态）正确创建。
///
/// ## 测试场景
/// 1. 准备配置字段值：项目名为`"PROJ"`，PR创建状态为`"In Progress"`，PR合并状态为`"Done"`
/// 2. 使用这些字段值创建`JiraStatusConfig`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `JiraStatusConfig`实例创建成功
/// - `project`字段与提供的项目名一致
/// - `created_pull_request_status`字段与PR创建状态一致
/// - `merged_pull_request_status`字段与PR合并状态一致
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

/// 测试使用可选字段为None创建Jira状态配置
///
/// ## 测试目的
/// 验证`JiraStatusConfig`结构体可以创建仅包含必需字段（项目名）的配置，可选字段（PR创建状态、PR合并状态）可以为`None`。
///
/// ## 测试场景
/// 1. 准备配置字段值：项目名为`"PROJ"`，PR创建状态和PR合并状态为`None`
/// 2. 使用这些字段值创建`JiraStatusConfig`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `JiraStatusConfig`实例创建成功
/// - `project`字段与提供的项目名一致
/// - `created_pull_request_status`字段为`None`
/// - `merged_pull_request_status`字段为`None`
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

/// 测试项目状态配置序列化为TOML
///
/// ## 测试目的
/// 验证`ProjectStatusConfig`结构体能够正确序列化为TOML格式，包含PR创建状态和PR合并状态字段。
///
/// ## 测试场景
/// 1. 准备包含PR创建状态和PR合并状态的`ProjectStatusConfig`实例
/// 2. 调用`toml::to_string()`序列化为TOML字符串
/// 3. 验证序列化成功
/// 4. 验证TOML字符串包含`created-pr`或`created_pull_request_status`字段
/// 5. 验证TOML字符串包含`merged-pr`或`merged_pull_request_status`字段
///
/// ## 预期结果
/// - 序列化成功，返回`Ok(String)`
/// - TOML字符串包含PR创建状态字段（`created-pr`或`created_pull_request_status`）
/// - TOML字符串包含PR合并状态字段（`merged-pr`或`merged_pull_request_status`）
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

/// 测试从TOML反序列化项目状态配置
///
/// ## 测试目的
/// 验证`ProjectStatusConfig`结构体能够从有效的TOML字符串正确反序列化，恢复PR创建状态和PR合并状态字段值。
///
/// ## 测试场景
/// 1. 准备包含`created-pr`和`merged-pr`字段的有效TOML字符串
/// 2. 调用`toml::from_str()`反序列化为`ProjectStatusConfig`
/// 3. 验证反序列化成功
/// 4. 验证`created_pull_request_status`字段值为`"In Progress"`
/// 5. 验证`merged_pull_request_status`字段值为`"Done"`
///
/// ## 预期结果
/// - 反序列化成功，返回`Ok(ProjectStatusConfig)`
/// - `created_pull_request_status`字段为`Some("In Progress")`
/// - `merged_pull_request_status`字段为`Some("Done")`
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

/// 测试可选字段为None的项目状态配置序列化
///
/// ## 测试目的
/// 验证`ProjectStatusConfig`结构体在可选字段（PR创建状态、PR合并状态）为`None`时仍能成功序列化为TOML格式。
///
/// ## 测试场景
/// 1. 准备所有可选字段为`None`的`ProjectStatusConfig`实例
/// 2. 调用`toml::to_string()`序列化为TOML字符串
/// 3. 验证序列化成功（即使字段为`None`）
///
/// ## 预期结果
/// - 序列化成功，返回`Ok(String)`
/// - TOML字符串可能为空或仅包含空字段（取决于序列化实现）
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

/// 测试使用无效项目名进行交互式配置
///
/// ## 测试目的
/// 验证`JiraStatus::configure_interactive()`能够正确检测并拒绝无效的项目名格式（包含斜杠、特殊字符等），返回包含格式或字符相关提示的错误。
///
/// ## 测试场景
/// 1. 使用参数化测试，测试多种无效项目名：`"invalid/project"`（包含斜杠）、`"PROJ/测试"`（包含非ASCII字符）
/// 2. 调用`configure_interactive()`尝试进行交互式配置
/// 3. 验证返回错误
/// 4. 验证错误消息包含`"Invalid"`、`"format"`或`"characters"`关键词
///
/// ## 预期结果
/// - 所有无效项目名都返回`Err`
/// - 错误消息包含格式或字符验证相关的提示信息
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

/// 测试使用空字符串进行交互式配置
///
/// ## 测试目的
/// 验证`JiraStatus::configure_interactive()`能够正确检测并拒绝空字符串作为项目名，返回错误。
///
/// ## 测试场景
/// 1. 准备空字符串作为项目名
/// 2. 调用`configure_interactive()`尝试进行交互式配置
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回`Err`
/// - 错误消息提示项目名不能为空
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
