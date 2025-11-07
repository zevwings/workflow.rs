//! Jira 用户相关 API
//!
//! 本模块提供了用户信息获取和本地缓存功能：
//! - 从 Jira API 获取用户信息
//! - 本地缓存用户信息到 `${HOME}/.workflow/users/${email}.json`
//! - 优先使用本地缓存，减少 API 调用

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::http::{Authorization, HttpClient};

use super::helpers::{get_auth, get_base_url, sanitize_email_for_filename};
use super::models::JiraUser;

/// 获取用户信息文件路径
///
/// 根据邮箱地址生成用户信息缓存文件的路径。
/// 路径格式：`${HOME}/.workflow/users/${sanitized_email}.json`
///
/// # 参数
///
/// * `email` - 用户邮箱地址
///
/// # 返回
///
/// 返回用户信息文件的完整路径。
fn get_user_info_file_path(email: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let home_dir = PathBuf::from(&home);
    let users_dir = home_dir.join(".workflow").join("users");

    // 确保目录存在
    fs::create_dir_all(&users_dir)
        .context("Failed to create .workflow/users directory")?;

    let sanitized_email = sanitize_email_for_filename(email);
    Ok(users_dir.join(format!("{}.json", sanitized_email)))
}

/// 从本地文件读取用户信息
///
/// 从缓存文件中读取用户信息。如果文件不存在，返回错误。
///
/// # 参数
///
/// * `email` - 用户邮箱地址
///
/// # 返回
///
/// 返回 `JiraUser` 结构体，如果文件不存在则返回错误。
fn get_user_info_from_local(email: &str) -> Result<JiraUser> {
    let file_path = get_user_info_file_path(email)?;

    if !file_path.exists() {
        anyhow::bail!("User info file does not exist: {:?}", file_path);
    }

    let file_content = fs::read_to_string(&file_path)
        .context(format!("Failed to read user info file: {:?}", file_path))?;

    let user: JiraUser = serde_json::from_str(&file_content)
        .context(format!("Failed to parse user info from file: {:?}", file_path))?;

    Ok(user)
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
fn get_user_info_from_remote(email: &str, api_token: &str) -> Result<JiraUser> {
    let base_url = get_base_url()?;
    let url = format!("{}/myself", base_url);

    let client: HttpClient = HttpClient::new()?;
    let auth = Authorization::new(email, api_token);
    let response = client
        .get::<serde_json::Value>(&url, Some(&auth), None)
        .context("Failed to get current Jira user")?;

    if !response.is_success() {
        anyhow::bail!("Failed to get current Jira user: {}", response.status);
    }

    // 尝试直接反序列化为 JiraUser
    let mut user: JiraUser = serde_json::from_value(response.data.clone())
        .context("Failed to parse Jira user response")?;

    // 确保 account_id 存在（如果没有，尝试使用 key 作为后备）
    if user.account_id.is_empty() {
        if let Some(key) = response.data.get("key").and_then(|k| k.as_str()) {
            user.account_id = key.to_string();
        } else {
            anyhow::bail!("Failed to extract accountId or key from Jira user response");
        }
    }

    // 保存到本地文件
    let file_path = get_user_info_file_path(email)?;
    let json_content = serde_json::to_string_pretty(&user)
        .context("Failed to serialize user info to JSON")?;
    fs::write(&file_path, json_content)
        .context(format!("Failed to write user info to file: {:?}", file_path))?;

    Ok(user)
}

/// 获取当前 Jira 用户信息
///
/// 优先从本地缓存读取用户信息，如果缓存不存在或读取失败，
/// 则从 Jira API 获取并保存到本地缓存。
///
/// # 返回
///
/// 返回 `JiraUser` 结构体，包含用户的 `account_id`、`display_name` 和 `email_address`。
pub fn get_user_info() -> Result<JiraUser> {
    let (email, api_token) = get_auth()?;

    // 先尝试从本地文件读取
    if let Ok(user) = get_user_info_from_local(&email) {
        return Ok(user);
    }

    // 本地文件不存在或读取失败，从 API 获取
    get_user_info_from_remote(&email, &api_token)
}

