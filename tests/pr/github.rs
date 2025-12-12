//! GitHub PR 模块测试
//!
//! 测试 GitHub PR 平台的 API 集成、请求构建和响应解析。

use workflow::pr::github::{
    requests::{CreatePullRequestRequest, MergePullRequestRequest, UpdatePullRequestRequest},
    responses::{CreatePullRequestResponse, GitHubUser, PullRequestBranch, PullRequestInfo},
};

// ==================== 请求结构体测试 ====================

#[test]
fn test_create_pull_request_request_structure() {
    // 测试创建 PR 请求结构
    let request = CreatePullRequestRequest {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        head: "feature/test".to_string(),
        base: "main".to_string(),
    };

    assert_eq!(request.title, "Test PR");
    assert_eq!(request.body, "Test body");
    assert_eq!(request.head, "feature/test");
    assert_eq!(request.base, "main");
}

#[test]
fn test_create_pull_request_request_serialization() {
    // 测试创建 PR 请求的序列化
    let request = CreatePullRequestRequest {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        head: "feature/test".to_string(),
        base: "main".to_string(),
    };

    // 验证可以序列化为 JSON
    let json = serde_json::to_string(&request);
    assert!(json.is_ok(), "Should serialize to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("Test PR"), "JSON should contain title");
    assert!(json_str.contains("Test body"), "JSON should contain body");
    assert!(
        json_str.contains("feature/test"),
        "JSON should contain head"
    );
    assert!(json_str.contains("main"), "JSON should contain base");
}

#[test]
fn test_merge_pull_request_request_structure() {
    // 测试合并 PR 请求结构
    let request = MergePullRequestRequest {
        commit_title: None,
        commit_message: None,
        merge_method: "squash".to_string(),
    };

    assert_eq!(request.commit_title, None);
    assert_eq!(request.commit_message, None);
    assert_eq!(request.merge_method, "squash");
}

#[test]
fn test_merge_pull_request_request_with_optional_fields() {
    // 测试合并 PR 请求（带可选字段）
    let request = MergePullRequestRequest {
        commit_title: Some("Merge PR #123".to_string()),
        commit_message: Some("Merged via workflow".to_string()),
        merge_method: "merge".to_string(),
    };

    assert_eq!(request.commit_title, Some("Merge PR #123".to_string()));
    assert_eq!(
        request.commit_message,
        Some("Merged via workflow".to_string())
    );
    assert_eq!(request.merge_method, "merge");
}

#[test]
fn test_merge_pull_request_request_serialization() {
    // 测试合并 PR 请求的序列化
    let request = MergePullRequestRequest {
        commit_title: None,
        commit_message: None,
        merge_method: "squash".to_string(),
    };

    let json = serde_json::to_string(&request);
    assert!(json.is_ok(), "Should serialize to JSON");

    let json_str = json.unwrap();
    assert!(
        json_str.contains("squash"),
        "JSON should contain merge_method"
    );
    // None 字段应该被跳过（skip_serializing_if）
    assert!(
        !json_str.contains("commit_title"),
        "None fields should be skipped"
    );
    assert!(
        !json_str.contains("commit_message"),
        "None fields should be skipped"
    );
}

#[test]
fn test_update_pull_request_request_structure() {
    // 测试更新 PR 请求结构
    let request = UpdatePullRequestRequest {
        title: None,
        body: None,
        state: Some("closed".to_string()),
        base: None,
    };

    assert_eq!(request.state, Some("closed".to_string()));
}

#[test]
fn test_update_pull_request_request_serialization() {
    // 测试更新 PR 请求的序列化
    let request = UpdatePullRequestRequest {
        title: None,
        body: None,
        state: Some("closed".to_string()),
        base: None,
    };

    let json = serde_json::to_string(&request);
    assert!(json.is_ok(), "Should serialize to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("closed"), "JSON should contain state");
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
fn test_create_pull_request_response_deserialization() {
    // 测试创建 PR 响应的反序列化
    let json = r#"{"html_url": "https://github.com/owner/repo/pull/123"}"#;

    let response: Result<CreatePullRequestResponse, _> = serde_json::from_str(json);
    assert!(response.is_ok(), "Should deserialize from JSON");

    let response = response.unwrap();
    assert_eq!(response.html_url, "https://github.com/owner/repo/pull/123");
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
fn test_pull_request_info_deserialization() {
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

    let pr_info: Result<PullRequestInfo, _> = serde_json::from_str(json);
    assert!(pr_info.is_ok(), "Should deserialize from JSON");

    let pr_info = pr_info.unwrap();
    assert_eq!(pr_info.number, 123);
    assert_eq!(pr_info.title, "Test PR");
    assert_eq!(pr_info.state, "open");
}

#[test]
fn test_pull_request_info_merged_state() {
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

    let pr_info: Result<PullRequestInfo, _> = serde_json::from_str(json);
    assert!(pr_info.is_ok(), "Should deserialize merged PR");

    let pr_info = pr_info.unwrap();
    assert!(pr_info.merged, "Should be marked as merged");
    assert_eq!(pr_info.state, "closed");
    assert!(pr_info.merged_at.is_some());
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
fn test_pull_request_branch_deserialization() {
    // 测试 PR 分支的反序列化（注意 JSON 中使用 "ref" 字段）
    let json = r#"{"ref": "feature/test"}"#;

    let branch: Result<PullRequestBranch, _> = serde_json::from_str(json);
    assert!(branch.is_ok(), "Should deserialize from JSON");

    let branch = branch.unwrap();
    assert_eq!(branch.ref_name, "feature/test");
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
fn test_github_user_deserialization() {
    // 测试 GitHub 用户的反序列化
    let json = r#"{
        "login": "testuser",
        "name": "Test User",
        "email": "test@example.com"
    }"#;

    let user: Result<GitHubUser, _> = serde_json::from_str(json);
    assert!(user.is_ok(), "Should deserialize from JSON");

    let user = user.unwrap();
    assert_eq!(user.login, "testuser");
    assert_eq!(user.name, Some("Test User".to_string()));
}

// ==================== 序列化/反序列化边界测试 ====================

#[test]
fn test_request_empty_strings() {
    // 测试空字符串的处理
    let request = CreatePullRequestRequest {
        title: "".to_string(),
        body: "".to_string(),
        head: "".to_string(),
        base: "".to_string(),
    };

    let json = serde_json::to_string(&request);
    assert!(json.is_ok(), "Should handle empty strings");
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
fn test_response_missing_optional_fields() {
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

    let pr_info: Result<PullRequestInfo, _> = serde_json::from_str(json);
    assert!(pr_info.is_ok(), "Should handle missing optional fields");

    let pr_info = pr_info.unwrap();
    assert_eq!(pr_info.body, None);
    assert_eq!(pr_info.merged_at, None);
    assert!(pr_info.user.is_none(), "User should be None");
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
