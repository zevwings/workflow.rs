//! Template 变量测试
//!
//! 测试 Template 变量相关的功能，包括：
//! - 变量生成
//! - 变量验证
//! - 变量序列化

use pretty_assertions::assert_eq;
use workflow::template::vars::{
    BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars,
};

// ==================== 分支模板变量测试 ====================

#[test]
fn test_branch_template_vars() {
    // 测试分支模板变量
    let vars = BranchTemplateVars {
        jira_key: Some("PROJ-123".to_string()),
        jira_summary: Some("Fix bug".to_string()),
        summary_slug: Some("fix-bug".to_string()),
        jira_type: Some("Bug".to_string()),
    };

    assert_eq!(vars.jira_key, Some("PROJ-123".to_string()));
    assert_eq!(vars.jira_summary, Some("Fix bug".to_string()));
    assert_eq!(vars.summary_slug, Some("fix-bug".to_string()));
    assert_eq!(vars.jira_type, Some("Bug".to_string()));
}

#[test]
fn test_branch_template_vars_with_none() {
    // 测试分支模板变量（部分为 None）
    let vars = BranchTemplateVars {
        jira_key: None,
        jira_summary: Some("Feature".to_string()),
        summary_slug: Some("feature".to_string()),
        jira_type: None,
    };

    assert_eq!(vars.jira_key, None);
    assert_eq!(vars.jira_summary, Some("Feature".to_string()));
}

#[test]
fn test_branch_template_vars_serialization() {
    // 测试分支模板变量序列化
    let vars = BranchTemplateVars {
        jira_key: Some("PROJ-123".to_string()),
        jira_summary: Some("Fix bug".to_string()),
        summary_slug: Some("fix-bug".to_string()),
        jira_type: Some("Bug".to_string()),
    };

    let json = serde_json::to_string(&vars).unwrap();
    assert!(json.contains("PROJ-123"));
    assert!(json.contains("fix-bug"));
}

// ==================== 提交模板变量测试 ====================

#[test]
fn test_commit_template_vars() {
    // 测试提交模板变量
    let vars = CommitTemplateVars {
        commit_type: "feat".to_string(),
        scope: Some("api".to_string()),
        subject: "Add new endpoint".to_string(),
        body: Some("This adds a new REST API endpoint".to_string()),
        jira_key: Some("PROJ-123".to_string()),
        use_scope: false,
    };

    assert_eq!(vars.commit_type, "feat");
    assert_eq!(vars.scope, Some("api".to_string()));
    assert_eq!(vars.subject, "Add new endpoint");
    assert_eq!(vars.jira_key, Some("PROJ-123".to_string()));
    assert!(!vars.use_scope);
}

#[test]
fn test_commit_template_vars_without_scope() {
    // 测试提交模板变量（无 scope）
    let vars = CommitTemplateVars {
        commit_type: "fix".to_string(),
        scope: None,
        subject: "Fix bug".to_string(),
        body: None,
        jira_key: None,
        use_scope: true,
    };

    assert_eq!(vars.scope, None);
    assert!(vars.use_scope);
}

#[test]
fn test_commit_template_vars_serialization() {
    // 测试提交模板变量序列化
    let vars = CommitTemplateVars {
        commit_type: "feat".to_string(),
        scope: Some("api".to_string()),
        subject: "Add endpoint".to_string(),
        body: None,
        jira_key: Some("PROJ-123".to_string()),
        use_scope: false,
    };

    let json = serde_json::to_string(&vars).unwrap();
    assert!(json.contains("feat"));
    assert!(json.contains("api"));
    assert!(json.contains("PROJ-123"));
}

// ==================== PR 模板变量测试 ====================

#[test]
fn test_pull_request_template_vars() {
    // 测试 PR 模板变量
    let vars = PullRequestTemplateVars {
        jira_key: Some("PROJ-123".to_string()),
        jira_summary: Some("Add feature".to_string()),
        jira_description: Some("This PR adds a new feature".to_string()),
        jira_type: Some("Feature".to_string()),
        jira_service_address: Some("https://jira.example.com".to_string()),
        change_types: vec![
            ChangeTypeItem {
                name: "Feature".to_string(),
                selected: true,
            },
            ChangeTypeItem {
                name: "Bug Fix".to_string(),
                selected: false,
            },
        ],
        short_description: Some("Short description".to_string()),
        dependency: None,
    };

    assert_eq!(vars.jira_key, Some("PROJ-123".to_string()));
    assert_eq!(vars.change_types.len(), 2);
    assert!(vars.change_types[0].selected);
    assert!(!vars.change_types[1].selected);
}

#[test]
fn test_pull_request_template_vars_minimal() {
    // 测试 PR 模板变量（最小配置）
    let vars = PullRequestTemplateVars {
        jira_key: None,
        jira_summary: None,
        jira_description: None,
        jira_type: None,
        jira_service_address: None,
        change_types: vec![],
        short_description: None,
        dependency: None,
    };

    assert_eq!(vars.jira_key, None);
    assert!(vars.change_types.is_empty());
}

#[test]
fn test_pull_request_template_vars_serialization() {
    // 测试 PR 模板变量序列化
    let vars = PullRequestTemplateVars {
        jira_key: Some("PROJ-123".to_string()),
        jira_summary: Some("Feature".to_string()),
        jira_description: None,
        jira_type: None,
        jira_service_address: None,
        change_types: vec![ChangeTypeItem {
            name: "Feature".to_string(),
            selected: true,
        }],
        short_description: None,
        dependency: None,
    };

    let json = serde_json::to_string(&vars).unwrap();
    assert!(json.contains("PROJ-123"));
    assert!(json.contains("Feature"));
}

// ==================== Change Type Item 测试 ====================

#[test]
fn test_change_type_item() {
    // 测试 Change Type Item
    let item = ChangeTypeItem {
        name: "Feature".to_string(),
        selected: true,
    };

    assert_eq!(item.name, "Feature");
    assert!(item.selected);
}

#[test]
fn test_change_type_item_serialization() {
    // 测试 Change Type Item 序列化
    let item = ChangeTypeItem {
        name: "Bug Fix".to_string(),
        selected: false,
    };

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("Bug Fix"));
    assert!(json.contains("false"));
}
