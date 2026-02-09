//! Pull Request 模板渲染工具

use domain::{get_all_change_types, ChangeTypeItem, JiraIssue, PullRequestTemplateVars};
use toolkit::TemplateEngine;

use crate::registry::{get_global_config_repository, get_repo_config_repository};

/// 生成 PR body（使用模板系统）
///
/// 变更类型与 `domain::CHANGE_TYPES` 一致，与分支类型（BranchType）一一对应。
///
/// # Arguments
/// * `selected_change_types` - 选中的变更类型数组（与 `CHANGE_TYPES` 顺序一致，可由 `map_branch_type_to_change_types` 生成）
/// * `short_description` - 简短描述（可选）
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `dependency` - 依赖信息（可选）
/// * `jira_info` - Optional JIRA issue information (for template variables)
pub fn generate_pull_request_body(
    selected_change_types: &[bool],
    short_description: Option<&str>,
    jira_ticket: Option<&str>,
    dependency: Option<&str>,
    jira_info: Option<&JiraIssue>,
) -> Result<String, Box<dyn std::error::Error>> {
    let config_repo = get_repo_config_repository();
    let config = config_repo.load().map_err(|e| format!("Failed to load repo config: {}", e))?;
    let template_str = config.project.template.pull_requests.default.clone();

    // 使用 domain 的变更类型（与 BranchType 一致）
    let change_types: Vec<ChangeTypeItem> = get_all_change_types()
        .iter()
        .enumerate()
        .map(|(i, ct)| ChangeTypeItem {
            name: ct.name.to_string(),
            selected: i < selected_change_types.len() && selected_change_types[i],
        })
        .collect();

    // Get JIRA service address
    let global_config_repo = get_global_config_repository();
    let global_config = global_config_repo
        .load()
        .map_err(|e| format!("Failed to load global config: {}", e))?;
    let jira_service_address = Some(global_config.jira.service_address.clone());

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
    };

    // Render template
    let engine = TemplateEngine::new();
    engine
        .render_string(&template_str, &vars)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
