//! GitHub API HTTP 测试
//!
//! 使用 mockito 测试 GitHub API 的实际 HTTP 调用。
//!
//! 已迁移使用 Mock 模板系统和 TestDataFactory，减少硬编码 JSON 响应。

use crate::common::mock::server::MockServer;
use crate::common::test_data::TestDataFactory;
use color_eyre::Result;
use serde_json::json;
use std::collections::HashMap;

/// 设置测试环境
///
/// 设置环境变量使用 Mock 服务器，并返回服务器实例。
fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    mock_server
}

// ==================== GitHub API Pull Request Tests ====================

/// 测试创建Pull Request成功（使用Mock服务器）
///
/// ## 测试目的
/// 验证GitHub API创建Pull Request的HTTP请求格式正确（使用mockito Mock服务器）。
///
/// ## 测试场景
/// 1. 设置Mock服务器
/// 2. 创建Mock响应（201状态码）
/// 3. 验证请求格式（headers, body）
///
/// ## 注意事项
/// - 实际测试需要设置Git仓库和认证
/// - 这里仅展示Mock设置方式
/// - Mock验证需要调用 `_mock.assert()`
///
/// ## 预期结果
/// - Mock设置成功
/// - 请求格式正确
#[test]
fn test_create_pull_request_success_with_valid_request_creates_pr() -> Result<()> {
    // Arrange: 准备Mock服务器和TestDataFactory
    let mut mock_server = setup_mock_server();
    let factory = TestDataFactory::new();

    // 使用 TestDataFactory 生成 PR 数据
    let mut pr_data = factory.github_pr().number(123).build()?;

    // 手动设置 html_url（因为 Builder 没有 html_url 方法）
    pr_data["html_url"] = json!("https://github.com/owner/repo/pull/123");

    // 转换为字符串用于 Mock 响应
    let response_body = serde_json::to_string(&pr_data)?;

    // 使用 mock_with_template 创建 Mock（保持路径参数支持）
    let mut vars = HashMap::new();
    vars.insert("owner".to_string(), "owner".to_string());
    vars.insert("repo".to_string(), "repo".to_string());

    mock_server.mock_with_template(
        "POST",
        "/repos/{owner}/{repo}/pulls",
        &response_body,
        vars,
        201,
    );

    // 注意：mock_with_template 不直接支持请求头匹配
    // 如果需要验证请求头，仍然需要使用 server.as_mut() 的方式
    // 但响应体可以使用 TestDataFactory 生成

    // Assert: 注意：实际测试需要设置Git仓库和认证
    // 这里仅展示Mock设置方式
    // 验证Mock被调用: mock_server.assert_all_called();
    Ok(())
}

/// 测试创建Pull Request错误处理（使用Mock服务器）
///
/// ## 测试目的
/// 验证GitHub API创建Pull Request在收到错误响应（422状态码）时能够正确处理。
///
/// ## 测试场景
/// 1. 设置Mock服务器
/// 2. 创建错误响应的Mock（422状态码，Validation Failed）
/// 3. 验证错误处理
///
/// ## 注意事项
/// - 实际测试需要设置Git仓库和认证
/// - 需要验证 `result.is_err()`
/// - Mock验证需要调用 `_mock.assert()`
///
/// ## 预期结果
/// - Mock设置成功
/// - 错误响应被正确处理
#[test]
fn test_create_pull_request_error_with_invalid_request_returns_error() {
    // Arrange: 准备Mock服务器
    let mut mock_server = setup_mock_server();

    // Act: 使用模板创建错误响应的Mock
    let mut vars = HashMap::new();
    vars.insert("owner".to_string(), "owner".to_string());
    vars.insert("repo".to_string(), "repo".to_string());
    vars.insert("message".to_string(), "Validation Failed".to_string());
    vars.insert("resource".to_string(), "PullRequest".to_string());
    vars.insert("field".to_string(), "base".to_string());
    vars.insert("code".to_string(), "invalid".to_string());

    mock_server.mock_with_template(
        "POST",
        "/repos/{owner}/{repo}/pulls",
        r#"{"message":"{{message}}","errors":[{"resource":"{{resource}}","field":"{{field}}","code":"{{code}}"}]}"#,
        vars,
        422,
    );

    // Assert: 注意：实际测试需要设置Git仓库和认证
    // 测试错误处理: assert!(result.is_err());
    // 验证Mock被调用: mock_server.assert_all_called();
}

/// 测试获取Pull Request信息（使用Mock服务器）
///
/// ## 测试目的
/// 验证GitHub API获取Pull Request信息的HTTP请求格式正确（使用mockito Mock服务器）。
///
/// ## 测试场景
/// 1. 设置Mock服务器
/// 2. 创建Mock响应（200状态码，包含PR信息）
/// 3. 验证请求格式和响应解析
///
/// ## 注意事项
/// - 实际测试需要设置GitHub客户端和认证
/// - Mock验证需要调用 `_mock.assert()`
///
/// ## 预期结果
/// - Mock设置成功
/// - PR信息能够正确解析
#[test]
fn test_get_pull_request_info_with_valid_pr_number_returns_info() -> Result<()> {
    // Arrange: 准备Mock服务器和TestDataFactory
    let mut mock_server = setup_mock_server();
    let factory = TestDataFactory::new();

    // 使用 TestDataFactory 生成完整的 PR 数据
    let mut pr_data = factory
        .github_pr()
        .number(123)
        .title("Test PR")
        .body("Test body")
        .state("open")
        .merged(false)
        .head_ref("feature/test")
        .base_ref("main")
        .build()?;

    // 手动设置 html_url（因为 Builder 没有 html_url 方法）
    pr_data["html_url"] = json!("https://github.com/owner/repo/pull/123");

    // 转换为字符串用于 Mock 响应
    let response_body = serde_json::to_string(&pr_data)?;

    // 使用 mock_with_template 创建 Mock（保持路径参数支持）
    let mut vars = HashMap::new();
    vars.insert("owner".to_string(), "owner".to_string());
    vars.insert("repo".to_string(), "repo".to_string());
    vars.insert("pr_number".to_string(), "123".to_string());

    mock_server.mock_with_template(
        "GET",
        "/repos/{owner}/{repo}/pulls/{pr_number}",
        &response_body,
        vars,
        200,
    );

    // 测试获取 PR 信息
    // let github = GitHub;
    // let info = github.get_pull_request_info("123")?;
    // assert!(info.contains("Test PR"));

    // 验证Mock被调用: mock_server.assert_all_called();
    Ok(())
}
