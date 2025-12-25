//! GitHub PR 模块测试
//!
//! 测试 GitHub PR 平台的 API 集成、请求构建和响应解析。
//!
//! ## 测试策略
//!
//! - 涉及序列化/反序列化的测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试请求和响应结构体的正确性
//! - 测试各种边界情况和可选字段处理

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};

use workflow::pr::github::{
    requests::{CreatePullRequestRequest, MergePullRequestRequest, UpdatePullRequestRequest},
    responses::{CreatePullRequestResponse, GitHubUser, PullRequestBranch, PullRequestInfo},
};

// ==================== Fixtures ====================

#[fixture]
fn sample_create_request() -> CreatePullRequestRequest {
    CreatePullRequestRequest {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        head: "feature/test".to_string(),
        base: "main".to_string(),
    }
}

#[fixture]
fn sample_merge_request() -> MergePullRequestRequest {
    MergePullRequestRequest {
        commit_title: None,
        commit_message: None,
        merge_method: "squash".to_string(),
    }
}

// ==================== 请求结构体测试 ====================

#[rstest]
fn test_create_request_structure(sample_create_request: CreatePullRequestRequest) {
    assert_eq!(sample_create_request.title, "Test PR");
    assert_eq!(sample_create_request.body, "Test body");
    assert_eq!(sample_create_request.head, "feature/test");
    assert_eq!(sample_create_request.base, "main");
}

#[rstest]
#[case("Test PR", "Test body", "feature/test", "main")]
#[case("", "", "", "")]
#[case(
    "Long Title with Special Chars !@#",
    "Long Body\nwith\nmultiple\nlines",
    "feature/long-branch-name",
    "develop"
)]
fn test_create_pr_request_serialization(
    #[case] title: &str,
    #[case] body: &str,
    #[case] head: &str,
    #[case] base: &str,
) -> Result<()> {
    let request = CreatePullRequestRequest {
        title: title.to_string(),
        body: body.to_string(),
        head: head.to_string(),
        base: base.to_string(),
    };

    let json_str = serde_json::to_string(&request)?;

    // 验证 JSON 是有效的，并包含必要的字段
    let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
    let obj = json_value.as_object().expect("Should be a JSON object");

    // 验证字段存在且值正确
    assert_eq!(obj.get("title").and_then(|v| v.as_str()), Some(title));
    assert_eq!(obj.get("body").and_then(|v| v.as_str()), Some(body));
    assert_eq!(obj.get("head").and_then(|v| v.as_str()), Some(head));
    assert_eq!(obj.get("base").and_then(|v| v.as_str()), Some(base));
    Ok(())
}

#[rstest]
fn test_merge_request_structure(sample_merge_request: MergePullRequestRequest) {
    assert_eq!(sample_merge_request.commit_title, None);
    assert_eq!(sample_merge_request.commit_message, None);
    assert_eq!(sample_merge_request.merge_method, "squash");
}

#[rstest]
#[case(None, None, "squash")]
#[case(Some("Merge PR #123"), Some("Merged via workflow"), "merge")]
#[case(Some("Custom Title"), None, "rebase")]
fn test_merge_pr_request_serialization(
    #[case] commit_title: Option<&str>,
    #[case] commit_message: Option<&str>,
    #[case] merge_method: &str,
) -> Result<()> {
    let request = MergePullRequestRequest {
        commit_title: commit_title.map(|s| s.to_string()),
        commit_message: commit_message.map(|s| s.to_string()),
        merge_method: merge_method.to_string(),
    };

    let json_str = serde_json::to_string(&request)?;
    assert!(
        json_str.contains(merge_method),
        "JSON should contain merge_method"
    );

    // 验证 None 字段被跳过
    if commit_title.is_none() {
        assert!(
            !json_str.contains("commit_title"),
            "None fields should be skipped"
        );
    }
    if commit_message.is_none() {
        assert!(
            !json_str.contains("commit_message"),
            "None fields should be skipped"
        );
    }
    Ok(())
}

