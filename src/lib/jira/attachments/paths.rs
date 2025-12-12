//! 附件路径管理
//!
//! 提供附件相关的路径管理功能，包括：
//! - 基础目录获取（展开配置路径）
//! - Ticket 基础目录构建

use color_eyre::{eyre::WrapErr, Result};
use std::path::PathBuf;

use crate::base::settings::defaults::default_download_base_dir;
use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;

/// 附件路径管理器
///
/// 提供附件相关的路径管理功能，包括基础目录获取和路径构建。
pub struct AttachmentPaths;

impl AttachmentPaths {
    /// 获取基础目录（展开配置路径）
    ///
    /// 从 Settings 读取 `download_base_dir` 配置，并使用 `Paths::expand()` 展开路径。
    ///
    /// # 返回
    ///
    /// 返回展开后的基础目录路径。
    ///
    /// # 错误
    ///
    /// 如果配置不存在或路径展开失败，返回相应的错误。
    pub fn base_dir() -> Result<PathBuf> {
        let settings = Settings::get();
        let base_dir_str =
            settings.log.download_base_dir.clone().unwrap_or_else(default_download_base_dir);
        Paths::expand(&base_dir_str)
            .wrap_err_with(|| format!("Failed to expand path: {}", base_dir_str))
    }

    /// 获取 ticket 基础目录
    ///
    /// 构建路径：`base_dir/jira/{jira_id}/`
    ///
    /// # 参数
    ///
    /// * `jira_id` - Jira ticket ID
    ///
    /// # 返回
    ///
    /// 返回 ticket 基础目录路径。
    pub fn ticket_base_dir(jira_id: &str) -> Result<PathBuf> {
        let base_dir = Self::base_dir()?;
        Ok(base_dir.join("jira").join(jira_id))
    }

    /// 获取整个 jira 目录（用于清理）
    ///
    /// 构建路径：`base_dir/jira/`
    ///
    /// # 返回
    ///
    /// 返回整个 jira 目录路径。
    pub fn jira_base_dir() -> Result<PathBuf> {
        let base_dir = Self::base_dir()?;
        Ok(base_dir.join("jira"))
    }
}
