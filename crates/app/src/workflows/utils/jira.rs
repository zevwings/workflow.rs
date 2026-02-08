use domain::jira::validate_jira_ticket_format;
use domain::{extract_jira_project, JiraRepository, JiraStatusConfig};
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
        validate_jira_ticket_format(&id).map_err(|e| format!("无效的 JIRA ID 格式: {}", e))?;
        Ok(id)
    } else {
        // 交互式输入
        let id = input!("请输入您的 JIRA 工单 ID（例如: PROJ-123）:")
            .validator(|input: &str| {
                validate_jira_ticket_format(input)
                    .map_err(|e| format!("无效的 JIRA ID 格式: {}", e))
            })
            .prompt()
            .map_err(|e| format!("获取 JIRA ID 失败: {}", e))?;
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
        validate_jira_ticket_format(&id).map_err(|e| format!("无效的 JIRA ID 格式: {}", e))?;
        Ok(Some(id))
    } else {
        // 交互式输入
        let id = input!("请输入您的 JIRA 工单 ID（可选，按 Enter 跳过）:").prompt().ok();

        if let Some(id) = id {
            let trimmed = id.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                // 验证格式
                validate_jira_ticket_format(trimmed)
                    .map_err(|e| format!("无效的 JIRA ID 格式: {}", e))?;
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
    info!("未找到 {} 的状态配置，正在配置...", ticket);

    let config_result = configure_jira_status_interactive(jira_repo, ticket)?;

    success!("Jira 状态配置已保存");
    info!(
        "  PR 创建状态: {}",
        config_result.created_pull_request_status
    );
    info!(
        "  PR 合并状态: {}",
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
            "无效的 Jira 工单格式: 无法从 '{}' 中提取项目名称",
            jira_ticket
        )
    })?;

    // 验证项目名格式
    if !project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "无效的 Jira 项目名称格式: '{}'。Jira 项目名称应仅包含 ASCII 字母、数字和下划线。",
            project
        )
        .into());
    }

    // 从 Jira API 获取项目状态列表
    let statuses = spinner!("正在获取项目 {} 的状态列表...", project)
        .with(|| jira_repo.get_project_statuses(project))
        .map_err(|e| format!(
            "获取 '{}' 的项目状态失败。请检查:\n  - 项目名称是否正确\n  - 项目是否存在于您的 Jira 实例中\n  - 您是否有访问此项目的权限\n错误: {}",
            project, e
        ))?;

    if statuses.is_empty() {
        return Err(format!("项目 {} 未找到任何状态", project).into());
    }

    // 交互式选择 PR 创建时的状态
    let created_pull_request_status = select!("选择 PR 创建时的状态:", statuses.clone())
        .prompt()
        .map_err(|e| format!("选择状态失败: {}", e))?;

    // 交互式选择 PR 合并时的状态
    let merged_pull_request_status = select!("选择 PR 合并时的状态:", statuses)
        .prompt()
        .map_err(|e| format!("选择状态失败: {}", e))?;

    // 保存配置
    let jira_config = JiraStatusConfig {
        project: project.to_string(),
        created_pull_request_status: Some(created_pull_request_status.clone()),
        merged_pull_request_status: Some(merged_pull_request_status.clone()),
    };

    jira_repo
        .write_status_config(&jira_config)
        .map_err(|e| format!("写入 Jira 状态配置失败: {}", e))?;

    Ok(JiraStatusConfigResult {
        project: project.to_string(),
        created_pull_request_status,
        merged_pull_request_status,
    })
}

/// 从 PR URL 提取 PR ID
///
/// 支持 GitHub PR URL 格式：`https://github.com/owner/repo/pull/123`
pub fn extract_pr_id_from_url(pr_url: impl AsRef<str>) -> Option<String> {
    let pr_url = pr_url.as_ref();
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
