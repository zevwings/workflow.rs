//! GitHub 命令辅助函数
//!
//! 提供 GitHub 账号管理的共享逻辑，减少代码重复。

use crate::base::settings::settings::GitHubAccount;
use anyhow::{Context, Result};
use dialoguer::Input;

/// 收集 GitHub 账号信息
///
/// 通过交互式输入收集 GitHub 账号的以下信息：
/// - 账号名称（必填）
/// - 邮箱（必填，需包含 @）
/// - API Token（必填）
/// - 分支前缀（可选）
///
/// # 返回
///
/// 返回收集到的 `GitHubAccount` 结构体
///
/// # 错误
///
/// 如果用户输入验证失败或输入过程中出错，返回错误
pub fn collect_github_account() -> Result<GitHubAccount> {
    let name: String = Input::new()
        .with_prompt("GitHub account name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Account name is required and cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to get GitHub account name")?;

    let email: String = Input::new()
        .with_prompt("GitHub account email")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Email is required and cannot be empty")
            } else if !input.contains('@') {
                Err("Please enter a valid email address")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to get GitHub account email")?;

    let api_token: String = Input::new()
        .with_prompt("GitHub API token")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("GitHub API token is required and cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to get GitHub API token")?;

    let branch_prefix: String = Input::new()
        .with_prompt("GitHub branch prefix (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to get GitHub branch prefix")?;

    Ok(GitHubAccount {
        name: name.trim().to_string(),
        email: email.trim().to_string(),
        api_token: api_token.trim().to_string(),
        branch_prefix: if branch_prefix.trim().is_empty() {
            None
        } else {
            Some(branch_prefix.trim().to_string())
        },
    })
}

/// 收集 GitHub 账号信息（使用现有值作为默认值）
///
/// 与 `collect_github_account()` 类似，但使用 `old_account` 的值作为默认值，
/// 用户可以直接按 Enter 保留现有值。
///
/// # 参数
///
/// * `old_account` - 现有的 GitHub 账号信息，用作默认值
///
/// # 返回
///
/// 返回收集到的 `GitHubAccount` 结构体
///
/// # 错误
///
/// 如果用户输入验证失败或输入过程中出错，返回错误
pub fn collect_github_account_with_defaults(old_account: &GitHubAccount) -> Result<GitHubAccount> {
    let name: String = Input::new()
        .with_prompt("GitHub account name")
        .default(old_account.name.clone())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Account name is required and cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to get GitHub account name")?;

    let email: String = Input::new()
        .with_prompt("GitHub account email")
        .default(old_account.email.clone())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Email is required and cannot be empty")
            } else if !input.contains('@') {
                Err("Please enter a valid email address")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to get GitHub account email")?;

    let api_token: String = Input::new()
        .with_prompt("GitHub API token")
        .default(old_account.api_token.clone())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("GitHub API token is required and cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("Failed to get GitHub API token")?;

    let branch_prefix: String = Input::new()
        .with_prompt("GitHub branch prefix (optional, press Enter to skip)")
        .default(
            old_account
                .branch_prefix
                .as_deref()
                .unwrap_or("")
                .to_string(),
        )
        .allow_empty(true)
        .interact_text()
        .context("Failed to get GitHub branch prefix")?;

    Ok(GitHubAccount {
        name: name.trim().to_string(),
        email: email.trim().to_string(),
        api_token: api_token.trim().to_string(),
        branch_prefix: if branch_prefix.trim().is_empty() {
            None
        } else {
            Some(branch_prefix.trim().to_string())
        },
    })
}
