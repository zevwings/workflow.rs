//! Pull Request 模板渲染工具

use domain::{
    get_all_change_types, ChangeTypeItem, JiraIssue, PrTitleTemplateVars, PullRequestError,
    PullRequestTemplateVars,
};
use prompt::input;
use toolkit::TemplateEngine;

use crate::bootstrap::{get_global_config_repository, get_repo_config_repository};

/// 交互式获取 JIRA ID（必填）
///
/// 如果提供了 JIRA ID，直接返回；否则提示用户输入。
///
/// # 参数
///
/// * `jira_id` - 可选的 JIRA ID
///
/// # 返回
///
/// 返回验证后的 JIRA ID 字符串
pub fn get_pull_request_id_interactive(
    pull_request_id: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(id) = pull_request_id {
        // 验证格式
        validate_pull_request_id(&id)
            .map_err(|e| format!("Invalid Pull Request ID format: {}", e))?;
        Ok(id)
    } else {
        // 交互式输入
        let id = input!("Please enter your Pull Request ID (e.g.: 123):")
            .validator(|input: &str| {
                validate_pull_request_id(input)
                    .map_err(|e| format!("Invalid Pull Request ID format: {}", e))
            })
            .prompt()
            .map_err(|e| format!("Failed to get Pull Request ID: {}", e))?;
        Ok(id)
    }
}

/// 验证 Jira ticket 格式
///
/// Jira ticket 应该是 PROJECT-123 格式（ticket），或纯项目名（PROJECT）。
/// 项目名只能包含字母、数字和下划线。
///
/// # 示例
/// ```
/// use domain::validate_jira_ticket_format;
/// assert!(validate_jira_ticket_format("PROJ-123").is_ok());
/// assert!(validate_jira_ticket_format("PROJ").is_ok());
/// assert!(validate_jira_ticket_format("PROJ-123-456").is_ok());
/// assert!(validate_jira_ticket_format("invalid/ticket").is_err());
/// ```
pub fn validate_pull_request_id(pull_request_id: &str) -> Result<(), PullRequestError> {
    if pull_request_id.trim().is_empty() {
        return Err(PullRequestError::InvalidPullRequestId(
            pull_request_id.to_string(),
        ));
    }

    let pr_number: u64 = pull_request_id
        .parse()
        .map_err(|_| PullRequestError::InvalidPullRequestId(pull_request_id.to_string()))?;

    if pr_number == 0 {
        return Err(PullRequestError::InvalidPullRequestId(
            pull_request_id.to_string(),
        ));
    }

    Ok(())
}

/// 使用配置的 PR 标题模板渲染标题
///
/// 变量：`jira_key`（可选）、`commit_type`、`scope`（可选）、`summary`。
/// 若未配置标题模板或渲染失败，返回 `None`，调用方应回退到 `format_pr_title`。
pub fn generate_pull_request_title(
    type_: &str,
    scope: Option<&str>,
    jira_id: Option<&str>,
    commit_message: &str,
) -> Option<String> {
    let config_repo = get_repo_config_repository();
    let config = config_repo.load().ok()?;
    let template_str = config.project.template.pull_requests.title.trim();
    if template_str.is_empty() {
        return None;
    }

    let jira_key = jira_id.and_then(|j| {
        let t = j.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    });
    let summary = match &jira_key {
        Some(j) => commit_message
            .strip_prefix(&format!("{}: ", j))
            .unwrap_or(commit_message)
            .trim()
            .to_string(),
        None => commit_message.trim().to_string(),
    };
    let vars = PrTitleTemplateVars {
        jira_key,
        commit_type: type_.trim().to_string(),
        scope: scope.map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from),
        summary,
    };
    let engine = TemplateEngine::new();
    engine.render_string(template_str, &vars).ok()
}

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
    let template_str = config.project.template.pull_requests.body.clone();

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
