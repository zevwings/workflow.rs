//! 清理 Jira 附件命令

use prompt::{confirm, info, spinner, success};

use crate::registry;
use crate::workflows::utils::jira::get_jira_id_interactive;

/// Jira Clean 命令
pub struct JiraCleanCommand {
    jira_id: Option<String>,
    all: bool,
}

impl JiraCleanCommand {
    /// 创建新的 JiraCleanCommand
    pub fn new(jira_id: Option<String>, all: bool) -> Self {
        Self { jira_id, all }
    }

    /// 运行 `workflow jira clean` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取 JiraRepository
        let jira_repo = registry::get_jira_repository();

        if self.all {
            // 清理所有附件目录
            info!("Cleaning all Jira attachment directories...");

            // 交互式确认
            let confirmed =
                confirm!("Are you sure you want to delete ALL Jira attachment directories?")
                    .default(false)
                    .prompt()
                    .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if !confirmed {
                info!("Clean operation cancelled.");
                return Ok(());
            }

            spinner!("Cleaning all attachment directories...")
                .with(|| jira_repo.clean_attachments(None))
                .map_err(|e| format!("Failed to clean attachments: {}", e))?;

            success!("All Jira attachment directories cleaned successfully");
        } else {
            // 清理指定 JIRA ID 的附件目录
            let jira_id = get_jira_id_interactive(self.jira_id.clone())?;

            info!("Cleaning attachment directory for {}...", jira_id);

            // 交互式确认
            let confirmed = confirm!(
                "Are you sure you want to delete the attachment directory for {}?",
                jira_id
            )
            .default(false)
            .prompt()
            .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if !confirmed {
                info!("Clean operation cancelled.");
                return Ok(());
            }

            spinner!("Cleaning attachment directory for {}...", jira_id)
                .with(|| jira_repo.clean_attachments(Some(&jira_id)))
                .map_err(|e| format!("Failed to clean attachments: {}", e))?;

            success!("Attachment directory for {} cleaned successfully", jira_id);
        }

        Ok(())
    }
}
