//! Git 配置管理
//!
//! 本模块提供了 Git 配置相关的功能，包括：
//! - 设置全局 user.email 和 user.name
//! - 读取 Git 配置

use anyhow::{Context, Result};
use duct::cmd;

use crate::{log_info, log_message};

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
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn set_global_user(email: &str, name: &str) -> Result<()> {
        // 设置全局 user.email
        cmd("git", &["config", "--global", "user.email", email])
            .run()
            .context("Failed to set git global user.email")?;

        // 设置全局 user.name
        cmd("git", &["config", "--global", "user.name", name])
            .run()
            .context("Failed to set git global user.name")?;

        log_info!("Git global config updated:");
        log_message!("  user.email: {}", email);
        log_message!("  user.name: {}", name);

        Ok(())
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
        let email = cmd("git", &["config", "--global", "user.email"])
            .read()
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let name = cmd("git", &["config", "--global", "user.name"])
            .read()
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Ok((email, name))
    }
}
