//! GitHub 账号管理命令
//! 用于管理多个 GitHub 账号的配置

use crate::base::dialog::{ConfirmDialog, SelectDialog};
use crate::base::indicator::Spinner;
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::base::settings::table::GitHubAccountListRow;
use crate::base::util::{
    mask_sensitive_value,
    table::{TableBuilder, TableStyle},
};
use crate::commands::github::helpers::{
    collect_github_account, collect_github_account_with_defaults,
};
use crate::git::GitConfig;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

/// GitHub 账号管理命令
pub struct GitHubCommand;

impl GitHubCommand {
    /// 列出所有 GitHub 账号（带验证）
    pub fn list() -> Result<()> {
        let settings = Settings::load();

        // 创建 spinner 显示验证进度
        let spinner = Spinner::new("Verifying GitHub accounts...");

        // 使用 verify_github() 获取验证结果
        let verification_result = settings.verify_github()?;

        // 完成 spinner
        spinner.finish();

        if !verification_result.configured || verification_result.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            log_message!("Run 'workflow github add' to add an account.");
            return Ok(());
        }

        // 构建表格数据（使用验证结果中的账号信息）
        let rows: Vec<GitHubAccountListRow> = verification_result
            .accounts
            .iter()
            .enumerate()
            .map(|(index, account)| GitHubAccountListRow {
                index: (index + 1).to_string(),
                name: account.name.clone(),
                email: account.email.clone(),
                token: account.token.clone(),
                status: if account.is_current {
                    "Current".to_string()
                } else {
                    "".to_string()
                },
            })
            .collect();

        // 使用表格显示
        log_message!(
            "{}",
            TableBuilder::new(rows)
                .with_title("GitHub Accounts")
                .with_style(TableStyle::Modern)
                .render()
        );

        // 显示验证状态
        log_break!();
        log_info!("Verification Status:");
        for account in &verification_result.accounts {
            if account.verification_status == "Success" {
                log_success!("  {}: {}", account.name, account.verification_status);
            } else {
                log_warning!("  {}: {}", account.name, account.verification_status);
                if let Some(ref error) = account.verification_error {
                    log_message!("    Error: {}", error);
                }
            }
        }

        // 显示总结
        let summary = &verification_result.summary;
        log_break!();
        log_info!("Total accounts: {}", summary.total_count);
        if summary.success_count == summary.total_count {
            log_success!("All accounts verified successfully!");
        } else {
            log_warning!(
                "Verification: {}/{} account(s) verified successfully",
                summary.success_count,
                summary.total_count
            );
        }

