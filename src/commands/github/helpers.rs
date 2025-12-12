//! GitHub 命令辅助函数
//!
//! 提供 GitHub 账号管理的共享逻辑，减少代码重复。

use crate::base::dialog::InputDialog;
use crate::base::settings::settings::GitHubAccount;
use color_eyre::{eyre::WrapErr, Result};

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
    let name = InputDialog::new("GitHub account name")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("Account name is required and cannot be empty".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get GitHub account name")?;

    let email = InputDialog::new("GitHub account email")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("Email is required and cannot be empty".to_string())
            } else if !input.contains('@') {
                Err("Please enter a valid email address".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get GitHub account email")?;

    let api_token = InputDialog::new("GitHub API token")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("GitHub API token is required and cannot be empty".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get GitHub API token")?;

    Ok(GitHubAccount {
        name: name.trim().to_string(),
        email: email.trim().to_string(),
        api_token: api_token.trim().to_string(),
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
    let name = InputDialog::new("GitHub account name")
        .with_default(old_account.name.clone())
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("Account name is required and cannot be empty".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get GitHub account name")?;

    let email = InputDialog::new("GitHub account email")
        .with_default(old_account.email.clone())
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("Email is required and cannot be empty".to_string())
            } else if !input.contains('@') {
                Err("Please enter a valid email address".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get GitHub account email")?;

    let api_token = InputDialog::new("GitHub API token")
        .with_default(old_account.api_token.clone())
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("GitHub API token is required and cannot be empty".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get GitHub API token")?;

    Ok(GitHubAccount {
        name: name.trim().to_string(),
        email: email.trim().to_string(),
        api_token: api_token.trim().to_string(),
    })
}
