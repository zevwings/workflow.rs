use domain::jira::validate_jira_ticket_format;
use domain::{extract_jira_project, JiraRepository, JiraStatusConfig, JiraWorkHistoryRepository};
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
        let id = input!("Please enter your JIRA ticket ID (e.g., PROJ-123):")
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
    info!(
        "No status configuration found for {}, configuring...",
        ticket
    );

    let config_result = configure_jira_status_interactive(jira_repo, ticket)?;

    success!("Jira status configuration saved");
    info!(
        "  PR created status: {}",
        config_result.created_pull_request_status
    );
    info!(
        "  PR merged status: {}",
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
            "Invalid Jira ticket format: cannot extract project from '{}'",
            jira_ticket
        )
    })?;

    // 验证项目名格式
    if !project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "Invalid Jira project name format: '{}'. Jira project names should contain only ASCII letters, numbers, and underscores.",
            project
        ).into());
    }

    // 从 Jira API 获取项目状态列表
    let statuses = spinner!("Fetching status list for project {}...", project)
        .with(|| jira_repo.get_project_statuses(project))
        .map_err(|e| format!(
            "Failed to fetch project statuses for '{}'. Please check:\n  - The project name is correct\n  - The project exists in your Jira instance\n  - You have access to this project\nError: {}",
            project, e
        ))?;

    if statuses.is_empty() {
        return Err(format!("No statuses found for project: {}", project).into());
    }

    // 交互式选择 PR 创建时的状态
    let created_pull_request_status = select!("Select status for PR created:", statuses.clone())
        .prompt()
        .map_err(|e| format!("Failed to select status: {}", e))?;

    // 交互式选择 PR 合并时的状态
    let merged_pull_request_status = select!("Select status for PR merged:", statuses)
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

/// PR 创建后更新 Jira ticket
///
/// 如果有 Jira ticket 和状态配置，更新 ticket：
/// - 更新状态到 "PR 创建" 状态
/// - 添加评论（PR URL）
/// - 写入工作历史记录
///
/// # 参数
///
/// * `jira_repo` - Jira 仓储
/// * `work_history_repo` - 工作历史记录仓储
/// * `jira_ticket` - 可选的 Jira ticket ID
/// * `created_status` - 可选的 PR 创建状态
/// * `pr_id` - PR ID
/// * `pr_url` - PR URL
/// * `repository_url` - 仓库 URL
/// * `branch_name` - 分支名称
pub fn update_jira_after_pr_created(
    jira_repo: &dyn JiraRepository,
    work_history_repo: &dyn JiraWorkHistoryRepository,
    jira_ticket: &Option<String>,
    created_status: &Option<String>,
    pr_id: &str,
    pr_url: &str,
    repository_url: &str,
    branch_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(ref ticket) = jira_ticket else {
        return Ok(());
    };

    let Some(ref status) = created_status else {
        return Ok(());
    };

    // 更新 Jira ticket
    spinner!("Updating Jira ticket {}...", ticket).with(
        || -> Result<(), Box<dyn std::error::Error>> {
            // 更新状态
            jira_repo
                .update_issue_status(ticket, status)
                .map_err(|e| format!("Failed to update issue status: {}", e))?;

            // 添加评论（PR URL）
            jira_repo
                .add_comment(ticket, pr_url)
                .map_err(|e| format!("Failed to add comment: {}", e))?;

            Ok(())
        },
    )?;

    success!("Updated Jira ticket {} to status: {}", ticket, status);

    // 写入工作历史记录
    work_history_repo
        .write_work_history(
            ticket,
            pr_id,
            Some(pr_url),
            repository_url,
            Some(branch_name),
        )
        .map_err(|e| format!("Failed to write work history: {}", e))?;

    info!("Work history recorded for PR #{}", pr_id);

    Ok(())
}

/// 从 PR URL 提取 PR ID
///
/// 支持 GitHub PR URL 格式：`https://github.com/owner/repo/pull/123`
pub fn extract_pr_id_from_url(pr_url: &str) -> Option<String> {
    // 匹配 /pull/123 或 /pulls/123 格式
    let parts: Vec<&str> = pr_url.split('/').collect();

    // 查找 "pull" 或 "pulls" 后面的数字
    for (i, part) in parts.iter().enumerate() {
        if (*part == "pull" || *part == "pulls") && i + 1 < parts.len() {
            let pr_id = parts[i + 1];
            // 移除可能的查询参数
            let pr_id = pr_id.split('?').next().unwrap_or(pr_id);
            if pr_id.chars().all(|c| c.is_ascii_digit()) {
                return Some(pr_id.to_string());
            }
        }
    }

    None
}

/// PR 合并后更新 Jira ticket
///
/// 尝试从工作历史或 PR 标题获取关联的 Jira ticket，更新状态到"已合并"，
/// 并清理工作历史记录。
///
/// # 参数
///
/// * `jira_repo` - Jira 仓储
/// * `work_history_repo` - 工作历史记录仓储
/// * `pr_id` - PR ID
/// * `pr_title` - PR 标题（用于提取 Jira ticket）
/// * `repository_url` - 仓库 URL（可选）
pub fn update_jira_after_pr_merged(
    jira_repo: &dyn JiraRepository,
    work_history_repo: &dyn JiraWorkHistoryRepository,
    pr_id: &str,
    pr_title: Option<&str>,
    repository_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use domain::extract_jira_ticket_id;
    use prompt::warning;

    // 如果没有仓库 URL，跳过工作历史相关操作
    let repo_url = repository_url.unwrap_or("");

    // 1. 尝试从工作历史读取 Jira ticket
    let mut jira_ticket = if !repo_url.is_empty() {
        work_history_repo.read_work_history(pr_id, repo_url).ok().flatten()
    } else {
        None
    };

    // 2. 如果工作历史中没有，尝试从 PR 标题提取
    if jira_ticket.is_none() {
        if let Some(title) = pr_title {
            jira_ticket = extract_jira_ticket_id(title);
        }
    }

    // 3. 如果有 Jira ticket，更新状态
    if let Some(ref ticket) = jira_ticket {
        // 读取合并时的状态配置
        if let Ok(Some(status)) = jira_repo.read_pull_request_merged_status(ticket) {
            spinner!("Updating Jira ticket {} to status: {}...", ticket, status)
                .with(|| jira_repo.update_issue_status(ticket, &status))
                .map_err(|e| format!("Failed to update Jira status: {}", e))?;

            success!("Jira ticket {} updated to: {}", ticket, status);
        } else {
            warning!(
                "No Jira merged status configuration found for ticket: {}",
                ticket
            );
        }
    } else {
        info!("No Jira ticket associated with this PR");
    }

    // 4. 删除工作历史记录中的 PR 条目（仅当有仓库 URL 时）
    if !repo_url.is_empty() {
        let delete_result = work_history_repo
            .delete_work_history_entry(pr_id, repo_url)
            .map_err(|e| format!("Failed to delete work history entry: {}", e))?;

        // 显示删除消息
        for message in &delete_result.messages {
            info!("{}", message);
        }

        // 显示警告信息
        for warning_msg in &delete_result.warnings {
            warning!("{}", warning_msg);
        }
    }

    Ok(())
}
