//! 附件路径管理
//!
//! 提供附件相关的路径管理功能，包括：
//! - 基础目录获取（展开配置路径）
//! - Ticket 基础目录构建

use crate::jira::JiraConfigProvider;
use color_eyre::Result;
use std::path::PathBuf;

/// 附件路径管理器
///
/// 提供附件相关的路径管理功能，包括基础目录获取和路径构建。
pub struct AttachmentPaths;

impl AttachmentPaths {
    /// 获取基础目录（展开配置路径）
    ///
    /// 从配置提供者读取 `download_base_dir` 配置。
    /// 默认使用 `SettingsAdapter` 从配置文件读取。
    ///
    /// # 返回
    ///
    /// 返回展开后的基础目录路径。
    ///
    /// # 错误
    ///
    /// 如果配置不存在或路径展开失败，返回相应的错误。
    pub fn base_dir() -> Result<PathBuf> {
        Self::base_dir_with_config(None)
    }

    /// 使用指定的配置提供者获取基础目录
    ///
    /// # 参数
    ///
    /// * `config` - 可选的配置提供者，如果为 `None`，使用默认配置适配器
    ///
    /// # 返回
    ///
    /// 返回展开后的基础目录路径。
    ///
    /// # 错误
    ///
    /// 如果配置不存在或路径展开失败，返回相应的错误。
    pub fn base_dir_with_config(config: Option<&dyn JiraConfigProvider>) -> Result<PathBuf> {
        let config_provider = if let Some(cfg) = config {
            cfg
        } else {
            // 使用默认配置适配器
            use crate::infra::adapters::config::SettingsAdapter;
            let adapter = SettingsAdapter::new();
            // 将适配器转换为静态引用（通过 Box::leak）
            Box::leak(Box::new(adapter))
        };

        config_provider.get_download_base_dir()
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
