use crate::commands::logs::{download, find, search};
use crate::log_success;
use crate::settings::Settings;
use anyhow::{Context, Result};
use dialoguer::Input;
use std::path::{Path, PathBuf};

/// Qk 统一命令包装器
/// 对应 Shell 脚本 qk.sh
pub struct Qk;

impl Qk {
    /// 查找日志文件
    /// 在指定目录中查找 flutter-api*.log 文件
    fn find_log_file(base_dir: &Path) -> Result<PathBuf> {
        // 尝试查找 flutter-api*.log 文件
        let log_files: Vec<_> = std::fs::read_dir(base_dir)
            .context("Failed to read directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            return name.starts_with("flutter-api") && name.ends_with(".log");
                        }
                    }
                }
                false
            })
            .map(|entry| entry.path())
            .collect();

        if let Some(log_file) = log_files.first() {
            Ok(log_file.clone())
        } else {
            // 如果没找到，返回默认路径
            Ok(base_dir.join("flutter-api.log"))
        }
    }

    /// 获取日志文件路径
    fn get_log_file_path(jira_id: &str) -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_path = PathBuf::from(&home);

        // 从 Settings 获取日志输出文件夹名称
        let settings = Settings::get();
        let base_dir = if !settings.log_output_folder_name.is_empty() {
            home_path.join(format!("Downloads/logs_{}/{}/merged", jira_id, settings.log_output_folder_name))
        } else {
            home_path.join(format!("Downloads/logs_{}/merged", jira_id))
        };

        // 如果 merged 目录不存在，尝试查找其他目录
        if !base_dir.exists() {
            // 尝试在 logs_<JIRA_ID> 目录下查找
            let logs_dir = home_path.join(format!("Downloads/logs_{}", jira_id));
            if logs_dir.exists() {
                // 查找 merged 或任何包含 flutter-api*.log 的目录
                if let Ok(entries) = std::fs::read_dir(&logs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let potential_log_file = path.join("flutter-api.log");
                            if potential_log_file.exists() {
                                return Ok(potential_log_file);
                            }
                        }
                    }
                }
            }
        }

        Self::find_log_file(&base_dir)
    }

    /// 下载日志
    pub fn download(jira_id: &str) -> Result<()> {
        log_success!("Downloading logs for {}...", jira_id);
        download::LogsDownloadCommand::download(jira_id)?;
        Ok(())
    }

    /// 查找请求 ID
    pub fn find_request_id(jira_id: &str, request_id: Option<String>) -> Result<()> {
        // 1. 获取日志文件路径
        let log_file = Self::get_log_file_path(jira_id)?;

        // 2. 检查日志文件是否存在
        if !log_file.exists() {
            anyhow::bail!(
                "❌ Log file not found at: {:?}\n💡 Try downloading logs first with: workflow qk {} download",
                log_file, jira_id
            );
        }

        // 3. 获取请求 ID（从参数或交互式输入）
        let req_id = if let Some(id) = request_id {
            id
        } else {
            Input::<String>::new()
                .with_prompt("Enter request ID to find")
                .interact()
                .context("Failed to read request ID")?
        };

        // 4. 调用 find 命令
        log_success!("Finding request ID: {}...", req_id);
        find::LogsFindCommand::find(&log_file, &req_id, Some(jira_id))?;
        Ok(())
    }

    /// 搜索关键词
    pub fn search(jira_id: &str, search_term: Option<String>) -> Result<()> {
        // 1. 获取日志文件路径
        let log_file = Self::get_log_file_path(jira_id)?;

        // 2. 检查日志文件是否存在
        if !log_file.exists() {
            anyhow::bail!(
                "❌ Log file not found at: {:?}\n💡 Try downloading logs first with: workflow qk {} download",
                log_file, jira_id
            );
        }

        // 3. 获取搜索词（从参数或交互式输入）
        let term = if let Some(t) = search_term {
            t
        } else {
            Input::<String>::new()
                .with_prompt("Enter search term")
                .interact()
                .context("Failed to read search term")?
        };

        // 4. 调用 search 命令
        log_success!("Searching for: {}...", term);
        search::LogsSearchCommand::search(&log_file, &term)?;
        Ok(())
    }
}
