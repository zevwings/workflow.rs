//! Jira 用户相关 API
//!
//! 本模块提供了用户信息获取和本地缓存功能：
//! - 从 Jira API 获取用户信息
//! - 本地缓存用户信息到 `${HOME}/.workflow/config/jira-users.toml`
//! - 优先使用本地缓存，减少 API 调用

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::base::settings::paths::Paths;

use super::helpers::{get_auth, get_base_url};
use super::models::JiraUser;

/// Jira 用户配置（TOML）
///
/// TOML 格式示例：
/// ```toml
/// [[users]]  # 数组表（array of tables），可以包含多个用户条目
/// email = "user@example.com"
/// account_id = "628d9616269a9a0068f27e0c"
/// display_name = "User Name"
///
/// [[users]]  # 第二个用户条目
/// email = "another@example.com"
/// account_id = "another_account_id"
/// display_name = "Another User"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JiraUsersConfig {
    /// 用户列表
    #[serde(default)]
    users: Vec<JiraUserEntry>,
}

/// Jira 用户条目（TOML）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JiraUserEntry {
    /// 用户邮箱（必需，用于查找）
    email: String,
    /// 账户 ID
    account_id: String,
    /// 显示名称
    display_name: String,
}

/// Jira 用户管理
///
/// 提供用户信息获取和本地缓存功能：
/// - 从 Jira API 获取用户信息
/// - 本地缓存用户信息到 `${HOME}/.workflow/config/jira-users.toml`
/// - 优先使用本地缓存，减少 API 调用
pub struct JiraUsers;

impl JiraUsers {
    /// 从本地 TOML 文件读取用户信息
    ///
    /// 从 `jira-users.toml` 配置文件中读取指定邮箱的用户信息。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，如果文件不存在或用户不存在则返回错误。
    fn from_local(email: &str) -> Result<JiraUser> {
        let config_path = Paths::jira_users_config()?;

        if !config_path.exists() {
            anyhow::bail!("Jira users config file does not exist: {:?}", config_path);
        }

        let content = fs::read_to_string(&config_path)
            .context(format!("Failed to read jira-users.toml: {:?}", config_path))?;

        let config: JiraUsersConfig = toml::from_str(&content).context(format!(
            "Failed to parse jira-users.toml: {:?}",
            config_path
        ))?;

        // 查找匹配的用户
        let user_entry = config
            .users
            .iter()
            .find(|u| u.email == email)
            .context(format!(
                "User with email '{}' not found in jira-users.toml",
                email
            ))?;

        Ok(JiraUser {
            account_id: user_entry.account_id.clone(),
            display_name: user_entry.display_name.clone(),
            email_address: Some(user_entry.email.clone()),
        })
    }

    /// 从远程 API 获取用户信息并保存到本地
    ///
    /// 调用 Jira API 的 `/myself` 接口获取当前用户信息，
    /// 并将结果保存到本地缓存文件中。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    /// * `api_token` - Jira API token
    ///
    /// # 返回
    ///
    /// 返回 `JiraUser` 结构体，包含用户的完整信息。
    fn from_remote(email: &str, api_token: &str) -> Result<JiraUser> {
        let base_url = get_base_url()?;
        let url = format!("{}/myself", base_url);

        let client = HttpClient::global()?;
        let auth = Authorization::new(email, api_token);
        let config = RequestConfig::<Value, Value>::new().auth(&auth);
        let response = client
            .get(&url, config)
            .context("Failed to get current Jira user")?;

        if !response.is_success() {
            anyhow::bail!("Failed to get current Jira user: {}", response.status);
        }

        // 解析 JSON 响应
        let data: Value = response.as_json()?;

        // 尝试直接反序列化为 JiraUser
        let mut user: JiraUser =
            serde_json::from_value(data.clone()).context("Failed to parse Jira user response")?;

        // 确保 account_id 存在（如果没有，尝试使用 key 作为后备）
        if user.account_id.is_empty() {
            if let Some(key) = data.get("key").and_then(|k| k.as_str()) {
                user.account_id = key.to_string();
            } else {
                anyhow::bail!("Failed to extract accountId or key from Jira user response");
            }
        }

        // 保存到本地 TOML 文件
        Self::to_local(email, &user)?;

        Ok(user)
    }

    /// 保存用户信息到本地 TOML 文件
    ///
    /// 将用户信息添加到或更新到 `jira-users.toml` 配置文件中。
    ///
    /// # 参数
    ///
    /// * `email` - 用户邮箱地址
    /// * `user` - JiraUser 结构体
    fn to_local(email: &str, user: &JiraUser) -> Result<()> {
        let config_path = Paths::jira_users_config()?;

        // 读取现有配置
        let mut config: JiraUsersConfig = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read existing jira-users.toml")?;
            toml::from_str(&content).unwrap_or_else(|_| JiraUsersConfig { users: vec![] })
        } else {
            JiraUsersConfig { users: vec![] }
        };

        // 查找并更新或添加用户
        // 使用 JiraUser 的 email_address，如果没有则使用传入的 email
        let email_to_save = user.email_address.as_deref().unwrap_or(email);
        let email_to_save_str = email_to_save.to_string();
        let user_entry = JiraUserEntry {
            email: email_to_save_str.clone(),
            account_id: user.account_id.clone(),
            display_name: user.display_name.clone(),
        };

        // 查找时使用保存的 email（可能是 email_address 或传入的 email）
        if let Some(existing) = config
            .users
            .iter_mut()
            .find(|u| u.email == email_to_save_str || u.email == email)
        {
            // 更新现有用户
            *existing = user_entry;
        } else {
            // 添加新用户
            config.users.push(user_entry);
        }

        // 写入 TOML
        let toml_content =
            toml::to_string_pretty(&config).context("Failed to serialize jira-users.toml")?;

        fs::write(&config_path, toml_content).context(format!(
            "Failed to write jira-users.toml: {:?}",
            config_path
        ))?;

        // 设置文件权限为 600（仅用户可读写）
        #[cfg(unix)]
        {
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))
                .context("Failed to set jira-users.toml permissions")?;
        }

        Ok(())
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

        // 先尝试从本地文件读取
        if let Ok(user) = Self::from_local(&email) {
            return Ok(user);
        }

        // 本地文件不存在或读取失败，从 API 获取
        Self::from_remote(&email, &api_token)
    }
}
