//! 清理 Jira 附件命令

use std::{fs, path::PathBuf};

use prompt::{confirm, error, info, multiselect, spinner, success};

use crate::bootstrap;
use crate::commands::jira::utils::get_jira_id_interactive;

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
        // 获取 JiraRepository 和 PathService
        let jira_repo = bootstrap::get_jira_repository();
        let path_service = bootstrap::get_path_service();

        if self.all {
            // 第一步：询问是否清理所有附件
            let clean_all = confirm!("Do you want to clean all attachments?")
                .default(false)
                .prompt()
                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if clean_all {
                // 清理所有附件目录
                info!("Cleaning all Jira attachment directories...");

                spinner!("Cleaning all attachment directories...")
                    .with(|| jira_repo.clean_attachments(None))
                    .map_err(|e| format!("Failed to clean attachments: {}", e))?;

                success!("All Jira attachment directories cleaned successfully");
            } else {
                // 扫描文件夹并让用户选择
                info!("Scanning attachment directories...");

                let download_dir = path_service.get_download_dir()?;
                let directories = self.scan_attachment_directories(&download_dir)?;

                if directories.is_empty() {
                    info!("No attachment directories found.");
                    return Ok(());
                }

                info!("Found {} attachment directory(ies)", directories.len());
                info!("");

                // 创建选项列表（显示目录名和大小）
                let options: Vec<String> = directories
                    .iter()
                    .map(|(name, size)| format!("{} ({})", name, Self::format_size(*size)))
                    .collect();

                // 让用户多选要删除的目录
                let selected = multiselect!(
                    "Select directories to delete (Space to select, Enter to confirm)",
                    options
                )
                .prompt()
                .map_err(|e| format!("Failed to get selection: {}", e))?;

                if selected.is_empty() {
                    info!("No directories selected. Clean operation cancelled.");
                    return Ok(());
                }

                // 确认删除
                info!("");
                let confirmed = confirm!(
                    "Are you sure you want to delete {} selected directory(ies)?",
                    selected.len()
                )
                .default(false)
                .prompt()
                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

                if !confirmed {
                    info!("Clean operation cancelled.");
                    return Ok(());
                }

                // 删除选中的目录
                let mut deleted_count = 0;
                let mut failed_count = 0;

                for selected_option in selected {
                    // 从选项文本中提取目录名（格式："DIR-123 (1.23 MB)"）
                    let dir_name =
                        selected_option.split(" (").next().unwrap_or(&selected_option).to_string();

                    let dir_path = download_dir.join(&dir_name);

                    match fs::remove_dir_all(&dir_path) {
                        Ok(_) => {
                            info!("✓ Deleted: {}", dir_name);
                            deleted_count += 1;
                        }
                        Err(e) => {
                            error!("✗ Failed to delete {}: {}", dir_name, e);
                            failed_count += 1;
                        }
                    }
                }

                info!("");
                if failed_count == 0 {
                    success!("Successfully deleted {} directory(ies)", deleted_count);
                } else {
                    error!(
                        "Deleted {} directory(ies), {} failed",
                        deleted_count, failed_count
                    );
                }
            }
        } else {
            // 清理指定 JIRA ID 的附件目录
            let jira_id = get_jira_id_interactive(self.jira_id.clone())?;

            info!("Cleaning attachment directory for {}...", jira_id);

            // 交互式确认
            let confirmed = confirm!(
                "Are you sure you want to delete the attachment directory for {}?",
                jira_id
            )
            .default(true)
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

    /// 扫描附件目录，返回 (目录名, 大小) 列表
    fn scan_attachment_directories(
        &self,
        download_dir: &PathBuf,
    ) -> Result<Vec<(String, u64)>, Box<dyn std::error::Error>> {
        let mut directories = Vec::new();

        if !download_dir.exists() {
            return Ok(directories);
        }

        for entry in fs::read_dir(download_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name =
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();

                // 计算目录大小
                let size = Self::calculate_dir_size(&path)?;

                directories.push((dir_name, size));
            }
        }

        // 按名称排序
        directories.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(directories)
    }

    /// 计算目录大小（递归）
    fn calculate_dir_size(path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        let mut total_size = 0;

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    total_size += Self::calculate_dir_size(&entry_path)?;
                } else {
                    total_size += entry.metadata()?.len();
                }
            }
        }

        Ok(total_size)
    }

    /// 格式化文件大小
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}
