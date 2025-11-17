//! 配置路径管理
//!
//! 统一管理所有配置文件的路径，所有配置文件存储在 `~/.workflow/config/` 目录下。

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 配置路径管理器
pub struct ConfigPaths;

impl ConfigPaths {
    /// 获取配置目录路径
    ///
    /// 返回 `~/.workflow/config/` 目录路径，如果目录不存在则创建。
    ///
    /// # 返回
    ///
    /// 返回配置目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let config_dir = PathBuf::from(home).join(".workflow").join("config");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir).context("Failed to create .workflow/config directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set config directory permissions")?;
        }

        Ok(config_dir)
    }

    /// 获取主配置文件路径
    ///
    /// 返回 `~/.workflow/config/workflow.toml` 的路径。
    pub fn workflow_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("workflow.toml"))
    }

    /// 获取 LLM 配置文件路径
    ///
    /// 返回 `~/.workflow/config/llm.toml` 的路径。
    pub fn llm_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("llm.toml"))
    }

    /// 获取 Jira 状态配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira-status.toml` 的路径。
    pub fn jira_status_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-status.toml"))
    }

    /// 获取 Jira 用户配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira-users.toml` 的路径。
    pub fn jira_users_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-users.toml"))
    }

    /// 获取工作流目录路径
    ///
    /// 返回 `~/.workflow/` 目录路径，如果目录不存在则创建。
    ///
    /// # 返回
    ///
    /// 返回工作流目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn workflow_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_dir = PathBuf::from(home).join(".workflow");

        // 确保工作流目录存在
        fs::create_dir_all(&workflow_dir).context("Failed to create .workflow directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set workflow directory permissions")?;
        }

        Ok(workflow_dir)
    }

    /// 获取工作历史记录目录路径
    ///
    /// 返回 `~/.workflow/work-history/` 目录路径，如果目录不存在则创建。
    ///
    /// # 返回
    ///
    /// 返回工作历史记录目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn work_history_dir() -> Result<PathBuf> {
        let history_dir = Self::workflow_dir()?.join("work-history");

        // 确保目录存在
        fs::create_dir_all(&history_dir)
            .context("Failed to create .workflow/work-history directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&history_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set work-history directory permissions")?;
        }

        Ok(history_dir)
    }
}
