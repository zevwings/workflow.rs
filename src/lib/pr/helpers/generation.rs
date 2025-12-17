//! 内容生成相关辅助函数
//!
//! 提供生成 commit 标题和 PR body 的函数。

use crate::base::settings::Settings;
use crate::template::{
    ChangeTypeItem, CommitTemplateVars, CommitTemplates, PullRequestTemplateVars,
    PullRequestsTemplates, TemplateConfig, TemplateEngine,
};
use color_eyre::{eyre::WrapErr, Result};

use super::super::platform::TYPES_OF_CHANGES;

/// 生成 PR body（使用模板系统）
///
/// # Arguments
/// * `selected_change_types` - 选中的变更类型数组
/// * `short_description` - 简短描述（可选）
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `dependency` - 依赖信息（可选）
/// * `jira_info` - Optional JIRA issue information (for template variables)
pub fn generate_pull_request_body(
    selected_change_types: &[bool],
    short_description: Option<&str>,
    jira_ticket: Option<&str>,
    dependency: Option<&str>,
    jira_info: Option<&crate::jira::JiraIssue>,
) -> Result<String> {
    // Load PR template
    let template_str = TemplateConfig::load_pull_request_template().unwrap_or_else(|_| {
        // Fallback to default template if loading fails
        PullRequestsTemplates::default_pull_request_template()
    });

    // Prepare change types
    let change_types: Vec<ChangeTypeItem> = TYPES_OF_CHANGES
        .iter()
        .enumerate()
        .map(|(i, name)| ChangeTypeItem {
            name: name.to_string(),
            selected: i < selected_change_types.len() && selected_change_types[i],
        })
        .collect();

    // Get JIRA service address
    let jira_service_address = Settings::get().jira.service_address.clone();

    // Prepare template variables
    let vars = PullRequestTemplateVars {
        jira_key: jira_ticket.map(|s| s.to_string()),
        jira_summary: jira_info.as_ref().map(|i| i.fields.summary.clone()),
        jira_description: jira_info.as_ref().and_then(|i| i.fields.description.clone()),
        jira_type: None,
        jira_service_address,
        change_types,
        short_description: short_description.map(|s| s.to_string()),
        dependency: dependency.map(|s| s.to_string()),
        title: None, // 可以根据需要设置 PR 标题
        description: None, // 可以根据需要设置 PR 描述
        branch_name: None, // 可以根据需要设置分支名称
        author: None, // 可以根据需要设置作者
        files_changed: None, // 可以根据需要设置变更文件列表
        commit_count: None, // 可以根据需要设置提交数量
        timestamp: None, // 可以根据需要设置时间戳
    };

    // Render template
    let engine = TemplateEngine::new();
    engine
        .render_string(&template_str, &vars)
        .wrap_err("Failed to render PR body template")
}

/// 生成 commit 标题（使用模板系统）
///
/// # Arguments
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `title` - PR 标题
/// * `commit_type` - Optional commit type (e.g., "feat", "fix")
/// * `scope` - Optional commit scope
/// * `body` - Optional commit body
pub fn generate_commit_title(
    jira_ticket: Option<&str>,
    title: &str,
    commit_type: Option<&str>,
    scope: Option<&str>,
    body: Option<&str>,
) -> Result<String> {
    // Load template configuration
    let config = TemplateConfig::load().unwrap_or_default();

    // Load commit template
    let template_str = TemplateConfig::load_commit_template().unwrap_or_else(|_| {
        // Fallback to default template
        CommitTemplates::default_commit_template()
    });

    // Scope extraction is handled by LLM (PullRequestLLM::generate).
    // If LLM doesn't return a scope, we keep it as None and let the template system
    // handle it based on the `use_scope` configuration.
    // When `use_scope = true` and scope is None, the template will use `commit_type: title` format.
    let final_scope = scope.map(|s| s.to_string());

    // Prepare template variables
    let vars = CommitTemplateVars {
        commit_type: commit_type.unwrap_or("feat").to_string(),
        scope: final_scope,
        subject: title.to_string(),
        body: body.map(|s| s.to_string()),
        jira_key: jira_ticket.map(|s| s.to_string()),
        use_scope: config.commit.use_scope,
    };

    // Render template
    let engine = TemplateEngine::new();
    engine
        .render_string(&template_str, &vars)
        .wrap_err("Failed to render commit title template")
}
