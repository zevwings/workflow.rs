//! GitHub 账号管理命令
//! 用于管理多个 GitHub 账号的配置

use crate::git::GitConfig;
use crate::settings::{
    paths::ConfigPaths,
    settings::{GitHubAccount, Settings},
};
use crate::{
    confirm, log_break, log_info, log_message, log_success, log_warning, mask_sensitive_value,
};
use anyhow::{Context, Result};
use dialoguer::{Input, Select};
use std::fs;
use toml;

/// GitHub 账号管理命令
pub struct GitHubCommand;

impl GitHubCommand {
    /// 列出所有 GitHub 账号
    pub fn list() -> Result<()> {
        log_break!('=', 40, "GitHub Accounts");
        log_break!();

        let settings = Settings::load();
        let github = &settings.github;

        if github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            log_message!("Run 'workflow github add' to add an account.");
            return Ok(());
        }

        log_info!("Total accounts: {}\n", github.accounts.len());

        for (index, account) in github.accounts.iter().enumerate() {
            let is_current = github
                .current
                .as_ref()
                .map(|c| c == &account.name)
                .unwrap_or_else(|| {
                    // 如果没有设置 current，第一个账号是当前账号
                    index == 0 && github.current.is_none()
                });

            let current_marker = if is_current { " (current)" } else { "" };
            log_break!(
                '-',
                40,
                &format!("Account {}: {}{}", index + 1, account.name, current_marker)
            );
            log_message!("  Name: {}", account.name);
            log_message!("  Email: {}", account.email);
            log_message!("  API Token: {}", mask_sensitive_value(&account.api_token));
            if let Some(ref prefix) = account.branch_prefix {
                log_message!("  Branch Prefix: {}", prefix);
            }
            log_break!();
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
            if let Some(ref prefix) = account.branch_prefix {
                log_message!("  Branch Prefix: {}", prefix);
            }
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

        let mut settings = Settings::load();
        let account = Self::collect_github_account()?;

        // 检查账号名称是否已存在
        if settings
            .github
            .accounts
            .iter()
            .any(|a| a.name == account.name)
        {
            return Err(anyhow::anyhow!(
                "Account with name '{}' already exists",
                account.name
            ));
        }

        settings.github.accounts.push(account.clone());

        // 如果这是第一个账号，自动设置为当前账号
        if settings.github.accounts.len() == 1 {
            settings.github.current = Some(account.name.clone());
            GitConfig::set_global_user(&account.email, &account.name)?;
            log_success!("Account '{}' added and set as current.", account.name);
        } else {
            log_success!("Account '{}' added.", account.name);
            log_break!();

            // 询问是否要将新账号设为当前账号
            let should_set_current = confirm(
                &format!("Set '{}' as the current GitHub account?", account.name),
                false,
                None,
            )?;

            if should_set_current {
                settings.github.current = Some(account.name.clone());
                GitConfig::set_global_user(&account.email, &account.name)?;
                log_success!("Account '{}' is now set as current.", account.name);
            } else {
                log_message!("Current account remains unchanged.");
            }
        }

        Self::save_settings(&settings)?;

        Ok(())
    }

    /// 删除 GitHub 账号
    pub fn remove() -> Result<()> {
        log_break!('=', 40, "Remove GitHub Account");
        log_break!();

        let mut settings = Settings::load();

        if settings.github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            return Ok(());
        }

        let account_names: Vec<String> = settings
            .github
            .accounts
            .iter()
            .map(|a| a.name.clone())
            .collect();

        // 找到当前账号的索引，作为默认选中项
        let default_index = settings
            .github
            .current
            .as_ref()
            .and_then(|current| account_names.iter().position(|n| n == current))
            .unwrap_or(0);

        let selection = Select::new()
            .with_prompt("Select account to remove")
            .items(&account_names)
            .default(default_index)
            .interact()
            .context("Failed to get account selection")?;

        let account_name = &account_names[selection];

        // 确认删除
        if !confirm(
            &format!(
                "Are you sure you want to remove account '{}'?",
                account_name
            ),
            false,
            None,
        )? {
            log_message!("Operation cancelled.");
            return Ok(());
        }

        // 如果删除的是当前账号，需要更新 current
        let was_current = settings
            .github
            .current
            .as_ref()
            .map(|c| c == account_name)
            .unwrap_or(false);

        settings.github.accounts.remove(selection);

        // 如果删除后还有账号
        if !settings.github.accounts.is_empty() {
            // 检查当前账号是否还在列表中
            let current_still_exists = settings
                .github
                .current
                .as_ref()
                .map(|current_name| {
                    settings
                        .github
                        .accounts
                        .iter()
                        .any(|a| &a.name == current_name)
                })
                .unwrap_or(false);

            // 如果删除的是当前账号，或者当前账号不在列表中，或者没有设置当前账号，则设置第一个账号为当前账号
            if was_current || !current_still_exists || settings.github.current.is_none() {
                let new_current = &settings.github.accounts[0];
                settings.github.current = Some(new_current.name.clone());
                GitConfig::set_global_user(&new_current.email, &new_current.name)?;
            }
        } else {
            // 如果没有账号了，清空 current
            settings.github.current = None;
        }

