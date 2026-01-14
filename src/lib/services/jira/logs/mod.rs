//! Jira 日志处理结构体
//! 提供从 Jira 下载的日志文件的下载、搜索、查找和处理功能

use crate::jira::JiraConfigProvider;
use color_eyre::Result;
use std::path::PathBuf;

// 子模块
mod constants;
mod download;
mod helpers;
mod path;
mod search;
mod table;

// 重新导出 LogEntry 作为公共 API
pub use helpers::LogEntry;

// 重新导出下载相关的类型（从 attachments 模块）
pub use crate::jira::attachments::{DownloadResult, ProgressCallback};

// 重新导出清理相关的类型（从 attachments 模块，保持向后兼容）
pub use crate::jira::attachments::{CleanResult, DirEntry, DirInfo};

// 重新导出表格相关类型
pub use table::SearchResultRow;

/// Jira 日志处理结构体
///
/// 提供从 Jira 下载的日志文件的下载、搜索、查找和处理功能
pub struct JiraLogs {
    /// 缓存的展开后的基础目录路径
    pub(crate) base_dir: PathBuf,
    /// 缓存的日志输出文件夹名称
    pub(crate) output_folder_name: String,
}

impl JiraLogs {
    /// 创建新的 JiraLogs 实例
    ///
    /// 从配置提供者读取配置并初始化缓存，避免重复获取配置。
    /// 默认使用 `SettingsAdapter` 从配置文件读取。
    ///
    /// # 返回
    ///
    /// 如果成功返回 `Ok(JiraLogs)`，否则返回错误（通常是路径展开失败）。
    pub fn new() -> Result<Self> {
        Self::new_with_config(None)
    }

    /// 使用指定的配置提供者创建新的 JiraLogs 实例
    ///
    /// # 参数
    ///
    /// * `config` - 可选的配置提供者，如果为 `None`，使用默认配置适配器
    ///
    /// # 返回
    ///
    /// 如果成功返回 `Ok(JiraLogs)`，否则返回错误（通常是路径展开失败）。
    pub fn new_with_config(config: Option<&dyn JiraConfigProvider>) -> Result<Self> {
        let config_provider = if let Some(cfg) = config {
            cfg
        } else {
            // 使用默认配置适配器
            use crate::infra::adapters::config::SettingsAdapter;
            let adapter = SettingsAdapter::new();
            // 将适配器转换为静态引用（通过 Box::leak）
            Box::leak(Box::new(adapter))
        };

        let base_dir = config_provider.get_download_base_dir()?;
        let output_folder_name = config_provider.get_log_output_folder_name();

        Ok(Self {
            base_dir,
            output_folder_name,
        })
    }
}
