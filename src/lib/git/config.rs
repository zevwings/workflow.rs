//! Git 配置管理
//!
//! 本模块提供了 Git 配置相关的功能，包括：
//! - 设置全局 user.email 和 user.name
//! - 读取 Git 配置

use color_eyre::{eyre::WrapErr, Result};

use super::GitCommand;
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
    /// 根据提供的 email 和 name 设置 Git 的全局 user.email 和 user.name 配置。
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
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn set_global_user(email: &str, name: &str) -> Result<GitConfigResult> {
        trace_info!("Updating Git global config: email={}, name={}", email, name);

        // 设置全局 user.email
        GitCommand::new(["config", "--global", "user.email", email])
            .run()
            .wrap_err("Failed to set git global user.email")?;

        // 设置全局 user.name
        GitCommand::new(["config", "--global", "user.name", name])
            .run()
            .wrap_err("Failed to set git global user.name")?;

        trace_info!("Git global config updated successfully");

        Ok(GitConfigResult {
            email: email.to_string(),
            name: name.to_string(),
        })
    }

    /// 读取 Git 全局配置
    ///
    /// 读取 Git 的全局 user.email 和 user.name 配置。
    ///
    /// # 返回
    ///
    /// 返回一个元组 `(email, name)`，如果配置不存在则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn get_global_user() -> Result<(Option<String>, Option<String>)> {
        let email = GitCommand::new(["config", "--global", "user.email"])
            .read()
            .ok()
            .filter(|s| !s.is_empty());

        let name = GitCommand::new(["config", "--global", "user.name"])
            .read()
            .ok()
            .filter(|s| !s.is_empty());

        Ok((email, name))
    }
}
