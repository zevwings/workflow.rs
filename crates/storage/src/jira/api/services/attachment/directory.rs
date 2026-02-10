//! 目录管理模块
//!
//! 提供附件下载目录的创建、清理等功能。

use domain::JiraError;
use std::path::{Path, PathBuf};

/// 目录管理器
pub struct DirectoryManager;

impl DirectoryManager {
    /// 准备下载目录
    ///
    /// 创建基础目录和下载子目录结构：
    /// ```text
    /// base_dir/
    ///   └── JIRA-123/           # download_base_dir
    ///       └── downloads/       # download_dir
    /// ```
    ///
    /// # 参数
    ///
    /// * `base_dir` - 基础目录路径
    /// * `jira_id` - JIRA ticket ID（如 "PROJ-123"）
    ///
    /// # 返回
    ///
    /// 返回 `(download_base_dir, download_dir)` 元组：
    /// - `download_base_dir`: JIRA ID 目录（如 `base_dir/PROJ-123`）
    /// - `download_dir`: 下载子目录（如 `base_dir/PROJ-123/downloads`）
    pub fn prepare_directory(base_dir: &Path, jira_id: &str) -> Result<PathBuf, JiraError> {
        // 创建 JIRA ID 目录
        let download_base_dir = base_dir.join("jira").join(jira_id);
        std::fs::create_dir_all(&download_base_dir).map_err(|e| {
            JiraError::IoError(format!(
                "Failed to create directory '{}': {}",
                download_base_dir.display(),
                e
            ))
        })?;

        Ok(download_base_dir)
    }

    /// 失败时清理目录
    ///
    /// 当下载失败时，删除已创建的目录结构。
    ///
    /// # 参数
    ///
    /// * `dir` - 要清理的目录路径
    pub fn cleanup_on_failure(dir: &Path) -> Result<(), JiraError> {
        if dir.exists() {
            std::fs::remove_dir_all(dir).map_err(|e| {
                JiraError::IoError(format!(
                    "Failed to cleanup directory '{}': {}",
                    dir.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_prepare_download_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        let download_dir = DirectoryManager::prepare_directory(base_dir, "PROJ-123").unwrap();

        assert!(download_dir.exists());
        assert_eq!(download_dir, base_dir.join("jira").join("PROJ-123"));
    }

    #[test]
    fn test_cleanup_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        let test_dir = base_dir.join("test");

        fs::create_dir_all(&test_dir).unwrap();
        assert!(test_dir.exists());

        DirectoryManager::cleanup_on_failure(&test_dir).unwrap();
        assert!(!test_dir.exists());
    }
}