        Ok(())
    }

    /// 显示当前激活的 GitHub 账号
    pub fn current() -> Result<()> {
        log_break!('=', 40, "Current GitHub Account");
        log_break!();

        let settings = Settings::load();
        let github = &settings.github;

        if let Some(account) = github.get_current_account() {
            log_success!("Current account: {}", account.name);
            log_message!("  Email: {}", account.email);
            log_message!("  API Token: {}", mask_sensitive_value(&account.api_token));
        } else {
            log_warning!("No GitHub account is currently active.");
            log_message!("Run 'workflow github add' to add an account.");
        }

        Ok(())
    }

    /// 添加新的 GitHub 账号
    pub fn add() -> Result<()> {
        log_break!('=', 40, "Add GitHub Account");
        log_break!();

        let account = collect_github_account()?;

        // 先检查账号名称是否已存在
        let settings = Settings::load();
        if settings.github.accounts.iter().any(|a| a.name == account.name) {
            return Err(eyre!("Account with name '{}' already exists", account.name));
        }

        let is_first_account = settings.github.accounts.is_empty();
        let account_name = account.name.clone();
        let account_email = account.email.clone();

        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.github.accounts.push(account.clone());

            // 如果这是第一个账号，自动设置为当前账号
            if is_first_account {
                settings.github.current = Some(account_name.clone());
            }
        })?;

        // 如果这是第一个账号，自动设置为当前账号
        if is_first_account {
            let result = GitConfig::set_global_user(&account_email, &account_name)?;
            log_info!("Git global config updated:");
            log_message!("  user.email: {}", result.email);
            log_message!("  user.name: {}", result.name);
            log_success!("Account '{}' added and set as current.", account_name);
        } else {
            log_success!("Account '{}' added.", account_name);
            log_break!();

            // 询问是否要将新账号设为当前账号
            let should_set_current = ConfirmDialog::new(format!(
                "Set '{}' as the current GitHub account?",
                account_name
            ))
            .with_default(false)
            .prompt()?;

            if should_set_current {
                let config_path = Paths::workflow_config()?;
                let manager = ConfigManager::<Settings>::new(config_path);
                manager.update(|settings| {
                    settings.github.current = Some(account_name.clone());
                })?;
                let result = GitConfig::set_global_user(&account_email, &account_name)?;
                log_info!("Git global config updated:");
                log_message!("  user.email: {}", result.email);
                log_message!("  user.name: {}", result.name);
                log_success!("Account '{}' is now set as current.", account_name);
            } else {
                log_message!("Current account remains unchanged.");
            }
        }

        Ok(())
    }

    /// 删除 GitHub 账号
    pub fn remove() -> Result<()> {
        log_break!('=', 40, "Remove GitHub Account");
        log_break!();

        let settings = Settings::load();

        if settings.github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            return Ok(());
        }

        let account_names: Vec<String> =
            settings.github.accounts.iter().map(|a| a.name.clone()).collect();

        // 找到当前账号的索引，作为默认选中项
        let default_index = settings
            .github
            .current
            .as_ref()
            .and_then(|current| account_names.iter().position(|n| n == current))
            .unwrap_or(0);

        let account_names_vec: Vec<String> = account_names.to_vec();
        let selected_account =
            SelectDialog::new("Select account to remove", account_names_vec.clone())
                .with_default(default_index)
                .prompt()
                .wrap_err("Failed to get account selection")?;
        let selection = account_names_vec
            .iter()
            .position(|name| name == &selected_account)
            .unwrap_or(default_index);

        let account_name = account_names[selection].clone();

        // 确认删除
        if !ConfirmDialog::new(format!(
            "Are you sure you want to remove account '{}'?",
            account_name
        ))
        .with_default(false)
        .prompt()?
        {
            log_message!("Operation cancelled.");
            return Ok(());
        }

        // 如果删除的是当前账号，需要更新 current
        let was_current =
            settings.github.current.as_ref().map(|c| c == &account_name).unwrap_or(false);

        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.github.accounts.remove(selection);

            // 如果删除后还有账号
            if !settings.github.accounts.is_empty() {
                // 检查当前账号是否还在列表中
                let current_still_exists = settings
                    .github
                    .current
                    .as_ref()
                    .map(|current_name| {
                        settings.github.accounts.iter().any(|a| &a.name == current_name)
                    })
                    .unwrap_or(false);

                // 如果删除的是当前账号，或者当前账号不在列表中，或者没有设置当前账号，则设置第一个账号为当前账号
                if was_current || !current_still_exists || settings.github.current.is_none() {
                    let new_current = &settings.github.accounts[0];
                    settings.github.current = Some(new_current.name.clone());
                }
            } else {
                // 如果没有账号了，清空 current
                settings.github.current = None;
            }
        })?;

        // 如果删除后还有账号，且需要更新 git config
        let settings_after = Settings::load();
        if !settings_after.github.accounts.is_empty() {
            let should_update_git = was_current
                || settings
                    .github
                    .current
                    .as_ref()
                    .map(|current_name| {
                        !settings_after.github.accounts.iter().any(|a| &a.name == current_name)
                    })
                    .unwrap_or(true);

            if should_update_git {
                if let Some(current_name) = &settings_after.github.current {
                    if let Some(current_account) =
                        settings_after.github.accounts.iter().find(|a| &a.name == current_name)
                    {
                        let result = GitConfig::set_global_user(
                            &current_account.email,
                            &current_account.name,
                        )?;
                        log_info!("Git global config updated:");
                        log_message!("  user.email: {}", result.email);
                        log_message!("  user.name: {}", result.name);
                    }
                }
            }
        }

        log_success!("Account '{}' removed.", account_name);

        Ok(())
    }

    /// 切换当前 GitHub 账号
    pub fn switch() -> Result<()> {
        log_break!('=', 40, "Switch GitHub Account");
        log_break!();

        let settings = Settings::load();

        if settings.github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            log_message!("Run 'workflow github add' to add an account.");
            return Ok(());
        }

        if settings.github.accounts.len() == 1 {
            log_warning!("Only one account configured. No need to switch.");
            return Ok(());
        }

        let account_names: Vec<String> =
            settings.github.accounts.iter().map(|a| a.name.clone()).collect();

        // 找到当前账号的索引，作为默认选中项
        let default_index = settings
            .github
            .current
            .as_ref()
            .and_then(|current| account_names.iter().position(|n| n == current))
            .unwrap_or(0);

        let account_names_vec: Vec<String> = account_names.to_vec();
        let selected_account =
            SelectDialog::new("Select account to switch to", account_names_vec.clone())
                .with_default(default_index)
                .prompt()
                .wrap_err("Failed to get account selection")?;
        let selection = account_names_vec
            .iter()
            .position(|name| name == &selected_account)
            .unwrap_or(default_index);

        let account_name = account_names[selection].clone();
        let account = &settings.github.accounts[selection];

        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.github.current = Some(account_name.clone());
        })?;

        let result = GitConfig::set_global_user(&account.email, &account.name)?;
        log_info!("Git global config updated:");
        log_message!("  user.email: {}", result.email);
        log_message!("  user.name: {}", result.name);
        log_success!("Switched to account '{}'.", account_name);

        Ok(())
    }

    /// 更新 GitHub 账号信息
    pub fn update() -> Result<()> {
        log_break!('=', 40, "Update GitHub Account");
        log_break!();

        let settings = Settings::load();

        if settings.github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            log_message!("Run 'workflow github add' to add an account.");
            return Ok(());
        }

        let account_names: Vec<String> =
            settings.github.accounts.iter().map(|a| a.name.clone()).collect();

        // 找到当前账号的索引，作为默认选中项
        let default_index = settings
            .github
            .current
            .as_ref()
            .and_then(|current| account_names.iter().position(|n| n == current))
            .unwrap_or(0);

        let account_names_vec: Vec<String> = account_names.to_vec();
        let selected_account =
            SelectDialog::new("Select account to update", account_names_vec.clone())
                .with_default(default_index)
                .prompt()
                .wrap_err("Failed to get account selection")?;
        let selection = account_names_vec
            .iter()
            .position(|name| name == &selected_account)
            .unwrap_or(default_index);

        let old_account = &settings.github.accounts[selection];
        log_info!("Current account information:");
        log_message!("  Name: {}", old_account.name);
        log_message!("  Email: {}", old_account.email);
        log_message!(
            "  API Token: {}",
            mask_sensitive_value(&old_account.api_token)
        );
        log_break!();

        // 收集新的账号信息（使用现有值作为默认值）
        let new_account = collect_github_account_with_defaults(old_account)?;

        // 如果账号名称改变了，检查新名称是否已存在（排除当前正在更新的账号）
        if new_account.name != old_account.name
            && settings
                .github
                .accounts
                .iter()
                .enumerate()
                .any(|(idx, a)| idx != selection && a.name == new_account.name)
        {
            return Err(eyre!(
                "Account with name '{}' already exists",
                new_account.name
            ));
        }

        // 检查这个账号是否是当前账号
        let is_current = settings
            .github
            .current
            .as_ref()
            .map(|c| c == &old_account.name)
            .unwrap_or(false);

        // 如果账号名称改变了，且这个账号是当前账号，需要更新 current 字段
        let name_changed = new_account.name != old_account.name;
        let should_update_current = name_changed && is_current;

        // 如果更新的是当前账号，且 email 或 name 改变了，需要更新 git config
        let should_update_git_config = is_current
            && (new_account.email != old_account.email || new_account.name != old_account.name);

        let new_account_name = new_account.name.clone();
        let new_account_email = new_account.email.clone();

        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.github.accounts[selection] = new_account.clone();

            // 如果账号名称改变了，且这个账号是当前账号，需要更新 current 字段
            if should_update_current {
                settings.github.current = Some(new_account_name.clone());
            }
        })?;

        // 更新 git config（在更新 accounts 之后）
        if should_update_git_config {
            let result = GitConfig::set_global_user(&new_account_email, &new_account_name)?;
            log_info!("Git global config updated:");
            log_message!("  user.email: {}", result.email);
            log_message!("  user.name: {}", result.name);
        }

        log_success!("Account '{}' updated.", new_account_name);

        Ok(())
    }
}
