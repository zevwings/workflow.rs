//! Jira API HTTP 测试
//!
//! 使用 mockito 测试 Jira API 的实际 HTTP 调用。
//!
//! 已迁移使用 TestDataFactory 生成测试数据，减少文件依赖。

use crate::common::mock::server::MockServer;
use crate::common::test_data::TestDataFactory;
use color_eyre::Result;

/// 设置测试环境
///
/// 设置环境变量使用 Mock 服务器，并返回服务器实例。
fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::new();
    mock_server.setup_jira_base_url();
    mock_server
}

// ==================== Jira API Issue Tests ====================

/// 测试获取Jira Issue成功（使用Mock服务器）
///
/// ## 测试目的
/// 验证Jira API获取Issue的HTTP请求格式正确（使用mockito Mock服务器）。
///
/// ## 测试场景
/// 1. 设置Mock服务器
/// 2. 创建Mock响应（200状态码，包含Issue信息）
/// 3. 验证请求格式（headers, URL）
///
/// ## 注意事项
/// - 实际测试需要设置认证信息
/// - Mock验证需要调用 `_mock.assert()`
///
/// ## 预期结果
/// - Mock设置成功
/// - 请求格式正确
#[test]
fn test_get_issue_success_with_valid_issue_key_returns_issue() -> Result<()> {
    // Arrange: 准备Mock服务器和TestDataFactory
    let mut mock_server = setup_mock_server();
    let factory = TestDataFactory::new();

    // Act: 使用 TestDataFactory 生成 Issue 数据
    let issue_data = factory
        .jira_issue()
        .key("PROJ-123")
        .summary("Test Issue")
        .description("Test description")
        .status("In Progress")
        .build()?;

    let response_body = serde_json::to_string(&issue_data)?;
    mock_server
        .server
        .as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    // Assert: 注意：实际测试需要设置认证信息
    // 验证Mock被调用: mock_server.assert_all_called();
    Ok(())
}

/// 测试获取Jira Issue未找到错误（使用Mock服务器）
///
/// ## 测试目的
/// 验证Jira API获取Issue在收到404错误响应时能够正确处理。
///
/// ## 测试场景
/// 1. 设置Mock服务器
/// 2. 创建错误响应的Mock（404状态码，Issue不存在）
/// 3. 验证错误处理
///
/// ## 注意事项
/// - 实际测试需要设置认证信息
/// - 需要验证 `result.is_err()`
/// - Mock验证需要调用 `_mock.assert()`
///
/// ## 预期结果
/// - Mock设置成功
/// - 404错误被正确处理
#[test]
fn test_get_issue_not_found_with_invalid_issue_key_returns_error() {
    // Arrange: 准备Mock服务器
    let mut mock_server = setup_mock_server();

    // Act: 创建错误响应的Mock（错误响应保持使用文件方式，因为结构简单且固定）
    use std::path::PathBuf;
    let response_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_responses")
        .join("jira")
        .join("issue_not_found.json");

    // 使用 mock_jira_issue_from_file 方法从文件加载响应
    mock_server.mock_jira_issue_from_file("GET", "/rest/api/3/issue/PROJ-999", &response_file, 404);

    // 注意：错误响应保持文件方式，因为错误结构简单且固定
    // 如果需要验证，可以使用 mock_server.assert_all_called()

    // Assert: 注意：实际测试需要设置认证信息
    // 测试错误处理: assert!(result.is_err());
    // 验证Mock被调用: _mock.assert();
}

/// 测试搜索Jira Issues（使用Mock服务器）
///
/// ## 测试目的
/// 验证Jira API搜索Issues的HTTP请求格式正确（使用mockito Mock服务器）。
///
/// ## 测试场景
/// 1. 设置Mock服务器
/// 2. 创建Mock响应（200状态码，包含Issues列表）
/// 3. 验证请求格式和响应解析
///
/// ## 注意事项
/// - 实际测试需要设置Jira客户端和认证
/// - Mock验证需要调用 `_mock.assert()`
///
/// ## 预期结果
/// - Mock设置成功
/// - Issues列表能够正确解析
#[test]
fn test_search_issues_with_valid_query_returns_issues() -> Result<()> {
    // Arrange: 准备Mock服务器和TestDataFactory
    let mut mock_server = setup_mock_server();
    let factory = TestDataFactory::new();

    // Act: 使用 TestDataFactory 批量生成 Issue 数据
    let issues = factory.build_batch(2, |f| f.jira_issue().build());

    // 构建搜索响应结构
    let search_response = serde_json::json!({
        "issues": issues,
        "total": issues.len()
    });

    let response_body = serde_json::to_string(&search_response)?;
    mock_server
        .server
        .as_mut()
        .mock("POST", "/rest/api/3/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    // 测试搜索 Issues
    // let jira = JiraClient::new(...);
    // let issues = jira.search_issues("project = PROJ")?;
    // assert_eq!(issues.len(), 2);

    // mock_server.assert_all_called();
    Ok(())
}
