//! 设置 Jira 配置命令

use domain::extract_jira_project;
use prompt::{spinner, success};

use crate::registry;
use crate::workflows::get_jira_id_interactive;

/// Jira Setup 命令
pub struct JiraAssignCommand {
    jira_id: Option<String>,
}

impl Default for JiraAssignCommand {
    fn default() -> Self {
        Self::new(None)
    }
}

impl JiraAssignCommand {
    /// 创建新的 PullRequestCreateCommand
    pub fn new(jira_id: Option<String>) -> Self {
        Self { jira_id }
    }

    /// 运行 `workflow jira assign` 命令
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

        let result = spinner!("Assigning Jira ticket {} to user...", jira_id)
            .with(|| jira_repo.assign_issue(&jira_id, None))
            .map_err(|e| format!("Failed to assign Jira ticket: {}", e))?;

        success!(
            "Jira ticket {} assigned to \"{}\"",
            jira_id,
            result.display_name
        );

        Ok(())
    }
}
