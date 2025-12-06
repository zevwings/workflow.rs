//! PR 辅助函数测试
//!
//! 测试 `pr::helpers` 模块中的 PR 相关辅助函数。

use workflow::pr::helpers::{
    detect_repo_type, extract_pull_request_id_from_url, generate_branch_name,
    generate_commit_title, generate_pull_request_body, transform_to_branch_name,
};

// ==================== extract_pull_request_id_from_url 测试 ====================

#[test]
fn test_extract_pull_request_id_from_url_github() {
    // 测试 GitHub URL
    assert_eq!(
        extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123").unwrap(),
        "123"
    );
    assert_eq!(
        extract_pull_request_id_from_url("https://github.com/owner/repo/pull/456/").unwrap(),
        "456"
    );
    assert_eq!(
        extract_pull_request_id_from_url("https://github.com/user/project/pull/789/files").unwrap(),
        "789"
    );
}

#[test]
fn test_extract_pull_request_id_from_url_codeup() {
    // 测试 Codeup URL
    assert_eq!(
        extract_pull_request_id_from_url("https://codeup.aliyun.com/owner/repo/pull/123").unwrap(),
        "123"
    );
    assert_eq!(
        extract_pull_request_id_from_url("https://codeup.aliyun.com/owner/repo/pull/456/").unwrap(),
        "456"
    );
}

#[test]
fn test_extract_pull_request_id_from_url_invalid() {
    // 测试无效的 URL
    assert!(extract_pull_request_id_from_url("https://github.com/owner/repo").is_err());
    assert!(extract_pull_request_id_from_url("not-a-url").is_err());
    assert!(extract_pull_request_id_from_url("").is_err());
}

// ==================== generate_branch_name 测试 ====================

#[test]
fn test_generate_branch_name_with_ticket() {
    // 测试带 ticket 的分支名生成
    let branch = generate_branch_name(Some("PROJ-123"), "Fix critical bug").unwrap();
    assert!(branch.contains("PROJ-123"));
    assert!(branch.contains("fix-critical-bug"));
}

#[test]
fn test_generate_branch_name_without_ticket() {
    // 测试不带 ticket 的分支名生成
    let branch = generate_branch_name(None, "Implement new feature").unwrap();
    assert!(!branch.contains("PROJ"));
    assert!(branch.contains("implement-new-feature"));
}

#[test]
fn test_generate_branch_name_special_characters() {
    // 测试特殊字符处理
    let branch = generate_branch_name(Some("PROJ-123"), "Fix bug: critical issue!").unwrap();
    // 特殊字符应该被替换或移除
    assert!(!branch.contains(':'));
    assert!(!branch.contains('!'));
}

#[test]
fn test_generate_branch_name_long_title() {
    // 测试长标题
    let long_title = "This is a very long title that should be truncated appropriately";
    let branch = generate_branch_name(Some("PROJ-123"), long_title).unwrap();
    // 分支名应该有合理的长度限制
    assert!(branch.len() < 200); // Git 分支名通常有长度限制
}

// ==================== generate_commit_title 测试 ====================

#[test]
fn test_generate_commit_title_with_ticket() {
    // 测试带 ticket 的提交标题
    let title = generate_commit_title(Some("PROJ-123"), "Fix bug");
    assert_eq!(title, "PROJ-123: Fix bug");
}

#[test]
fn test_generate_commit_title_without_ticket() {
    // 测试不带 ticket 的提交标题
    // 注意：实际实现会在标题前添加 "# "
    let title = generate_commit_title(None, "Fix bug");
    assert_eq!(title, "# Fix bug");
}

#[test]
fn test_generate_commit_title_empty() {
    // 测试空标题
    let title = generate_commit_title(Some("PROJ-123"), "");
    assert_eq!(title, "PROJ-123: ");
}

// ==================== transform_to_branch_name 测试 ====================

#[test]
fn test_transform_to_branch_name_basic() {
    // 测试基本转换
    assert_eq!(transform_to_branch_name("Hello World"), "hello-world");
    assert_eq!(transform_to_branch_name("Fix Bug"), "fix-bug");
}

#[test]
fn test_transform_to_branch_name_special_characters() {
    // 测试特殊字符处理
    assert_eq!(transform_to_branch_name("Fix: Bug!"), "fix-bug");
    assert_eq!(transform_to_branch_name("Test (Feature)"), "test-feature");
    assert_eq!(transform_to_branch_name("Code/Path"), "code-path");
}

#[test]
fn test_transform_to_branch_name_unicode() {
    // 测试 Unicode 字符（中文等）
    let result = transform_to_branch_name("测试功能");
    // Unicode 字符应该被处理（可能被移除或转换）
    assert!(!result.is_empty());
}

