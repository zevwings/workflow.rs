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
        let current_location = self.get_ticket_output_dir_path(jira_id);
        if current_location.exists() {
            return self.find_log_file(&current_location);
        }

        // 如果目录不存在，返回默认路径
        self.find_log_file(&current_location)
    }

    /// 获取 ticket 输出目录路径
    ///
    /// 返回当前标准的 ticket 日志输出目录路径：`{base_dir}/jira/{jira_id}/{output_folder}`
    fn get_ticket_output_dir_path(&self, jira_id: &str) -> PathBuf {
        let ticket_dir = self.base_dir.join("jira").join(jira_id);
        if !self.output_folder_name.is_empty() {
            ticket_dir.join(&self.output_folder_name)
        } else {
            ticket_dir.join(DEFAULT_OUTPUT_FOLDER)
        }
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
