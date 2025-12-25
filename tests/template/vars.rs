//! TemplateVars 业务逻辑测试
//!
//! 测试 TemplateVars 模块的核心功能，包括：
//! - 分支模板变量创建和序列化
//! - 提交模板变量创建和序列化
//! - PR 模板变量创建和序列化
//! - 变更类型项处理
//! - 特殊字符处理
//!
//! ## 测试策略
//!
//! - 涉及序列化/反序列化的测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试变量的创建、克隆和调试输出功能
//! - 测试特殊字符处理和完整性验证

use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::template::{
    BranchTemplateVars, ChangeTypeItem, CommitTemplateVars, PullRequestTemplateVars,
};

// ==================== BranchTemplateVars Tests ====================

/// 测试使用所有字段创建分支模板变量
///
/// ## 测试目的
/// 验证`BranchTemplateVars`结构体可以使用所有字段值（Jira键、Jira摘要、摘要slug、Jira类型）正确创建。
///
/// ## 测试场景
/// 1. 准备分支模板变量字段值：Jira键、Jira摘要、摘要slug、Jira类型
/// 2. 使用这些字段值创建`BranchTemplateVars`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `BranchTemplateVars`实例创建成功
/// - `jira_key`、`jira_summary`、`summary_slug`、`jira_type`字段值与提供的值一致
#[test]
fn test_branch_template_vars_creation_with_all_fields_creates_vars() {
    // Arrange: 准备分支模板变量字段值
    let jira_key = Some("PROJ-123".to_string());
    let jira_summary = Some("Implement user authentication".to_string());
    let summary_slug = Some("implement-user-authentication".to_string());
    let jira_type = Some("Feature".to_string());

    // Act: 创建 BranchTemplateVars 实例
    let vars = BranchTemplateVars {
        jira_key: jira_key.clone(),
        jira_summary: jira_summary.clone(),
        summary_slug: summary_slug.clone(),
        jira_type: jira_type.clone(),
    };

    // Assert: 验证所有字段值正确
    assert_eq!(vars.jira_key, jira_key);
    assert_eq!(vars.jira_summary, jira_summary);
    assert_eq!(vars.summary_slug, summary_slug);
    assert_eq!(vars.jira_type, jira_type);
}

/// 测试使用部分字段（部分为None）创建分支模板变量
///
/// ## 测试目的
/// 验证`BranchTemplateVars`结构体可以创建仅包含部分字段的配置，可选字段（Jira摘要、Jira类型）可以为`None`。
///
/// ## 测试场景
/// 1. 准备分支模板变量字段值：Jira键和摘要slug为`Some`，Jira摘要和Jira类型为`None`
/// 2. 使用这些字段值创建`BranchTemplateVars`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `BranchTemplateVars`实例创建成功
/// - `jira_key`和`summary_slug`字段为`Some`值
/// - `jira_summary`和`jira_type`字段为`None`
#[test]
fn test_branch_template_vars_partial_with_some_none_fields_creates_vars() {
    // Arrange: 准备分支模板变量字段值（部分为 None）
    let jira_key = Some("TASK-456".to_string());
    let summary_slug = Some("fix-login-bug".to_string());

    // Act: 创建 BranchTemplateVars 实例（部分字段为 None）
    let vars = BranchTemplateVars {
        jira_key: jira_key.clone(),
        jira_summary: None,
        summary_slug: summary_slug.clone(),
        jira_type: None,
    };

    // Assert: 验证字段值正确
    assert_eq!(vars.jira_key, jira_key);
    assert_eq!(vars.jira_summary, None);
    assert_eq!(vars.summary_slug, summary_slug);
    assert_eq!(vars.jira_type, None);
}

// ==================== CommitTemplateVars Tests ====================