#[rstest]
#[case(None, None, Some("closed"), None)]
#[case(Some("New Title"), None, None, None)]
#[case(None, Some("New Body"), None, None)]
#[case(Some("New Title"), Some("New Body"), None, None)]
fn test_update_pr_request_serialization(
    #[case] title: Option<&str>,
    #[case] body: Option<&str>,
    #[case] state: Option<&str>,
    #[case] base: Option<&str>,
) -> Result<()> {
    let request = UpdatePullRequestRequest {
        title: title.map(|s| s.to_string()),
        body: body.map(|s| s.to_string()),
        state: state.map(|s| s.to_string()),
        base: base.map(|s| s.to_string()),
    };

    let json_str = serde_json::to_string(&request)?;

    // 验证存在的字段
    if let Some(t) = title {
        assert!(json_str.contains(t), "JSON should contain title");
    }
    if let Some(b) = body {
        assert!(json_str.contains(b), "JSON should contain body");
    }
    if let Some(s) = state {
        assert!(json_str.contains(s), "JSON should contain state");
    }

    // 验证 None 字段被跳过
    if title.is_none() {
        assert!(
            !json_str.contains("\"title\""),
            "None title should be skipped"
        );
    }
    if body.is_none() {
        assert!(
            !json_str.contains("\"body\""),
            "None body should be skipped"
        );
    }
    if state.is_none() {
        assert!(
            !json_str.contains("\"state\""),
            "None state should be skipped"
        );
    }
    Ok(())
}

// ==================== 响应结构体测试 ====================

#[test]
fn test_create_pull_request_response_structure() {
    // 测试创建 PR 响应结构
    let response = CreatePullRequestResponse {
        html_url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert_eq!(response.html_url, "https://github.com/owner/repo/pull/123");
}

#[test]
fn test_create_pull_request_response_deserialization() -> Result<()> {
    // 测试创建 PR 响应的反序列化
    let json = r#"{"html_url": "https://github.com/owner/repo/pull/123"}"#;

    let response: CreatePullRequestResponse = serde_json::from_str(json)?;
    assert_eq!(response.html_url, "https://github.com/owner/repo/pull/123");
    Ok(())
}

#[test]
fn test_pull_request_info_structure() {
    // 测试 PR 信息结构
    let pr_info = PullRequestInfo {
        number: 123,
        title: "Test PR".to_string(),
        body: Some("Test body".to_string()),
        state: "open".to_string(),
        merged: false,
        merged_at: None,
        html_url: "https://github.com/owner/repo/pull/123".to_string(),
        head: PullRequestBranch {
            ref_name: "feature/test".to_string(),
        },
        base: PullRequestBranch {
            ref_name: "main".to_string(),
        },
        user: None,
    };

    assert_eq!(pr_info.number, 123);
    assert_eq!(pr_info.title, "Test PR");
    assert_eq!(pr_info.body, Some("Test body".to_string()));
    assert_eq!(pr_info.state, "open");
    assert!(!pr_info.merged);
    assert_eq!(pr_info.head.ref_name, "feature/test");
    assert_eq!(pr_info.base.ref_name, "main");
}

#[test]
fn test_pull_request_info_deserialization() -> Result<()> {
    // 测试 PR 信息的反序列化
    let json = r#"{
        "number": 123,
        "title": "Test PR",
        "body": "Test body",
        "state": "open",
        "merged": false,
        "html_url": "https://github.com/owner/repo/pull/123",
        "head": {"ref": "feature/test"},
        "base": {"ref": "main"}
    }"#;

    let pr_info: PullRequestInfo = serde_json::from_str(json)?;
    assert_eq!(pr_info.number, 123);
    assert_eq!(pr_info.title, "Test PR");
    assert_eq!(pr_info.state, "open");
    Ok(())
}

#[test]
fn test_pull_request_info_merged_state() -> Result<()> {
    // 测试 PR 信息的合并状态
    let json = r#"{
        "number": 123,
        "title": "Merged PR",
        "body": null,
        "state": "closed",
        "merged": true,
        "merged_at": "2024-01-01T00:00:00Z",
        "html_url": "https://github.com/owner/repo/pull/123",
        "head": {"ref": "feature/test"},
        "base": {"ref": "main"}
    }"#;

    let pr_info: PullRequestInfo = serde_json::from_str(json)?;
    assert!(pr_info.merged, "Should be marked as merged");
    assert_eq!(pr_info.state, "closed");
    assert!(pr_info.merged_at.is_some());
    Ok(())
}

#[test]
fn test_pull_request_branch_structure() {
    // 测试 PR 分支结构
    let branch = PullRequestBranch {
        ref_name: "feature/test".to_string(),
    };

    assert_eq!(branch.ref_name, "feature/test");
}

#[test]
fn test_pull_request_branch_deserialization() -> Result<()> {
    // 测试 PR 分支的反序列化（注意 JSON 中使用 "ref" 字段）
    let json = r#"{"ref": "feature/test"}"#;

    let branch: PullRequestBranch = serde_json::from_str(json)?;
    assert_eq!(branch.ref_name, "feature/test");
    Ok(())
}

