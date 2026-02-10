//! 设置 Jira 配置命令

use domain::extract_jira_project;
use prompt::{select, spinner, success};

use crate::registry;
use crate::utils::get_jira_id_interactive;

/// Jira Setup 命令
pub struct JiraTransitionCommand {
    jira_id: Option<String>,
}

impl Default for JiraTransitionCommand {
    fn default() -> Self {
        Self::new(None)
    }
}

impl JiraTransitionCommand {
    /// 创建新的 PullRequestCreateCommand
    pub fn new(jira_id: Option<String>) -> Self {
        Self { jira_id }
    }

    /// 运行 `workflow jira setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let jira_repo = registry::get_jira_repository();
        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive(self.jira_id.clone())?;

        let project = extract_jira_project(&jira_id).ok_or_else(|| {
            format!(
                "Invalid Jira ticket format: cannot extract project from '{}'",
                jira_id
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
        let created_pull_request_status =
            select!("Select status for PR created:", statuses.clone())
                .prompt()
                .map_err(|e| format!("Failed to select status: {}", e))?;

        spinner!(
            "Updating Jira ticket {} to status: {}...",
            jira_id,
            created_pull_request_status
        )
        .with(|| jira_repo.update_issue_status(&jira_id, &created_pull_request_status))
        .map_err(|e| format!("Failed to update Jira status: {}", e))?;

        success!(
            "Jira ticket {} updated to: {}",
            jira_id,
            created_pull_request_status
        );

        Ok(())
    }
}
