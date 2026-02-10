//! 下载 Jira ticket 附件命令

use prompt::{br, error, info, spinner, success};
use toolkit::expand;

use crate::registry::{get_jira_repository, get_path_service};
use crate::utils::get_jira_id_interactive;

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
        let jira_repo = get_jira_repository();
        // 获取 PathService
        let path_service = get_path_service();

        // 获取基础目录（使用 domain 层的配置函数）
        let base_dir = path_service.get_download_dir()?;
        let base_dir = expand(base_dir.to_string_lossy().as_ref()).map_err(|e| {
            format!(
                "Failed to expand attachments base directory '{}': {}",
                base_dir.display(),
                e
            )
        })?;

        // 1. 先获取附件列表以确定总数
        let attachments = spinner!("Getting attachments info for {}...", jira_id)
            .with(|| jira_repo.get_attachments(&jira_id))
            .map_err(|e| format!("Failed to get attachments: {}", e))?;

        let total_files = attachments.len();

        if total_files == 0 {
            error!("No attachments found for {}", jira_id);
            return Err("No attachments found".into());
        }

        // 2. 显示将要下载的文件数量
        info!("{} file(s) will be downloaded", total_files);
        br!();

        // 3. 使用 spinner 显示下载进度
        let result = spinner!("Downloading {} attachment(s)...", total_files)
            .with(|| jira_repo.download_attachments(&jira_id, &base_dir, None))
            .map_err(|e| format!("Failed to download attachments: {}", e))?;

        if result.downloaded_files.is_empty() {
            error!("No attachments were downloaded");
            return Err("All downloads failed".into());
        }

        success!("Download completed!");
        info!("");
        info!("Downloaded {} file(s):", result.downloaded_files.len());
        for file_path in &result.downloaded_files {
            if let Some(file_name) = file_path.file_name() {
                info!("  ✓ {}", file_name.to_string_lossy());
            }
        }
        info!("");
        info!("Files located at: {}", result.base_dir.display());

        if !result.failed_files.is_empty() {
            error!("");
            error!(
                "Warning: {} attachment(s) failed to download:",
                result.failed_files.len()
            );
            for (filename, error_msg) in &result.failed_files {
                error!("  - {}: {}", filename, error_msg);
            }
        }

        Ok(())
    }
}
