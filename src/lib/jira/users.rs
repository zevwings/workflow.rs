//! Jira 用户相关 API
//!
//! 本模块提供了用户信息获取和本地缓存功能：
//! - 从 Jira API 获取用户信息
//! - 本地缓存用户信息到 `${HOME}/.workflow/config/jira.toml`
//! - 优先使用本地缓存，减少 API 调用

use color_eyre::{eyre::WrapErr, Result};

use crate::base::settings::paths::Paths;

use super::api::user::JiraUserApi;
use super::config::{ConfigManager, JiraConfig, JiraUserEntry};
use super::helpers::get_auth;
use super::types::JiraUser;

/// Jira 用户管理
///
/// 提供用户信息获取和本地缓存功能：
/// - 从 Jira API 获取用户信息
/// - 本地缓存用户信息到 `${HOME}/.workflow/config/jira.toml`
/// - 优先使用本地缓存，减少 API 调用
pub struct JiraUsers;

impl JiraUsers {
    /// 从本地 TOML 文件读取用户信息
    ///
    /// 从 `jira.toml` 配置文件中读取指定邮箱的用户信息。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，如果文件不存在或用户不存在则返回错误。
    fn from_local(email: &str) -> Result<JiraUser> {
        let config_path = Paths::jira_config()?;
        let manager = ConfigManager::<JiraConfig>::new(config_path);
        let config = manager.read()?;

        if let Some(user_entry) = config.users.iter().find(|u| u.email == email) {
            Ok(JiraUser {
                account_id: user_entry.account_id.clone(),
                display_name: user_entry.display_name.clone(),
                email_address: Some(user_entry.email.clone()),
            })
        } else {
            color_eyre::eyre::bail!("User with email '{}' not found in jira.toml", email)
        }
    }

    /// 从远程 API 获取用户信息并保存到本地
    ///
    /// 调用 Jira API 的 `/myself` 接口获取当前用户信息，
    /// 并将结果保存到本地缓存文件中。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    /// * `api_token` - Jira API token（已不再使用，保留以保持接口兼容性）
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，包含用户的完整信息。
    fn from_remote(email: &str, _api_token: &str) -> Result<JiraUser> {
        let user = JiraUserApi::get_current_user().wrap_err("Failed to get current Jira user")?;

        if user.account_id.is_empty() {
            color_eyre::eyre::bail!("Failed to extract accountId from Jira user response");
        }

        Self::to_local(email, &user)?;

        Ok(user)
    }

    /// 保存用户信息到本地 TOML 文件
    ///
    /// 将用户信息添加到或更新到 `jira.toml` 配置文件中。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    /// * `user` - JiraUser 结构体
    fn to_local(email: &str, user: &JiraUser) -> Result<()> {
        let config_path = Paths::jira_config()?;
        let manager = ConfigManager::<JiraConfig>::new(config_path);

        manager.update(|config| {
            let email_to_save = user.email_address.as_deref().unwrap_or(email);
            let email_to_save_str = email_to_save.to_string();
            let user_entry = JiraUserEntry {
                email: email_to_save_str.clone(),
                account_id: user.account_id.clone(),
                display_name: user.display_name.clone(),
            };

            if let Some(existing) = config
                .users
                .iter_mut()
                .find(|u| u.email == email_to_save_str || u.email == email)
            {
                *existing = user_entry;
            } else {
                config.users.push(user_entry);
            }
        })
    }

    /// 获取当前 Jira 用户信息
    ///
    /// 优先从本地缓存读取用户信息，如果缓存不存在或读取失败，
    /// 则从 Jira API 获取并保存到本地缓存。
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，包含用户的 `account_id`、`display_name` 和 `email_address`。
    pub fn get() -> Result<JiraUser> {
        let (email, api_token) = get_auth()?;

        if let Ok(user) = Self::from_local(&email) {
            return Ok(user);
        }

        Self::from_remote(&email, &api_token)
    }
}