#[test]
fn test_github_user_structure() {
    // 测试 GitHub 用户结构
    let user = GitHubUser {
        login: "testuser".to_string(),
        name: Some("Test User".to_string()),
        email: Some("test@example.com".to_string()),
    };

    assert_eq!(user.login, "testuser");
    assert_eq!(user.name, Some("Test User".to_string()));
    assert_eq!(user.email, Some("test@example.com".to_string()));
}

#[test]
fn test_github_user_minimal() {
    // 测试 GitHub 用户结构（最小字段）
    let user = GitHubUser {
        login: "testuser".to_string(),
        name: None,
        email: None,
    };

    assert_eq!(user.login, "testuser");
    assert_eq!(user.name, None);
    assert_eq!(user.email, None);
}

#[test]
fn test_github_user_deserialization() -> Result<()> {
    // 测试 GitHub 用户的反序列化
    let json = r#"{
        "login": "testuser",
        "name": "Test User",
        "email": "test@example.com"
    }"#;

    let user: GitHubUser = serde_json::from_str(json)?;
    assert_eq!(user.login, "testuser");
    assert_eq!(user.name, Some("Test User".to_string()));
    Ok(())
}

// ==================== 序列化/反序列化边界测试 ====================

#[rstest]
#[case("", "", "", "")]
#[case("a", "b", "c", "d")]
fn test_request_edge_cases(
    #[case] title: &str,
    #[case] body: &str,
    #[case] head: &str,
    #[case] base: &str,
) {
    let request = CreatePullRequestRequest {
        title: title.to_string(),
        body: body.to_string(),
        head: head.to_string(),
        base: base.to_string(),
    };

    let json = serde_json::to_string(&request);
    assert!(json.is_ok(), "Should handle edge cases");
}

#[test]
fn test_request_long_strings() {
    // 测试长字符串的处理
    let long_string = "a".repeat(1000);
    let request = CreatePullRequestRequest {
        title: long_string.clone(),
        body: long_string.clone(),
        head: "feature/test".to_string(),
        base: "main".to_string(),
    };

    let json = serde_json::to_string(&request);
    assert!(json.is_ok(), "Should handle long strings");
}

#[test]
fn test_response_missing_optional_fields() -> Result<()> {
    // 测试响应中缺失可选字段
    let json = r#"{
        "number": 123,
        "title": "Test PR",
        "state": "open",
        "merged": false,
        "html_url": "https://github.com/owner/repo/pull/123",
        "head": {"ref": "feature/test"},
        "base": {"ref": "main"}
    }"#;

    let pr_info: PullRequestInfo = serde_json::from_str(json)?;
    assert_eq!(pr_info.body, None);
    assert_eq!(pr_info.merged_at, None);
    assert!(pr_info.user.is_none(), "User should be None");
    Ok(())
}

// ==================== 类型安全测试 ====================

#[test]
fn test_request_type_safety() {
    // 测试请求类型的安全性
    let create_request: CreatePullRequestRequest = CreatePullRequestRequest {
        title: "Test".to_string(),
        body: "Test".to_string(),
        head: "feature/test".to_string(),
        base: "main".to_string(),
    };

    let merge_request: MergePullRequestRequest = MergePullRequestRequest {
        commit_title: None,
        commit_message: None,
        merge_method: "squash".to_string(),
    };

    // 验证类型正确（通过编译验证）
    assert!(true, "Types should be type-safe");

    // 验证可以分别序列化
    assert!(serde_json::to_string(&create_request).is_ok());
    assert!(serde_json::to_string(&merge_request).is_ok());
}

#[test]
fn test_response_type_safety() {
    // 测试响应类型的安全性
    let _create_response: CreatePullRequestResponse = CreatePullRequestResponse {
        html_url: "https://example.com".to_string(),
    };

    let _pr_info: PullRequestInfo = PullRequestInfo {
        number: 1,
        title: "Test".to_string(),
        body: None,
        state: "open".to_string(),
        merged: false,
        merged_at: None,
        html_url: "https://example.com".to_string(),
        head: PullRequestBranch {
            ref_name: "head".to_string(),
        },
        base: PullRequestBranch {
            ref_name: "base".to_string(),
        },
        user: None,
    };

    // 验证类型正确（通过编译验证）
    assert!(true, "Types should be type-safe");
}
