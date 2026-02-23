use domain::{extract_jira_project, validate_jira_ticket_format, JiraRepository, JiraStatusConfig};
use prompt::{info, input, select, spinner, success};

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
pub fn get_jira_id_interactive(
    jira_id: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(id) = jira_id {
        // 验证格式
        validate_jira_ticket_format(&id).map_err(|e| format!("Invalid JIRA ID format: {}", e))?;
        Ok(id)
    } else {
        // 交互式输入
        let id = input!("Please enter your JIRA ticket ID (e.g.: PROJ-123):")
            .validator(|input: &str| {
                validate_jira_ticket_format(input)
                    .map_err(|e| format!("Invalid JIRA ID format: {}", e))
            })
            .prompt()
            .map_err(|e| format!("Failed to get JIRA ID: {}", e))?;
        Ok(id)
    }
}

/// 交互式获取 JIRA ID（可选）
///
/// 如果提供了 JIRA ID，直接返回；否则提示用户输入。
/// 用户可以跳过输入（按 Enter），返回 None。
///
/// # 参数
///
/// * `jira_id` - 可选的 JIRA ID
///
/// # 返回
///
/// 返回验证后的 JIRA ID 字符串，如果用户跳过则返回 None
pub fn get_jira_id_interactive_optional(
    jira_id: Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(id) = jira_id {
        // 验证格式
        validate_jira_ticket_format(&id).map_err(|e| format!("Invalid JIRA ID format: {}", e))?;
        Ok(Some(id))
    } else {
        // 交互式输入
        let id = input!("Please enter your JIRA ticket ID (optional, press Enter to skip):")
            .prompt()
            .ok();

        if let Some(id) = id {
            let trimmed = id.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                // 验证格式
                validate_jira_ticket_format(trimmed)
                    .map_err(|e| format!("Invalid JIRA ID format: {}", e))?;
                Ok(Some(trimmed.to_string()))
            }
        } else {
            Ok(None)
        }
    }
}

/// 确保 Jira 状态配置存在
///
/// 如果有 Jira ticket，检查并配置状态。如果已配置则读取，否则进行交互式配置。
///
/// # 参数
///
/// * `jira_repo` - Jira 仓储
/// * `jira_ticket` - 可选的 Jira ticket ID
///
/// # 返回
///
/// 返回配置的 PR 创建状态（如果有），否则返回 `None`。
pub fn ensure_jira_status_config(
    jira_repo: &dyn JiraRepository,
    jira_ticket: &Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(ref ticket) = jira_ticket else {
        return Ok(None);
    };

    // 读取状态配置
    if let Ok(Some(status)) = jira_repo.read_pull_request_created_status(ticket) {
        return Ok(Some(status));
    }

    // 如果没有配置，提示配置
    info!("Status config not found for {}, configuring...", ticket);

    let config_result = configure_jira_status_interactive(jira_repo, ticket)?;

    success!("Jira status configuration saved");
    info!("Project name: {}", config_result.project);
    info!(
        "PR status on creation: {}",
        config_result.created_pull_request_status
    );
    info!(
        "PR status on merge: {}",
        config_result.merged_pull_request_status
    );

    Ok(Some(config_result.created_pull_request_status))
}

/// Jira 状态配置结果
pub struct JiraStatusConfigResult {
    /// 项目名称
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: String,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: String,
}

/// 交互式配置 Jira 状态
///
/// 通过交互式界面配置指定项目的 PR 创建和合并时的目标状态。
fn configure_jira_status_interactive(
    jira_repo: &dyn JiraRepository,
    jira_ticket: &str,
) -> Result<JiraStatusConfigResult, Box<dyn std::error::Error>> {
    let project = extract_jira_project(jira_ticket).ok_or_else(|| {
        format!(
            "Invalid Jira ticket format: cannot extract project name from '{}'",
            jira_ticket
        )
    })?;

    // 验证项目名格式
    if !project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "Invalid Jira project name format: '{}'. Jira project names should contain only ASCII letters, numbers, and underscores.",
            project
        )
        .into());
    }

    // 从 Jira API 获取项目状态列表
    let statuses = spinner!("Fetching status list for project {}...", project)
        .with(|| jira_repo.get_project_statuses(project))
        .map_err(|e| format!(
            "Failed to fetch project statuses for '{}'. Please check:\n  - The project name is correct\n  - The project exists in your Jira instance\n  - You have access to this project\nError: {}",
            project, e
        ))?;

    if statuses.is_empty() {
        return Err(format!("No statuses found for project {}", project).into());
    }

    // 交互式选择 PR 创建时的状态
    let created_pull_request_status = select!("Select status for PR creation:", statuses.clone())
        .prompt()
        .map_err(|e| format!("Failed to select status: {}", e))?;

    // 交互式选择 PR 合并时的状态
    let merged_pull_request_status = select!("Select status for PR merge:", statuses)
        .prompt()
        .map_err(|e| format!("Failed to select status: {}", e))?;

    // 保存配置
    let jira_config = JiraStatusConfig {
        project: project.to_string(),
        created_pull_request_status: Some(created_pull_request_status.clone()),
        merged_pull_request_status: Some(merged_pull_request_status.clone()),
    };

    jira_repo
        .write_status_config(&jira_config)
        .map_err(|e| format!("Failed to write Jira status configuration: {}", e))?;

    Ok(JiraStatusConfigResult {
        project: project.to_string(),
        created_pull_request_status,
        merged_pull_request_status,
    })
}
