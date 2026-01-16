//! GitHub 配置处理模块

use crate::config::settings::GitHubAccount;
use crate::core::prompt::{FormBuilder, GroupConfig, InputFormField, SelectFormField};
use crate::services::git::GitConfig;
use crate::settings::GitHubSettings;
use crate::{br, info, print};
use color_eyre::{eyre::WrapErr, Result};
use std::sync::Arc;

/// 处理 GitHub 配置
///
/// 参考 Go 版本的实现逻辑：
/// 1. 检测现有账户（信息展示在表单外）
/// 2. 如果已有账户，提供三个选项：保持当前账户、使用其他已有账户（切换）、添加新账户
/// 3. 如果选择保持，直接返回
/// 4. 如果选择切换，让用户选择要切换到的账户
/// 5. 如果选择添加，添加新账户，直接使用新账号作为当前账号
///
/// 使用 FormBuilder 处理交互式输入，信息展示和后处理逻辑在表单外
pub fn handle_github_config(existing: GitHubSettings) -> Result<GitHubSettings> {
    // 显示检测信息（信息展示 - 表单外）
    if !existing.accounts.is_empty() {
        info!("The following GitHub accounts were detected.");
        for account in &existing.accounts {
            let is_current = existing.current.as_ref().map(|c| c == &account.name).unwrap_or(false);
            if is_current {
                print!("  - {} ({}) [current]", account.name, account.email);
            } else {
                print!("  - {} ({})", account.name, account.email);
            }
        }
    }
    br!();

    // 如果已有账号，使用 FormBuilder 处理交互式输入
    if !existing.accounts.is_empty() {
        let form_result = build_github_management_form(&existing)
            .run()
            .wrap_err("Failed to collect GitHub configuration")?;

        // 解析表单结果并处理（后处理逻辑 - 表单外）
        let (accounts, current) = parse_management_result(form_result, &existing)?;

        // 同步 Git 配置（后处理逻辑 - 表单外）
        sync_git_config(&accounts, &current)?;

        Ok(GitHubSettings { accounts, current })
    } else {
        // 没有账号，使用 FormBuilder 收集第一个账号信息
        let form_result = build_add_account_form()
            .run()
            .wrap_err("Failed to collect GitHub account information")?;

        // 解析表单结果（后处理逻辑 - 表单外）
        let account = parse_account_form_result(form_result)?;

        // 账户名为空则使用 "default"
        let account_name = if account.name.trim().is_empty() {
            "default".to_string()
        } else {
            account.name.clone()
        };

        let account = GitHubAccount {
            name: account_name.clone(),
            ..account
        };

        let accounts = vec![account];
        let current = Some(account_name.clone());

        // 同步 Git 配置（后处理逻辑 - 表单外）
        if let Some(current_account) = accounts.first() {
            let _ = GitConfig::set_global_user(&current_account.email, &current_account.name)?;
        }

        Ok(GitHubSettings { accounts, current })
    }
}

