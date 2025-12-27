//! Git 配置管理
//!
//! 本模块提供了 Git 配置相关的功能，包括：
//! - 设置全局 user.email 和 user.name
//! - 读取 Git 配置

use color_eyre::{eyre::WrapErr, Result};

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
    /// 使用 git2 库根据提供的 email 和 name 设置 Git 的全局 user.email 和 user.name 配置。
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

        // 打开全局配置
        let mut config =
            git2::Config::open_default().wrap_err("Failed to open Git global config")?;

        // 设置全局 user.email
        config
            .set_str("user.email", email)
            .wrap_err("Failed to set git global user.email")?;

        // 设置全局 user.name
        config
            .set_str("user.name", name)
            .wrap_err("Failed to set git global user.name")?;

        trace_info!("Git global config updated successfully");

        Ok(GitConfigResult {
            email: email.to_string(),
            name: name.to_string(),
        })
    }

    /// 读取 Git 全局配置
    ///
    /// 使用 git2 库读取 Git 的全局 user.email 和 user.name 配置。
    ///
    /// # 返回
    ///
    /// 返回一个元组 `(email, name)`，如果配置不存在则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn get_global_user() -> Result<(Option<String>, Option<String>)> {
        // 打开全局配置
        let config = git2::Config::open_default().wrap_err("Failed to open Git global config")?;

        // 读取 user.email
        let email = config.get_string("user.email").ok().filter(|s| !s.is_empty());

        // 读取 user.name
        let name = config.get_string("user.name").ok().filter(|s| !s.is_empty());

        Ok((email, name))
    }
}
