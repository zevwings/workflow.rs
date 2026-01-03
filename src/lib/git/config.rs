//! Git 配置管理
//!
//! 本模块提供了 Git 配置相关的功能，包括：
//! - 设置全局 user.email 和 user.name
//! - 读取 Git 配置

use color_eyre::{eyre::WrapErr, Result};

use crate::git::commands::GitConfigCommand;
use crate::trace_info;

/// Git 配置结果
#[derive(Debug, Clone)]
pub struct GitConfigResult {
    /// 用户邮箱
    pub email: String,
    /// 用户名称
    pub name: String,
}

/// Git 配置管理结构体
pub struct GitConfig;

impl GitConfig {
    /// 设置 Git 全局配置（email 和 name）
    ///
    /// 使用 Git 命令行工具根据提供的 email 和 name 设置 Git 的全局 user.email 和 user.name 配置。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    /// * `name` - 用户名称
    ///
    /// # 返回
    ///
    /// 返回 `GitConfigResult`，包含设置后的 email 和 name。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn set_global_user(email: &str, name: &str) -> Result<GitConfigResult> {
        trace_info!("Updating Git global config: email={}, name={}", email, name);

        // 使用 GitConfigCommand 设置全局用户配置
        GitConfigCommand::set_user(email, name, true, None)
            .wrap_err("Failed to set git global user config")?;

        trace_info!("Git global config updated successfully");

        Ok(GitConfigResult {
            email: email.to_string(),
            name: name.to_string(),
        })
    }

    /// 读取 Git 全局配置
    ///
    /// 使用 Git 命令行工具读取 Git 的全局 user.email 和 user.name 配置。
    ///
    /// # 返回
    ///
    /// 返回一个元组 `(email, name)`，如果配置不存在则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn get_global_user() -> Result<(Option<String>, Option<String>)> {
        // 使用 GitConfigCommand 读取全局用户配置
        let email = GitConfigCommand::get_user_email(true, None)
            .wrap_err("Failed to get git global user.email")?;

        let name = GitConfigCommand::get_user_name(true, None)
            .wrap_err("Failed to get git global user.name")?;

        Ok((email, name))
    }

    /// 读取 Git 配置项
    ///
    /// 使用 Git 命令行工具读取指定配置项的值。
    ///
    /// # 参数
    ///
    /// * `key` - 配置项键名（如 "branch.main.remote"）
    ///
    /// # 返回
    ///
    /// 返回配置项的值，如果配置不存在则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn get_config_string(key: &str) -> Result<Option<String>> {
        GitConfigCommand::get_config(key, true, None)
            .wrap_err_with(|| format!("Failed to read config: {}", key))
    }
}
