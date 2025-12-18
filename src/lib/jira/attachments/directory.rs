//! 目录管理

use super::constants::DOWNLOADS_FOLDER;
use crate::base::util::directory::DirectoryWalker;
use color_eyre::{eyre::WrapErr, Result};
use std::path::{Path, PathBuf};

/// 目录管理器
///
/// 提供下载目录的创建和管理功能。
pub struct DirectoryManager;

impl DirectoryManager {
    /// 准备下载目录
    ///
    /// 创建下载目录结构：
    /// - `base_dir/jira/{jira_id}/downloads/`
    ///
    /// 如果目录已存在，会先删除再创建。
    ///
    /// # 参数
    ///
    /// * `base_dir` - 基础目录路径
    /// * `jira_id` - Jira ticket ID
    ///
    /// # 返回
    ///
    /// 返回 `(download_base_dir, download_dir)` 元组：
    /// - `download_base_dir`: `base_dir/jira/{jira_id}/`
    /// - `download_dir`: `base_dir/jira/{jira_id}/downloads/`
    pub fn prepare_download_directory(
        base_dir: &Path,
        jira_id: &str,
    ) -> Result<(PathBuf, PathBuf)> {
        let download_base_dir = base_dir.join("jira").join(jira_id);
        let download_dir = download_base_dir.join(DOWNLOADS_FOLDER);

        // 如果目录已存在，删除它
        if download_base_dir.exists() {
            std::fs::remove_dir_all(&download_base_dir)
                .wrap_err("Failed to remove existing directory")?;
        }

        DirectoryWalker::new(&download_dir).ensure_exists()?;

        Ok((download_base_dir, download_dir))
    }

    /// 清理目录（如果下载失败）
    ///
    /// 在下载失败时清理已创建的目录。
    ///
    /// # 参数
    ///
    /// * `dir` - 要清理的目录路径
    ///
    /// # 返回
    ///
    /// 如果清理成功，返回 `Ok(())`；否则返回错误。
    pub fn cleanup_on_failure(dir: &Path) -> Result<()> {
        if dir.exists() {
            std::fs::remove_dir_all(dir).wrap_err("Failed to cleanup directory")?;
        }
        Ok(())
    }
}
