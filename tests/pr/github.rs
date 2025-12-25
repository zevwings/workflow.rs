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

// ==================== Request Structure Tests ====================

#[rstest]
fn test_create_request_structure_with_valid_fields_creates_request(
    sample_create_request: CreatePullRequestRequest,
) {
    // Arrange: 使用 fixture 提供的请求

    // Act: 验证请求结构
    // (结构验证在 Assert 中完成)

    // Assert: 验证所有字段值正确
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
fn test_create_pr_request_serialization_with_various_inputs_serializes_correctly(
    #[case] title: &str,
    #[case] body: &str,
    #[case] head: &str,
    #[case] base: &str,
) -> Result<()> {
    // Arrange: 准备 CreatePullRequestRequest 实例
    let request = CreatePullRequestRequest {
        title: title.to_string(),
        body: body.to_string(),
        head: head.to_string(),
        base: base.to_string(),
    };

    // Act: 序列化为 JSON
    let json_str = serde_json::to_string(&request)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
    let obj = json_value.as_object().expect("Should be a JSON object");

    // Assert: 验证 JSON 字段存在且值正确
    assert_eq!(obj.get("title").and_then(|v| v.as_str()), Some(title));
    assert_eq!(obj.get("body").and_then(|v| v.as_str()), Some(body));
    assert_eq!(obj.get("head").and_then(|v| v.as_str()), Some(head));
    assert_eq!(obj.get("base").and_then(|v| v.as_str()), Some(base));
    Ok(())
}

#[rstest]
fn test_merge_request_structure_with_valid_fields_creates_request(
    sample_merge_request: MergePullRequestRequest,
) {
    // Arrange: 使用 fixture 提供的请求

    // Act: 验证请求结构
    // (结构验证在 Assert 中完成)

    // Assert: 验证所有字段值正确
    assert_eq!(sample_merge_request.commit_title, None);
    assert_eq!(sample_merge_request.commit_message, None);
    assert_eq!(sample_merge_request.merge_method, "squash");
}

#[rstest]
#[case(None, None, "squash")]
#[case(Some("Merge PR #123"), Some("Merged via workflow"), "merge")]
#[case(Some("Custom Title"), None, "rebase")]
fn test_merge_pr_request_serialization_with_various_options_serializes_correctly(
    #[case] commit_title: Option<&str>,
    #[case] commit_message: Option<&str>,
    #[case] merge_method: &str,
) -> Result<()> {
    // Arrange: 准备 MergePullRequestRequest 实例
    let request = MergePullRequestRequest {
        commit_title: commit_title.map(|s| s.to_string()),
        commit_message: commit_message.map(|s| s.to_string()),
        merge_method: merge_method.to_string(),
    };

    // Act: 序列化为 JSON
    let json_str = serde_json::to_string(&request)?;

    // Assert: 验证 JSON 包含 merge_method 且 None 字段被跳过
    assert!(
        json_str.contains(merge_method),
        "JSON should contain merge_method"
    );
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
fn test_update_pr_request_serialization_with_various_options_serializes_correctly(
    #[case] title: Option<&str>,
    #[case] body: Option<&str>,
    #[case] state: Option<&str>,
    #[case] base: Option<&str>,
) -> Result<()> {
    // Arrange: 准备 UpdatePullRequestRequest 实例
    let request = UpdatePullRequestRequest {
        title: title.map(|s| s.to_string()),
        body: body.map(|s| s.to_string()),
        state: state.map(|s| s.to_string()),
        base: base.map(|s| s.to_string()),
    };

    // Act: 序列化为 JSON
    let json_str = serde_json::to_string(&request)?;

    // Assert: 验证存在的字段包含在 JSON 中，None 字段被跳过
    if let Some(t) = title {
        assert!(json_str.contains(t), "JSON should contain title");
    }
    if let Some(b) = body {
        assert!(json_str.contains(b), "JSON should contain body");
    }
    if let Some(s) = state {
        assert!(json_str.contains(s), "JSON should contain state");
    }
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

// ==================== Response Structure Tests ====================

#[test]
fn test_create_pull_request_response_structure_with_valid_fields_creates_response() {
    // Arrange: 准备响应字段值
    let html_url = "https://github.com/owner/repo/pull/123";

    // Act: 创建 CreatePullRequestResponse 实例
    let response = CreatePullRequestResponse {
        html_url: html_url.to_string(),
    };

    // Assert: 验证字段值正确
    assert_eq!(response.html_url, html_url);
}

#[test]
fn test_create_pull_request_response_deserialization_with_valid_json_deserializes_response() -> Result<()> {
    // Arrange: 准备有效的 JSON 字符串
    let json = r#"{"html_url": "https://github.com/owner/repo/pull/123"}"#;

    // Act: 反序列化为 CreatePullRequestResponse
    let response: CreatePullRequestResponse = serde_json::from_str(json)?;

    // Assert: 验证字段值正确
    assert_eq!(response.html_url, "https://github.com/owner/repo/pull/123");
    Ok(())
}

#[test]
fn test_pull_request_info_structure_with_valid_fields_creates_info() {
    // Arrange: 准备 PR 信息字段值
    let number = 123;
    let title = "Test PR";
    let body = Some("Test body".to_string());
    let state = "open";

    // Act: 创建 PullRequestInfo 实例
    let pr_info = PullRequestInfo {
        number,
        title: title.to_string(),
        body: body.clone(),
        state: state.to_string(),
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

    // Assert: 验证所有字段值正确
    assert_eq!(pr_info.number, number);
    assert_eq!(pr_info.title, title);
    assert_eq!(pr_info.body, body);
    assert_eq!(pr_info.state, state);
    assert!(!pr_info.merged);
    assert_eq!(pr_info.head.ref_name, "feature/test");
    assert_eq!(pr_info.base.ref_name, "main");
}

#[test]
fn test_pull_request_info_deserialization_with_valid_json_deserializes_info() -> Result<()> {
    // Arrange: 准备有效的 JSON 字符串
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

    // Act: 反序列化为 PullRequestInfo
    let pr_info: PullRequestInfo = serde_json::from_str(json)?;

    // Assert: 验证字段值正确
    assert_eq!(pr_info.number, 123);
    assert_eq!(pr_info.title, "Test PR");
    assert_eq!(pr_info.state, "open");
    Ok(())
}

#[test]
fn test_pull_request_info_merged_state_with_merged_pr_returns_merged() -> Result<()> {
    // Arrange: 准备已合并 PR 的 JSON
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

    // Act: 反序列化为 PullRequestInfo
    let pr_info: PullRequestInfo = serde_json::from_str(json)?;

    // Assert: 验证合并状态正确
    assert!(pr_info.merged, "Should be marked as merged");
    assert_eq!(pr_info.state, "closed");
    assert!(pr_info.merged_at.is_some());
    Ok(())
}

// ==================== PullRequestBranch Tests ====================

#[test]
fn test_pull_request_branch_structure_with_valid_ref_creates_branch() {
    // Arrange: 准备分支引用名
    let ref_name = "feature/test";

    // Act: 创建 PullRequestBranch 实例
    let branch = PullRequestBranch {
        ref_name: ref_name.to_string(),
    };

    // Assert: 验证字段值正确
    assert_eq!(branch.ref_name, ref_name);
}

#[test]
fn test_pull_request_branch_deserialization_with_valid_json_deserializes_branch() -> Result<()> {
    // Arrange: 准备有效的 JSON 字符串（注意 JSON 中使用 "ref" 字段）
    let json = r#"{"ref": "feature/test"}"#;

    // Act: 反序列化为 PullRequestBranch
    let branch: PullRequestBranch = serde_json::from_str(json)?;

    // Assert: 验证字段值正确
    assert_eq!(branch.ref_name, "feature/test");
    Ok(())
}

// ==================== GitHubUser Tests ====================

#[test]
fn test_github_user_structure_with_all_fields_creates_user() {
    // Arrange: 准备用户字段值
    let login = "testuser";
    let name = Some("Test User".to_string());
    let email = Some("test@example.com".to_string());

    // Act: 创建 GitHubUser 实例
    let user = GitHubUser {
        login: login.to_string(),
        name: name.clone(),
        email: email.clone(),
    };

    // Assert: 验证所有字段值正确
    assert_eq!(user.login, login);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
}

#[test]
fn test_github_user_minimal_with_only_login_creates_user() {
    // Arrange: 准备最小字段值（只有 login）
    let login = "testuser";

    // Act: 创建 GitHubUser 实例（可选字段为 None）
    let user = GitHubUser {
        login: login.to_string(),
        name: None,
        email: None,
    };

    // Assert: 验证字段值正确
    assert_eq!(user.login, login);
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