/// 构建 GitHub 管理表单（交互式输入 - FormBuilder）
///
/// 处理已有账户的情况：
/// - 直接列出所有账户选项：保持当前账户、使用其他账户（Use xxx）、添加新账户
/// - 根据选择执行相应操作
fn build_github_management_form(existing: &GitHubSettings) -> FormBuilder {
    // 获取当前账号的 email，用于显示
    let current_email = existing
        .current
        .as_ref()
        .and_then(|current_name| {
            existing
                .accounts
                .iter()
                .find(|a| &a.name == current_name)
                .map(|a| a.email.clone())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // 构建选项列表：
    // 1. Keep current account (email)
    // 2. Use account1 (email)
    // 3. Use account2 (email)
    // ...
    // n. Add new account
    let mut options = Vec::new();

    // 第一个选项：保持当前账户
    options.push(format!("Keep current account ({})", current_email));

    let other_accounts: Vec<_> = existing
        .accounts
        .iter()
        .filter(|a| existing.current.as_ref().map(|current| &a.name != current).unwrap_or(true))
        .collect();

    for account in &other_accounts {
        options.push(format!("Use {}({})", account.name, account.email));
    }

    // 最后一个选项：添加新账户
    options.push("Add new account".to_string());

    // 默认选择第一个（保持当前账户）
    let default_idx = 0;

    FormBuilder::new().add_group(
        "github_management",
        |g| {
            // 第一步：选择操作或账户
            g.add_step(|s| {
                s.add_select(
                    SelectFormField::new("github_action", "GitHub account management", options)
                        .default(default_idx)
                        .result_title("GitHub account management"),
                )
            })
            // 如果选择添加新账户
            .add_step_if(
                move |result| {
                    if let Some(action) = result.get("github_action") {
                        action == "Add new account"
                    } else {
                        false
                    }
                },
                |s| {
                    s.add_input(
                        InputFormField::new(
                            "github_account_name",
                            "Please enter your GitHub account name",
                        )
                        .required()
                        .result_title("Your GitHub account name"),
                    )
                    .add_input(
                        InputFormField::new(
                            "github_account_email",
                            "Please enter your GitHub account email",
                        )
                        .required()
                        .validator(Arc::new(move |input: &str| {
                            if input.trim().is_empty() {
                                Err("Email is required and cannot be empty".to_string())
                            } else if !input.contains('@') {
                                Err("Please enter a valid email address".to_string())
                            } else {
                                Ok(())
                            }
                        }))
                        .result_title("Your GitHub account email"),
                    )
                    .add_input(
                        InputFormField::new(
                            "github_account_token",
                            "Please enter your GitHub API token",
                        )
                        .required()
                        .result_title("Your GitHub API token"),
                    )
                },
            )
        },
        GroupConfig::required().with_title("GitHub Configuration (Required)"),
    )
}

/// 构建添加账号表单（交互式输入 - FormBuilder）
///
/// 处理没有账户的情况，直接收集账号信息
fn build_add_account_form() -> FormBuilder {
    FormBuilder::new().add_group(
        "github_add_account",
        |g| {
            g.add_step(|s| {
                s.add_input(
                    InputFormField::new(
                        "github_account_name",
                        "Please enter your GitHub account name",
                    )
                    .required()
                    .result_title("Your GitHub account name"),
                )
                .add_input(
                    InputFormField::new(
                        "github_account_email",
                        "Please enter your GitHub account email",
                    )
                    .required()
                    .validator(Arc::new(move |input: &str| {
                        if input.trim().is_empty() {
                            Err("Email is required and cannot be empty".to_string())
                        } else if !input.contains('@') {
                            Err("Please enter a valid email address".to_string())
                        } else {
                            Ok(())
                        }
                    }))
                    .result_title("Your GitHub account email"),
                )
                .add_input(
                    InputFormField::new(
                        "github_account_token",
                        "Please enter your GitHub API token",
                    )
                    .required()
                    .result_title("Your GitHub API token"),
                )
            })
        },
        GroupConfig::required().with_title("GitHub Configuration (Required)"),
    )
}

/// 解析管理表单结果（后处理逻辑 - 表单外）
fn parse_management_result(
    form_result: crate::prompt::FormResult,
    existing: &GitHubSettings,
) -> Result<(Vec<GitHubAccount>, Option<String>)> {
    let mut accounts = existing.accounts.clone();
    let mut current = existing.current.clone();

    let action = form_result
        .get("github_action")
        .ok_or_else(|| color_eyre::eyre::eyre!("GitHub action not found in form result"))?;

    // 检查是否是"保持当前账户"选项（选项文本以 "Keep current account" 开头）
    if action.starts_with("Keep current account") {
        // 保持当前账户，直接返回
        return Ok((accounts, current));
    } else if action.starts_with("Use ") {
        // 切换到其他已有账户（格式：Use account_name(email)）
        // 从 "Use account_name(email)" 中提取 account_name
        let account_name = action
            .strip_prefix("Use ")
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid account selection format"))?
            .split('(')
            .next()
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid account selection format"))?
            .trim()
            .to_string();

        // 验证账户是否存在
        if !accounts.iter().any(|a| a.name == account_name) {
            return Err(color_eyre::eyre::eyre!(
                "Selected account '{}' not found",
                account_name
            ));
        }

        current = Some(account_name);
        return Ok((accounts, current));
    } else if action == "Add new account" {
        // 添加新账号
        let account = parse_account_form_result(form_result)?;
        let account_name = account.name.clone();

        // 检查账户名是否已存在，如果存在则更新，否则新增
        let is_new_account =
            if let Some(existing_account) = accounts.iter_mut().find(|a| a.name == account_name) {
                // 更新现有账户
                *existing_account = account;
                false
            } else {
                // 新增账户
                accounts.push(account);
                true
            };

        // 如果是新账号，直接设置为当前账号
        if is_new_account {
            current = Some(account_name);
        } else if current.is_none() {
            // 如果是更新已有账号且当前账户为空，设置为该账户名
            current = Some(account_name);
        }
    }

    Ok((accounts, current))
}

/// 解析账号表单结果（后处理逻辑 - 表单外）
fn parse_account_form_result(form_result: crate::prompt::FormResult) -> Result<GitHubAccount> {
    let name = form_result
        .get("github_account_name")
        .ok_or_else(|| color_eyre::eyre::eyre!("Account name not found in form result"))?
        .trim()
        .to_string();

    let email = form_result
        .get("github_account_email")
        .ok_or_else(|| color_eyre::eyre::eyre!("Account email not found in form result"))?
        .trim()
        .to_string();

    let api_token = form_result
        .get("github_account_token")
        .ok_or_else(|| color_eyre::eyre::eyre!("API token not found in form result"))?
        .trim()
        .to_string();

    Ok(GitHubAccount {
        name,
        email,
        api_token,
    })
}

/// 同步 Git 配置（后处理逻辑 - 表单外）
fn sync_git_config(accounts: &[GitHubAccount], current: &Option<String>) -> Result<()> {
    if let Some(ref current_name) = current {
        if let Some(current_account) = accounts.iter().find(|a| &a.name == current_name) {
            let _ = GitConfig::set_global_user(&current_account.email, &current_account.name)?;
        }
    }
    Ok(())
}
