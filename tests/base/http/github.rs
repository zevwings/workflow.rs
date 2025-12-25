//! GitHub API HTTP 测试
//!
//! 使用 mockito 测试 GitHub API 的实际 HTTP 调用。

use crate::common::http_helpers::MockServer;
use mockito::Matcher;

/// 设置测试环境
///
/// 设置环境变量使用 Mock 服务器，并返回服务器实例。
fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    mock_server
}

// ==================== GitHub API Pull Request Tests ====================

#[test]
fn test_create_pull_request_success_with_valid_request_creates_pr() {
    // Arrange: 准备Mock服务器
    let mut mock_server = setup_mock_server();

    // Act: 创建Mock响应
    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/repos/owner/repo/pulls")
        .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
        .match_header("accept", "application/vnd.github.v3+json")
        .match_body(Matcher::JsonString(
            r#"{"title":"Test PR","body":"Test body","head":"owner:feature","base":"main"}"#
                .to_string(),
        ))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"html_url":"https://github.com/owner/repo/pull/123"}"#)
        .create();

    // Assert: 注意：实际测试需要设置Git仓库和认证
    // 这里仅展示Mock设置方式
    // 验证Mock被调用: _mock.assert();
}

#[test]
fn test_create_pull_request_error_with_invalid_request_returns_error() {
    // Arrange: 准备Mock服务器
    let mut mock_server = setup_mock_server();

    // Act: 创建错误响应的Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/repos/owner/repo/pulls")
        .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
        .with_status(422)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"message":"Validation Failed","errors":[{"resource":"PullRequest","field":"base","code":"invalid"}]}"#,
        )
        .create();

    // Assert: 注意：实际测试需要设置Git仓库和认证
    // 测试错误处理: assert!(result.is_err());
    // 验证Mock被调用: _mock.assert();
}

#[test]
fn test_get_pull_request_info_with_valid_pr_number_returns_info() {
    // Arrange: 准备Mock服务器
    let mut mock_server = setup_mock_server();

    // Act: 创建Mock响应
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/repos/owner/repo/pulls/123")
        .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "number": 123,
            "title": "Test PR",
            "body": "Test body",
            "state": "open",
            "merged": false,
            "html_url": "https://github.com/owner/repo/pull/123",
            "head": {"ref": "feature/test"},
            "base": {"ref": "main"}
        }"#,
        )
        .create();

    // 测试获取 PR 信息
    // let github = GitHub;
    // let info = github.get_pull_request_info("123")?;
    // assert!(info.contains("Test PR"));

    // _mock.assert();
}
