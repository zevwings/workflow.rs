//! Jira 日志处理结构体
//! 提供从 Jira 下载的日志文件的下载、搜索、查找和处理功能

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::base::settings::defaults::default_download_base_dir;
use crate::base::settings::Settings;

// 子模块
mod clean;
mod constants;
mod download;
mod helpers;
mod path;
mod search;
mod zip;

// 重新导出 LogEntry 作为公共 API
pub use helpers::LogEntry;

// 重新导出下载相关的类型
pub use download::{DownloadResult, ProgressCallback};

/// Jira 日志处理结构体
///
/// 提供从 Jira 下载的日志文件的下载、搜索、查找和处理功能
pub struct JiraLogs {
    /// 缓存的 Settings 实例
    #[allow(dead_code)]
    pub(crate) settings: Settings,
    /// 缓存的展开后的基础目录路径
    pub(crate) base_dir: PathBuf,
    /// 缓存的日志输出文件夹名称
    pub(crate) output_folder_name: String,
    /// 缓存的 HTTP 客户端
    pub(crate) http_client: reqwest::blocking::Client,
}

impl JiraLogs {
    /// 创建新的 JiraLogs 实例
    ///
    /// 从 Settings 读取配置并初始化缓存，避免重复获取配置。
    ///
    /// # 返回
    ///
    /// 如果成功返回 `Ok(JiraLogs)`，否则返回错误（通常是路径展开失败）。
    pub fn new() -> Result<Self> {
        let settings = Settings::get().clone();
        let base_dir_str = settings
            .log
            .download_base_dir
            .clone()
            .unwrap_or_else(default_download_base_dir);
        let base_dir = helpers::expand_path(&base_dir_str)
            .with_context(|| format!("Failed to expand path: {}", base_dir_str))?;
        let output_folder_name = settings.log.output_folder_name.clone();

        // 创建并缓存 HTTP 客户端
        let http_client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            settings,
            base_dir,
            output_folder_name,
            http_client,
        })
    }
}
