//! Jira API HTTP 测试
//!
//! 使用 mockito 测试 Jira API 的实际 HTTP 调用。

use crate::common::http_helpers::MockServer;
use mockito::Matcher;

/// 设置测试环境
///
/// 设置环境变量使用 Mock 服务器，并返回服务器实例。
fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::new();
    mock_server.setup_jira_base_url();
    mock_server
}

#[test]
fn test_get_issue_success() {
    let mut mock_server = setup_mock_server();

    // 创建 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
        .match_header("accept", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "id": "12345",
            "key": "PROJ-123",
            "fields": {
                "summary": "Test Issue",
                "description": "Test description",
                "status": {
                    "name": "In Progress"
                }
            }
        }"#,
        )
        .create();

    // 注意：实际测试需要设置认证信息
    // 这里仅展示 Mock 设置方式
    // let jira = JiraClient::new(...);
    // let issue = jira.get_issue("PROJ-123")?;
    // assert_eq!(issue.key, "PROJ-123");

    // 验证 Mock 被调用
    // _mock.assert();
}

#[test]
fn test_get_issue_not_found() {
    let mut mock_server = setup_mock_server();

    // 创建错误响应的 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-999")
        .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages":["Issue does not exist or you do not have permission to see it."]}"#)
        .create();

    // 测试错误处理
    // let jira = JiraClient::new(...);
    // let result = jira.get_issue("PROJ-999");
    // assert!(result.is_err());

    // 验证 Mock 被调用
    // _mock.assert();
}

#[test]
fn test_search_issues() {
    let mut mock_server = setup_mock_server();

    let _mock = mock_server
        .server
        .as_mut()
        .mock("POST", "/rest/api/3/search")
        .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "id": "12345",
                    "key": "PROJ-123",
                    "fields": {
                        "summary": "Test Issue 1"
                    }
                },
                {
                    "id": "12346",
                    "key": "PROJ-124",
                    "fields": {
                        "summary": "Test Issue 2"
                    }
                }
            ],
            "total": 2
        }"#,
        )
        .create();

    // 测试搜索 Issues
    // let jira = JiraClient::new(...);
    // let issues = jira.search_issues("project = PROJ")?;
    // assert_eq!(issues.len(), 2);

    // _mock.assert();
}
