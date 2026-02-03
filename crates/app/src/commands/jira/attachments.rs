//! 下载 Jira ticket 附件命令

use crate::registry;
use crate::workflows::utils::jira::get_jira_id_interactive;
use color_eyre::Result;
use prompt::{error, info, spinner, success};
use toolkit::paths::{default_download_base_dir, Paths};

/// Jira Attachments 命令
pub struct JiraAttachmentsCommand {
    jira_id: Option<String>,
}

impl JiraAttachmentsCommand {
    /// 创建新的 JiraAttachmentsCommand
    pub fn new(jira_id: Option<String>) -> Self {
        Self { jira_id }
    }

    /// 运行 `workflow jira attachments` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive(self.jira_id.clone())?;

        // 获取 JiraRepository
        let jira_repo = registry::get_jira_repository();

        // 获取基础目录（使用 domain 层的配置函数）
        let dir_str = default_download_base_dir();
        let base_dir = Paths::expand(&dir_str).map_err(|e| {
            format!(
                "Failed to expand attachments base directory '{}': {}",
                dir_str, e
            )
        })?;

        // 下载所有附件（通过 domain 接口）
        info!("Downloading all attachments for {}...", jira_id);

        let result = spinner!("Downloading attachments for {}...", jira_id)
            .with(|| jira_repo.download_attachments(&jira_id, &base_dir))
            .map_err(|e| format!("Failed to download attachments: {}", e))?;

        if result.downloaded_files.is_empty() {
            error!("No attachments were downloaded");
            return Err("No attachments found or all downloads failed".into());
        }

        success!(
            "Downloaded {} attachment(s) to: {}",
            result.downloaded_files.len(),
            result.base_dir.display()
        );

        if !result.failed_files.is_empty() {
            error!(
                "Failed to download {} attachment(s):",
                result.failed_files.len()
            );
            for (filename, error_msg) in &result.failed_files {
                error!("  - {}: {}", filename, error_msg);
            }
        }

        Ok(())
    }
}
