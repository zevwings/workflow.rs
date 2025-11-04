use crate::{log_info, log_success, utils::string, Jira};
use crate::jira::status;
use anyhow::{Context, Result};
use dialoguer::Select;

/// Jira 状态管理命令
pub struct JiraStatus;

impl JiraStatus {
    /// 交互式配置 Jira 状态
    pub fn configure_interactive(jira_ticket_or_project: &str) -> Result<()> {
        // 从 ticket 或项目名中提取项目名
        let project = string::extract_jira_project(jira_ticket_or_project)
            .unwrap_or(jira_ticket_or_project);

        // 获取项目状态列表
        log_info!("Fetching status list for project: {}", project);
        let statuses =
            Jira::get_project_statuses(project).context("Failed to get project statuses")?;

        if statuses.is_empty() {
            anyhow::bail!("No statuses found for project: {}", project);
        }

        // 选择 PR 创建时的状态
        log_info!("\nSelect one of the following states to change when PR is ready or In progress:");

        // 使用 Select 进行单选
        let selection = Select::new()
            .with_prompt("Select status for PR created")
            .items(&statuses)
            .interact()
            .context("Failed to select status")?;

        let created_pr_status = &statuses[selection];

        // 选择 PR 合并时的状态
        log_info!("\nSelect one of the following states to change when PR is merged or Done:");
        let selection = Select::new()
            .with_prompt("Select status for PR merged")
            .items(&statuses)
            .interact()
            .context("Failed to select status")?;

        let merged_pr_status = &statuses[selection];

        // 写入配置
        let jira_config = status::JiraStatusConfig {
            project: project.to_string(),
            created_pr_status: Some(created_pr_status.clone()),
            merged_pr_status: Some(merged_pr_status.clone()),
        };

        status::write_jira_status(&jira_config)
            .context("Failed to write Jira status configuration")?;

        log_success!("Jira status configuration saved");
        log_info!("  PR created status: {}", created_pr_status);
        log_info!("  PR merged status: {}", merged_pr_status);

        Ok(())
    }

    /// 读取 PR 创建时的状态
    pub fn read_pr_created_status(jira_ticket: &str) -> Result<Option<String>> {
        let project = string::extract_jira_project(jira_ticket)
            .context("Invalid Jira ticket format")?;

        let config = status::read_jira_status(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.created_pr_status)
    }

    /// 读取 PR 合并时的状态
    pub fn read_pr_merged_status(jira_ticket: &str) -> Result<Option<String>> {
        let project = string::extract_jira_project(jira_ticket)
            .context("Invalid Jira ticket format")?;

        let config = status::read_jira_status(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.merged_pr_status)
    }
}