/// 测试使用所有字段创建提交模板变量
///
/// ## 测试目的
/// 验证`CommitTemplateVars`结构体可以使用所有字段值（提交类型、作用域、主题、正文、Jira键、使用作用域标志）正确创建。
///
/// ## 测试场景
/// 1. 准备提交模板变量字段值：提交类型、作用域、主题、正文、Jira键、使用作用域标志
/// 2. 使用这些字段值创建`CommitTemplateVars`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `CommitTemplateVars`实例创建成功
/// - `commit_type`、`scope`、`subject`、`body`、`jira_key`、`use_scope`字段值与提供的值一致
#[test]
fn test_commit_template_vars_creation_with_all_fields_creates_vars() {
    // Arrange: 准备提交模板变量字段值
    let commit_type = "feat";
    let scope = Some("auth".to_string());
    let subject = "implement user authentication system";
    let body = Some("Add comprehensive authentication with JWT tokens and session management.".to_string());
    let jira_key = Some("AUTH-789".to_string());
    let use_scope = true;

    // Act: 创建 CommitTemplateVars 实例
    let vars = CommitTemplateVars {
        commit_type: commit_type.to_string(),
        scope: scope.clone(),
        subject: subject.to_string(),
        body: body.clone(),
        jira_key: jira_key.clone(),
        use_scope,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(vars.commit_type, commit_type);
    assert_eq!(vars.scope, scope);
    assert_eq!(vars.subject, subject);
    assert_eq!(vars.body, body);
    assert_eq!(vars.jira_key, jira_key);
    assert_eq!(vars.use_scope, use_scope);
}

/// 测试使用最小字段创建提交模板变量
///
/// ## 测试目的
/// 验证`CommitTemplateVars`结构体可以创建仅包含必需字段（提交类型、主题）的配置，可选字段（作用域、正文、Jira键）可以为`None`，`use_scope`为`false`。
///
/// ## 测试场景
/// 1. 准备最小配置的提交模板变量字段值：提交类型和主题，其他字段为`None`或`false`
/// 2. 使用这些字段值创建`CommitTemplateVars`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `CommitTemplateVars`实例创建成功
/// - `commit_type`和`subject`字段为提供的值
/// - `scope`、`body`、`jira_key`字段为`None`
/// - `use_scope`字段为`false`
#[test]
fn test_commit_template_vars_minimal_with_minimal_fields_creates_vars() {
    // Arrange: 准备最小配置的提交模板变量字段值
    let commit_type = "fix";
    let subject = "resolve login issue";

    // Act: 创建 CommitTemplateVars 实例（可选字段为 None）
    let vars = CommitTemplateVars {
        commit_type: commit_type.to_string(),
        scope: None,
        subject: subject.to_string(),
        body: None,
        jira_key: None,
        use_scope: false,
    };

    // Assert: 验证字段值正确
    assert_eq!(vars.commit_type, commit_type);
    assert_eq!(vars.scope, None);
    assert_eq!(vars.subject, subject);
    assert_eq!(vars.body, None);
    assert_eq!(vars.jira_key, None);
    assert_eq!(vars.use_scope, false);
}

// ==================== PullRequestTemplateVars Tests ====================

/// 测试使用所有字段创建PR模板变量
///
/// ## 测试目的
/// 验证`PullRequestTemplateVars`结构体可以使用所有字段值（Jira键、Jira摘要、Jira描述、Jira类型、Jira服务地址、变更类型列表、简短描述、依赖）正确创建。
///
/// ## 测试场景
/// 1. 准备PR模板变量字段值：包含3个变更类型项的列表、Jira相关信息、简短描述、依赖
/// 2. 使用这些字段值创建`PullRequestTemplateVars`实例
/// 3. 验证所有字段值正确设置，包括变更类型列表的长度和内容
///
/// ## 预期结果
/// - `PullRequestTemplateVars`实例创建成功
/// - `jira_key`、`jira_summary`、`jira_type`字段值与提供的值一致
/// - `change_types`包含3个变更类型项，第一个和第三个被选中，第二个未选中
#[test]
fn test_pr_template_vars_creation_with_all_fields_creates_vars() {
    // Arrange: 准备 PR 模板变量字段值
    let change_types = vec![
        ChangeTypeItem {
            name: "Bug fix".to_string(),
            selected: true,
        },
        ChangeTypeItem {
            name: "New feature".to_string(),
            selected: false,
        },
        ChangeTypeItem {
            name: "Documentation".to_string(),
            selected: true,
        },
    ];
    let jira_key = Some("PR-101".to_string());
    let jira_summary = Some("Fix authentication bug".to_string());
    let jira_type = Some("Bug".to_string());

    // Act: 创建 PullRequestTemplateVars 实例
    let vars = PullRequestTemplateVars {
        jira_key: jira_key.clone(),
        jira_summary: jira_summary.clone(),
        jira_description: Some("Detailed description of the authentication issue.".to_string()),
        jira_type: jira_type.clone(),
        jira_service_address: Some("https://company.atlassian.net".to_string()),
        change_types: change_types.clone(),
        short_description: Some(
            "This PR fixes the authentication bug that was preventing users from logging in."
                .to_string(),
        ),
        dependency: Some("Requires backend API v2.1+".to_string()),
    };

    // Assert: 验证所有字段值正确
    assert_eq!(vars.jira_key, jira_key);
    assert_eq!(vars.jira_summary, jira_summary);
    assert_eq!(vars.jira_type, jira_type);
    assert_eq!(vars.change_types.len(), 3);
    assert_eq!(vars.change_types[0].name, "Bug fix");
    assert_eq!(vars.change_types[0].selected, true);
    assert_eq!(vars.change_types[1].selected, false);
}

/// 测试创建默认PR模板变量
///
/// ## 测试目的
/// 验证`PullRequestTemplateVars::default()`能够创建所有字段为`None`或空的默认PR模板变量实例。
///
/// ## 测试场景
/// 1. 调用`PullRequestTemplateVars::default()`创建默认实例
/// 2. 验证所有可选字段为`None`
/// 3. 验证`change_types`列表为空
///
/// ## 预期结果
/// - `PullRequestTemplateVars`实例创建成功
/// - 所有可选字段（`jira_key`、`jira_summary`、`jira_description`、`jira_type`、`jira_service_address`、`short_description`、`dependency`）为`None`
/// - `change_types`列表为空
#[test]
fn test_pr_template_vars_default_with_no_parameters_creates_empty_vars() {
    // Arrange: 准备创建默认变量

    // Act: 创建默认的 PullRequestTemplateVars
    let vars = PullRequestTemplateVars::default();

    // Assert: 验证所有字段为 None 或空
    assert_eq!(vars.jira_key, None);
    assert_eq!(vars.jira_summary, None);
    assert_eq!(vars.jira_description, None);
    assert_eq!(vars.jira_type, None);
    assert_eq!(vars.jira_service_address, None);
    assert!(vars.change_types.is_empty());
    assert_eq!(vars.short_description, None);
    assert_eq!(vars.dependency, None);
}

// ==================== ChangeTypeItem Tests ====================

/// 测试使用有效字段创建变更类型项
///
/// ## 测试目的
/// 验证`ChangeTypeItem`结构体可以使用有效的字段值（名称、选中状态）正确创建，并验证`Clone`和`Debug`实现。
///
/// ## 测试场景
/// 1. 准备变更类型项字段值：名称为`"Breaking change"`，选中状态为`true`
/// 2. 使用这些字段值创建`ChangeTypeItem`实例
/// 3. 验证字段值正确设置
/// 4. 测试`Clone`实现，验证克隆后的字段值与原始值相同
/// 5. 测试`Debug`实现，验证Debug字符串包含结构体名称和字段值
///
/// ## 预期结果
/// - `ChangeTypeItem`实例创建成功
/// - `name`和`selected`字段值与提供的值一致
/// - 克隆后的字段值与原始值相同
/// - Debug字符串包含`"ChangeTypeItem"`、名称和选中状态
#[test]
fn test_change_type_item_parsing_with_valid_fields_creates_item() {
    // Arrange: 准备变更类型项字段值
    let name = "Breaking change";
    let selected = true;

    // Act: 创建 ChangeTypeItem 实例
    let change_type = ChangeTypeItem {
        name: name.to_string(),
        selected,
    };

    // Assert: 验证字段值正确
    assert_eq!(change_type.name, name);
    assert_eq!(change_type.selected, selected);

    // Arrange: 准备测试克隆和调试输出
    let cloned = change_type.clone();
    assert_eq!(cloned.name, change_type.name);
    assert_eq!(cloned.selected, change_type.selected);
    let debug_str = format!("{:?}", change_type);
    assert!(debug_str.contains("ChangeTypeItem"));
    assert!(debug_str.contains("Breaking change"));
    assert!(debug_str.contains("true"));
}

/// 测试变量序列化
#[test]
fn test_vars_serialization() -> Result<()> {
    // Arrange: 准备测试分支模板变量序列化
    let branch_vars = BranchTemplateVars {
        jira_key: Some("SER-123".to_string()),
        jira_summary: Some("Test serialization".to_string()),
        summary_slug: Some("test-serialization".to_string()),
        jira_type: Some("Task".to_string()),
    };

    let branch_json_str = serde_json::to_string(&branch_vars)?;
    assert!(branch_json_str.contains("SER-123"));
    assert!(branch_json_str.contains("test-serialization"));

    // Arrange: 准备测试提交模板变量序列化
    let commit_vars = CommitTemplateVars {
        commit_type: "test".to_string(),
        scope: Some("serialization".to_string()),
        subject: "test variable serialization".to_string(),
        body: Some("Testing serialization functionality".to_string()),
        jira_key: Some("SER-456".to_string()),
        use_scope: true,
    };

    let commit_json_str = serde_json::to_string(&commit_vars)?;
    assert!(commit_json_str.contains("test"));
    assert!(commit_json_str.contains("serialization"));
    assert!(commit_json_str.contains("SER-456"));

    // Arrange: 准备测试 PR 模板变量序列化
    let pr_vars = PullRequestTemplateVars {
        jira_key: Some("PR-789".to_string()),
        change_types: vec![ChangeTypeItem {
            name: "Test change".to_string(),
            selected: true,
        }],
        ..Default::default()
    };

    let pr_json_str = serde_json::to_string(&pr_vars)?;
    assert!(pr_json_str.contains("PR-789"));
    assert!(pr_json_str.contains("Test change"));
    Ok(())
}

/// 测试特殊字符处理
#[test]
fn test_vars_with_special_characters() -> Result<()> {
    let branch_vars = BranchTemplateVars {
        jira_key: Some("SPECIAL-123".to_string()),
        jira_summary: Some("Fix \"authentication\" & <security> issues".to_string()),
        summary_slug: Some("fix-authentication-security-issues".to_string()),
        jira_type: Some("Bug/Fix".to_string()),
    };

    // Arrange: 准备测试序列化能正确处理特殊字符
    let json_str = serde_json::to_string(&branch_vars)?;
    assert!(json_str.contains("SPECIAL-123"));
    assert!(json_str.contains("authentication"));
    assert!(json_str.contains("security"));
    assert!(json_str.contains("Bug/Fix"));

    // Assert: 验证特殊字符被正确转义
    assert!(json_str.contains("\\\"")); // 引号被转义
    assert!(json_str.contains("&")); // & 符号保持原样
    Ok(())
}

/// 测试变量完整性
#[test]
fn test_vars_completeness() -> Result<()> {
    // Arrange: 准备测试所有字段都存在的完整变量
    let complete_branch_vars = BranchTemplateVars {
        jira_key: Some("COMPLETE-001".to_string()),
        jira_summary: Some("Complete test case".to_string()),
        summary_slug: Some("complete-test-case".to_string()),
        jira_type: Some("Story".to_string()),
    };

    let complete_commit_vars = CommitTemplateVars {
        commit_type: "feat".to_string(),
        scope: Some("complete".to_string()),
        subject: "add complete functionality".to_string(),
        body: Some("This commit adds complete functionality with all features.".to_string()),
        jira_key: Some("COMPLETE-002".to_string()),
        use_scope: true,
    };

    let complete_pr_vars = PullRequestTemplateVars {
        jira_key: Some("COMPLETE-003".to_string()),
        jira_summary: Some("Complete PR".to_string()),
        jira_description: Some("Complete PR description".to_string()),
        jira_type: Some("Epic".to_string()),
        jira_service_address: Some("https://test.atlassian.net".to_string()),
        change_types: vec![
            ChangeTypeItem {
                name: "Feature".to_string(),
                selected: true,
            },
            ChangeTypeItem {
                name: "Bug fix".to_string(),
                selected: false,
            },
            ChangeTypeItem {
                name: "Documentation".to_string(),
                selected: true,
            },
        ],
        short_description: Some("Complete short description".to_string()),
        dependency: Some("No dependencies".to_string()),
    };

    // Assert: 验证所有字段都能正确序列化
    let branch_json = serde_json::to_value(&complete_branch_vars)?;
    assert!(branch_json["jira_key"].is_string());
    assert!(branch_json["jira_summary"].is_string());
    assert!(branch_json["summary_slug"].is_string());
    assert!(branch_json["jira_type"].is_string());

    let commit_json = serde_json::to_value(&complete_commit_vars)?;
    assert!(commit_json["commit_type"].is_string());
    assert!(commit_json["scope"].is_string());
    assert!(commit_json["subject"].is_string());
    assert!(commit_json["body"].is_string());
    assert!(commit_json["jira_key"].is_string());
    assert!(commit_json["use_scope"].is_boolean());

    let pr_json = serde_json::to_value(&complete_pr_vars)?;
    assert!(pr_json["jira_key"].is_string());
    assert!(pr_json["change_types"].is_array());
    if let Some(arr) = pr_json["change_types"].as_array() {
        assert_eq!(arr.len(), 3);
    }
    Ok(())
}

/// 测试变量克隆功能
#[test]
fn test_vars_clone() {
    let original_branch_vars = BranchTemplateVars {
        jira_key: Some("CLONE-123".to_string()),
        jira_summary: Some("Test cloning".to_string()),
        summary_slug: Some("test-cloning".to_string()),
        jira_type: Some("Test".to_string()),
    };

    let cloned_branch_vars = original_branch_vars.clone();
    assert_eq!(original_branch_vars.jira_key, cloned_branch_vars.jira_key);
    assert_eq!(
        original_branch_vars.jira_summary,
        cloned_branch_vars.jira_summary
    );
    assert_eq!(
        original_branch_vars.summary_slug,
        cloned_branch_vars.summary_slug
    );
    assert_eq!(original_branch_vars.jira_type, cloned_branch_vars.jira_type);

    let original_commit_vars = CommitTemplateVars {
        commit_type: "clone".to_string(),
        scope: Some("test".to_string()),
        subject: "test cloning functionality".to_string(),
        body: None,
        jira_key: Some("CLONE-456".to_string()),
        use_scope: false,
    };

    let cloned_commit_vars = original_commit_vars.clone();
    assert_eq!(
        original_commit_vars.commit_type,
        cloned_commit_vars.commit_type
    );
    assert_eq!(original_commit_vars.scope, cloned_commit_vars.scope);
    assert_eq!(original_commit_vars.subject, cloned_commit_vars.subject);
    assert_eq!(original_commit_vars.jira_key, cloned_commit_vars.jira_key);
    assert_eq!(original_commit_vars.use_scope, cloned_commit_vars.use_scope);
}

/// 测试变量调试输出
#[test]
fn test_vars_debug() {
    let branch_vars = BranchTemplateVars {
        jira_key: Some("DEBUG-001".to_string()),
        jira_summary: Some("Debug test".to_string()),
        summary_slug: Some("debug-test".to_string()),
        jira_type: Some("Debug".to_string()),
    };

    let debug_str = format!("{:?}", branch_vars);
    assert!(debug_str.contains("BranchTemplateVars"));
    assert!(debug_str.contains("DEBUG-001"));
    assert!(debug_str.contains("Debug test"));

    let commit_vars = CommitTemplateVars {
        commit_type: "debug".to_string(),
        scope: None,
        subject: "debug commit vars".to_string(),
        body: None,
        jira_key: None,
        use_scope: true,
    };

    let commit_debug_str = format!("{:?}", commit_vars);
    assert!(commit_debug_str.contains("CommitTemplateVars"));
    assert!(commit_debug_str.contains("debug"));
    assert!(commit_debug_str.contains("debug commit vars"));

    let change_type = ChangeTypeItem {
        name: "Debug change".to_string(),
        selected: false,
    };

    let change_type_debug = format!("{:?}", change_type);
    assert!(change_type_debug.contains("ChangeTypeItem"));
    assert!(change_type_debug.contains("Debug change"));
    assert!(change_type_debug.contains("false"));
}
