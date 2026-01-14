//! GitHub 配置处理模块

use crate::commands::github::helpers::collect_github_account;
use crate::commands::setup::types::CollectedConfig;
use crate::config::settings::GitHubAccount;
use crate::services::git::GitConfig;
use crate::{br, info};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

/// 处理 GitHub 配置
pub fn handle_github_config(
    existing: &CollectedConfig,
) -> Result<(Vec<GitHubAccount>, Option<String>)> {
    // 显示标题
    br!();
    info!("  GitHub Configuration (Required)");
    br!('─', 65);

    // 显示检测信息（借鉴 Go 版本）
    if !existing.github_accounts.is_empty() {
        info!("The following GitHub accounts were detected:");
        for account in &existing.github_accounts {
            let is_current =
                existing.github_current.as_ref().map(|c| c == &account.name).unwrap_or(false);
            if is_current {
                info!("  - {} ({}) [current]", account.name, account.email);
            } else {
                info!("  - {} ({})", account.name, account.email);
            }
        }
    } else {
        info!("No GitHub accounts were detected.");
    }
    br!();

    let mut github_accounts = existing.github_accounts.clone();
    let mut github_current = existing.github_current.clone();

    // 如果已有账号，询问是否要管理账号
    if !github_accounts.is_empty() {
        // 获取当前账号的 email，用于显示
        let current_email = github_current
            .as_ref()
            .and_then(|current_name| {
                github_accounts
                    .iter()
                    .find(|a| &a.name == current_name)
                    .map(|a| a.email.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let keep_option = format!("Keep current accounts ({})", current_email);
        let options = vec!["Add new account".to_string(), keep_option];
        let selected_option = crate::select!("GitHub account management", options.clone())
            .default(1)
            .result_title("GitHub account management")
            .prompt()
            .wrap_err("Failed to get GitHub account management choice")?;
        let selection = options.iter().position(|opt| opt == &selected_option).unwrap_or(1);

        let mut account_added = false;
        match selection {
            0 => {
                // 添加新账号
                let account = collect_github_account()?;
                github_accounts.push(account);
                account_added = true;
                // 如果是第一个账号，自动设置为当前账号
                if github_accounts.len() == 1 {
                    let first_account = github_accounts
                        .first()
                        .ok_or_else(|| eyre!("Expected at least one GitHub account"))?;
                    github_current = Some(first_account.name.clone());
                    let _ = GitConfig::set_global_user(&first_account.email, &first_account.name)?;
                }
            }
            _ => {
                // 保持现有账号，但需要确保 Git 配置与当前账号一致
                if let Some(ref current_name) = github_current {
                    if let Some(current_account) =
                        github_accounts.iter().find(|a| &a.name == current_name)
                    {
                        let _ = GitConfig::set_global_user(
                            &current_account.email,
                            &current_account.name,
                        )?;
                    }
                }
            }
        }

        // 只有在添加了新账号后，如果有多个账号，才询问选择当前账号
        if account_added && github_accounts.len() > 1 {
            let account_names: Vec<String> =
                github_accounts.iter().map(|a| a.name.clone()).collect();
            let default_index = github_current
                .as_ref()
                .and_then(|current| account_names.iter().position(|n| n == current))
                .unwrap_or(0);

            let account_names_vec: Vec<String> = account_names.to_vec();
            let selected_account =
                crate::select!("Select current GitHub account", account_names_vec.clone())
                    .default(default_index)
                    .result_title("Current GitHub account")
                    .prompt()
                    .wrap_err("Failed to select current account")?;
            let selection = account_names_vec
                .iter()
                .position(|name| name == &selected_account)
                .unwrap_or(default_index);

            github_current = Some(account_names[selection].clone());
            let current_account = &github_accounts[selection];
            let _ = GitConfig::set_global_user(&current_account.email, &current_account.name)?;
        } else if github_accounts.len() == 1 {
            // 如果只有一个账号，确保设置了 Git 配置
            let account = &github_accounts[0];
            if github_current.as_ref().map(|c| c == &account.name).unwrap_or(false) {
                let _ = GitConfig::set_global_user(&account.email, &account.name)?;
            }
        }
    } else {
        // 没有账号，添加第一个账号
        info!("No GitHub accounts configured. Let's add one:");
        let account = collect_github_account()?;
        github_accounts.push(account);
        let first_account = github_accounts
            .first()
            .ok_or_else(|| eyre!("Expected at least one GitHub account"))?;
        github_current = Some(first_account.name.clone());
        let _ = GitConfig::set_global_user(&first_account.email, &first_account.name)?;
    }

    Ok((github_accounts, github_current))
}
