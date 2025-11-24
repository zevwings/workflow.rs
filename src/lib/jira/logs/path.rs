//! 路径管理相关功能

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::constants::*;
use super::JiraLogs;

impl JiraLogs {
    /// 获取基础目录路径
    pub fn get_base_dir_path(&self) -> &PathBuf {
        &self.base_dir
    }

    /// 获取日志文件路径
    /// 根据 JIRA ID 自动解析日志文件路径
    pub fn get_log_file_path(&self, jira_id: &str) -> Result<PathBuf> {
        // 尝试新位置
        let new_location = self.get_new_location_path(jira_id);
        if new_location.exists() {
            return self.find_log_file(&new_location);
        }

        // 向后兼容：尝试旧位置
        if let Ok(old_location) = self.get_old_location_path_v1(jira_id) {
            if old_location.exists() {
                return self.find_log_file(&old_location);
            }
        }

        // 向后兼容：尝试在旧目录下查找
        if let Ok(log_file) = self.find_log_file_in_old_directory(jira_id) {
            return Ok(log_file);
        }

        // 如果都找不到，返回新位置的默认路径
        self.find_log_file(&new_location)
    }

    /// 获取新位置路径
    fn get_new_location_path(&self, jira_id: &str) -> PathBuf {
        let ticket_dir = self.base_dir.join(jira_id);
        if !self.output_folder_name.is_empty() {
            ticket_dir.join(&self.output_folder_name)
        } else {
            ticket_dir.join(DEFAULT_OUTPUT_FOLDER)
        }
    }

    /// 获取旧位置路径（版本1）
    ///
    /// 跨平台支持：
    /// - Unix (macOS/Linux): `~/Downloads/logs_{jira_id}/...`
    /// - Windows: `%USERPROFILE%\Downloads\logs_{jira_id}\...`
    fn get_old_location_path_v1(&self, jira_id: &str) -> Result<PathBuf> {
        use std::env;
        let user_dir = if cfg!(target_os = "windows") {
            // Windows: 使用 %USERPROFILE%
            env::var("USERPROFILE")
                .context("USERPROFILE environment variable not set")?
        } else {
            // Unix-like: 使用 HOME
            env::var("HOME").context("HOME environment variable not set")?
        };
        let user_path = PathBuf::from(&user_dir);
        Ok(if !self.output_folder_name.is_empty() {
            user_path.join("Downloads").join(format!(
                "logs_{}",
                jira_id
            )).join(&self.output_folder_name).join(DEFAULT_OUTPUT_FOLDER)
        } else {
            user_path.join("Downloads").join(format!(
                "logs_{}",
                jira_id
            )).join(DEFAULT_OUTPUT_FOLDER)
        })
    }

    /// 在旧目录下查找日志文件
    ///
    /// 跨平台支持：
    /// - Unix (macOS/Linux): `~/Downloads/logs_{jira_id}`
    /// - Windows: `%USERPROFILE%\Downloads\logs_{jira_id}`
    fn find_log_file_in_old_directory(&self, jira_id: &str) -> Result<PathBuf> {
        use std::env;
        let user_dir = if cfg!(target_os = "windows") {
            // Windows: 使用 %USERPROFILE%
            env::var("USERPROFILE")
                .context("USERPROFILE environment variable not set")?
        } else {
            // Unix-like: 使用 HOME
            env::var("HOME").context("HOME environment variable not set")?
        };
        let user_path = PathBuf::from(&user_dir);
        let old_logs_dir = user_path.join("Downloads").join(format!("logs_{}", jira_id));

        if !old_logs_dir.exists() {
            anyhow::bail!("Old logs directory does not exist");
        }

        // 查找 merged 或任何包含 flutter-api*.log 的目录
        let entries =
            std::fs::read_dir(&old_logs_dir).context("Failed to read old logs directory")?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let potential_log_file = path.join(format!(
                    "{}{}",
                    FLUTTER_API_LOG_PREFIX, FLUTTER_API_LOG_SUFFIX
                ));
                if potential_log_file.exists() {
                    return Ok(potential_log_file);
                }
            }
        }

        anyhow::bail!("No log file found in old directory")
    }

    /// 确保日志文件存在
    /// 如果文件不存在，返回错误
    pub fn ensure_log_file_exists(&self, jira_id: &str) -> Result<PathBuf> {
        let log_file = self.get_log_file_path(jira_id)?;
        if !log_file.exists() {
            anyhow::bail!(
                "Log file not found at: {:?}\nTry downloading logs first with: workflow qk {} download",
                log_file, jira_id
            );
        }
        Ok(log_file)
    }

    /// 查找日志文件
    /// 在指定目录中查找 flutter-api*.log 文件
    fn find_log_file(&self, base_dir: &Path) -> Result<PathBuf> {
        // 尝试查找 flutter-api*.log 文件
        let log_files: Vec<_> = std::fs::read_dir(base_dir)
            .context("Failed to read directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            return name.starts_with(FLUTTER_API_LOG_PREFIX)
                                && name.ends_with(FLUTTER_API_LOG_SUFFIX);
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
            Ok(base_dir.join(format!(
                "{}{}",
                FLUTTER_API_LOG_PREFIX, FLUTTER_API_LOG_SUFFIX
            )))
        }
    }

    /// 获取 api.log 文件路径
    /// 基于 flutter-api.log 的路径，在同一目录下查找 api.log
    pub fn get_api_log_file_path(&self, jira_id: &str) -> Result<PathBuf> {
        let flutter_api_log = self.get_log_file_path(jira_id)?;
        let api_log = flutter_api_log
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?
            .join("api.log");
        Ok(api_log)
    }
}