#[test]
fn test_transform_to_branch_name_multiple_spaces() {
    // 测试多个空格
    assert_eq!(
        transform_to_branch_name("Fix   Multiple    Spaces"),
        "fix-multiple-spaces"
    );
}

#[test]
fn test_transform_to_branch_name_leading_trailing() {
    // 测试首尾空格和特殊字符
    assert_eq!(transform_to_branch_name("  Fix Bug  "), "fix-bug");
    assert_eq!(transform_to_branch_name("-Fix-Bug-"), "fix-bug");
}

// ==================== generate_pull_request_body 测试 ====================

#[test]
fn test_generate_pull_request_body_basic() {
    // 测试基本 PR body 生成
    // 注意：generate_pull_request_body 的签名是 (selected_change_types, short_description, jira_ticket, dependency)
    let change_types = vec![false; 10]; // 假设有 10 种变更类型
    let body = generate_pull_request_body(
        &change_types,
        Some("Fix critical bug"),
        Some("PROJ-123"),
        None,
    )
    .unwrap();
    assert!(body.contains("PROJ-123"));
    assert!(body.contains("Fix critical bug"));
}

#[test]
fn test_generate_pull_request_body_without_ticket() {
    // 测试不带 ticket 的 PR body
    let change_types = vec![false; 10];
    let body =
        generate_pull_request_body(&change_types, Some("Implement feature"), None, None).unwrap();
    assert!(!body.contains("PROJ"));
    assert!(body.contains("Implement feature"));
}

#[test]
fn test_generate_pull_request_body_with_change_types() {
    // 测试带变更类型的 PR body
    // 前两个为 true，表示选中了前两种变更类型
    let mut change_types = vec![false; 10];
    change_types[0] = true; // Bug fix
    change_types[1] = true; // New feature
    let body =
        generate_pull_request_body(&change_types, Some("Description"), Some("PROJ-123"), None)
            .unwrap();
    assert!(body.contains("PROJ-123"));
    assert!(body.contains("Description"));
}

#[test]
fn test_generate_pull_request_body_with_dependency() {
    // 测试带依赖信息的 PR body
    let change_types = vec![false; 10];
    let body = generate_pull_request_body(
        &change_types,
        Some("Description"),
        Some("PROJ-123"),
        Some("Depends on PR #456"),
    )
    .unwrap();
    assert!(body.contains("PROJ-123"));
    assert!(body.contains("Depends on PR #456"));
}

// ==================== detect_repo_type 测试 ====================

#[test]
fn test_detect_repo_type_github() {
    // 测试 GitHub 仓库类型检测
    let result = detect_repo_type(
        |repo_type| {
            assert_eq!(repo_type, workflow::git::RepoType::GitHub);
            Ok(())
        },
        "Get repo type",
    );
    assert!(result.is_ok());
}

#[test]
fn test_detect_repo_type_codeup() {
    // 测试 Codeup 仓库类型检测
    let result = detect_repo_type(
        |repo_type| {
            assert_eq!(repo_type, workflow::git::RepoType::Codeup);
            Ok(())
        },
        "Get repo type",
    );
    assert!(result.is_ok());
}

#[test]
fn test_detect_repo_type_error() {
    // 测试错误情况
    let result: Result<(), _> = detect_repo_type(
        |_repo_type| Err(anyhow::anyhow!("Failed to detect")),
        "Get repo type",
    );
    assert!(result.is_err());
}

// ==================== 集成测试场景 ====================

#[test]
fn test_pr_workflow_complete() {
    // 测试完整的 PR 工作流
    let ticket = Some("PROJ-123");
    let title = "Fix critical bug in authentication";

    // 1. 生成分支名
    let branch = generate_branch_name(ticket, title).unwrap();
    assert!(branch.contains("PROJ-123"));

    // 2. 生成提交标题
    let commit_title = generate_commit_title(ticket, title);
    assert_eq!(commit_title, "PROJ-123: Fix critical bug in authentication");

    // 3. 生成 PR body
    let change_types = vec![false; 10];
    let body = generate_pull_request_body(&change_types, Some(title), ticket, None).unwrap();
    assert!(body.contains("PROJ-123"));
    assert!(body.contains(title));
}

#[test]
fn test_branch_name_consistency() {
    // 测试分支名生成的一致性
    let title = "Fix Bug";
    let branch1 = generate_branch_name(None, title).unwrap();
    let branch2 = generate_branch_name(None, title).unwrap();
    assert_eq!(branch1, branch2);
}

#[test]
fn test_url_extraction_round_trip() {
    // 测试 URL 提取的往返一致性
    let urls = vec![
        "https://github.com/owner/repo/pull/123",
        "https://codeup.aliyun.com/owner/repo/pull/456",
    ];

    for url in urls {
        let pr_id = extract_pull_request_id_from_url(url).unwrap();
        // 验证 PR ID 是数字
        assert!(pr_id.parse::<u32>().is_ok());
    }
}