        log_success!("Account '{}' removed.", account_name);
        Self::save_settings(&settings)?;

        Ok(())
    }

    /// 切换当前 GitHub 账号
    pub fn switch() -> Result<()> {
        log_break!('=', 40, "Switch GitHub Account");
        log_break!();

        let mut settings = Settings::load();

        if settings.github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            log_message!("Run 'workflow github add' to add an account.");
            return Ok(());
        }

        if settings.github.accounts.len() == 1 {
            log_warning!("Only one account configured. No need to switch.");
            return Ok(());
        }

        let account_names: Vec<String> = settings
            .github
            .accounts
            .iter()
            .map(|a| a.name.clone())
            .collect();

        // 找到当前账号的索引，作为默认选中项
        let default_index = settings
            .github
            .current
            .as_ref()
            .and_then(|current| account_names.iter().position(|n| n == current))
            .unwrap_or(0);

        let selection = Select::new()
            .with_prompt("Select account to switch to")
            .items(&account_names)
            .default(default_index)
            .interact()
            .context("Failed to get account selection")?;

        let account_name = account_names[selection].clone();
        let account = &settings.github.accounts[selection];
        settings.github.current = Some(account_name.clone());

        GitConfig::set_global_user(&account.email, &account.name)?;
        log_success!("Switched to account '{}'.", account_name);
        Self::save_settings(&settings)?;

        Ok(())
    }

    /// 更新 GitHub 账号信息
    pub fn update() -> Result<()> {
        log_break!('=', 40, "Update GitHub Account");
        log_break!();

        let mut settings = Settings::load();

        if settings.github.accounts.is_empty() {
            log_warning!("No GitHub accounts configured.");
            log_message!("Run 'workflow github add' to add an account.");
            return Ok(());
        }

        let account_names: Vec<String> = settings
            .github
            .accounts
            .iter()
            .map(|a| a.name.clone())
            .collect();

        // 找到当前账号的索引，作为默认选中项
        let default_index = settings
            .github
            .current
            .as_ref()
            .and_then(|current| account_names.iter().position(|n| n == current))
            .unwrap_or(0);

        let selection = Select::new()
            .with_prompt("Select account to update")
            .items(&account_names)
            .default(default_index)
            .interact()
            .context("Failed to get account selection")?;

        let old_account = &settings.github.accounts[selection];
        log_info!("Current account information:");
        log_message!("  Name: {}", old_account.name);
        log_message!("  Email: {}", old_account.email);
        log_message!(
            "  API Token: {}",
            mask_sensitive_value(&old_account.api_token)
        );
        if let Some(ref prefix) = old_account.branch_prefix {
            log_message!("  Branch Prefix: {}", prefix);
        }
        log_break!();

        // 收集新的账号信息（使用现有值作为默认值）
        let new_account = Self::collect_github_account_with_defaults(old_account)?;

        // 如果账号名称改变了，检查新名称是否已存在（排除当前正在更新的账号）
        if new_account.name != old_account.name
            && settings
                .github
                .accounts
                .iter()
                .enumerate()
                .any(|(idx, a)| idx != selection && a.name == new_account.name)
        {
            return Err(anyhow::anyhow!(
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
        if new_account.name != old_account.name && is_current {
            settings.github.current = Some(new_account.name.clone());
        }

        // 如果更新的是当前账号，且 email 或 name 改变了，需要更新 git config
        let should_update_git_config = is_current
            && (new_account.email != old_account.email || new_account.name != old_account.name);

        settings.github.accounts[selection] = new_account.clone();

        // 更新 git config（在更新 accounts 之后）
        if should_update_git_config {
            let updated_account = &settings.github.accounts[selection];
            GitConfig::set_global_user(&updated_account.email, &updated_account.name)?;
        }

        log_success!("Account '{}' updated.", new_account.name);
        Self::save_settings(&settings)?;

        Ok(())
    }

    /// 收集 GitHub 账号信息
    fn collect_github_account() -> Result<GitHubAccount> {
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
    fn collect_github_account_with_defaults(old_account: &GitHubAccount) -> Result<GitHubAccount> {
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

    /// 保存设置到 TOML 文件
    fn save_settings(settings: &Settings) -> Result<()> {
        let workflow_config_path = ConfigPaths::workflow_config()?;
        let toml_content =
            toml::to_string_pretty(settings).context("Failed to serialize settings to TOML")?;
        fs::write(&workflow_config_path, toml_content).context("Failed to write workflow.toml")?;

        Ok(())
    }
}
