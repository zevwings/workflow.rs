//! 账户操作函数

use domain::GlobalConfig;
use prompt::{br, confirm, info, success, warning, SelectBuilder};

use crate::workflows::core::context::{WorkflowContext, WorkflowMode};

use super::traits::{
    GlobalConfigAccessor, PlatformAccount, PlatformConfigurator, PlatformSettings,
};
use super::types::AccountSetMode;

/// 添加账户的通用逻辑
pub fn add_account_generic<S, A, F>(
    context: &mut WorkflowContext,
    account_creator: F,
    set_mode: AccountSetMode,
    platform_name: &str,
    verify_fn: Option<fn() -> Result<(), String>>,
) -> Result<String, String>
where
    GlobalConfig: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
    F: FnOnce() -> Result<A, String>,
{
    let account = account_creator()?;
    let account_name = account.name().to_string();

    let settings = context.settings_mut().get_settings_mut();
    if settings.account_exists(&account_name) {
        return Err(format!(
            "Account '{}' already exists. Please use 'Update {} account information' to modify it.",
            account_name, platform_name
        ));
    }

    settings.accounts_mut().push(account);

    if set_mode.should_set_current() || settings.current().is_empty() {
        settings.set_current(account_name.clone());
    }

    if context.mode() == WorkflowMode::Command {
        context.save().map_err(|e| format!("Failed to save config: {}", e))?;

        br!();
        success!(
            "{} account '{}' added successfully.",
            platform_name,
            account_name
        );

        if let Some(verify) = verify_fn {
            br!();
            if let Err(err) = verify() {
                warning!("Failed to verify {} account: {}", platform_name, err);
            }
        }
    } else {
        br!();
        success!(
            "{} account '{}' added successfully.",
            platform_name,
            account_name
        );
    }

    Ok(account_name)
}

/// 切换账户的通用逻辑
pub fn switch_account_generic<S, A>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
) -> Result<(), String>
where
    GlobalConfig: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
{
    let settings = context.settings().get_settings();
    if !settings.has_accounts() {
        return Err(format!(
            "No {} accounts available to switch",
            configurator.platform_name()
        ));
    }

    br!();
    info!(
        "Switching current {} account...",
        configurator.platform_name()
    );
    br!();

    let account_options: Vec<String> = settings
        .accounts()
        .iter()
        .map(|acc| acc.display_with_marker(settings.current() == acc.name()))
        .collect();

    let default_index = settings
        .accounts()
        .iter()
        .position(|acc| acc.name() == settings.current())
        .unwrap_or(0);

    let selected_account = SelectBuilder::new(
        format!(
            "Please select the {} account to switch to",
            configurator.platform_name()
        ),
        account_options,
    )
    .default(default_index)
    .result_title("Account to switch to")
    .prompt()
    .map_err(|e| e.to_string())?;

    let account_name = selected_account
        .split(' ')
        .next()
        .ok_or_else(|| "Failed to parse account name".to_string())?
        .to_string();

    let settings = context.settings_mut().get_settings_mut();
    if !settings.account_exists(&account_name) {
        return Err(format!("Account '{}' not found", account_name));
    }

    settings.set_current(account_name.clone());

    if context.mode() == WorkflowMode::Command {
        context.save().map_err(|e| format!("Failed to save config: {}", e))?;

        br!();
        success!(
            "Switched to {} account: {}",
            configurator.platform_name(),
            account_name
        );

        if configurator.auto_verify_in_command_setup() {
            br!();
            if let Err(err) = configurator.verify() {
                warning!(
                    "Failed to verify {} account: {}",
                    configurator.platform_name(),
                    err
                );
            }
        }
    } else {
        br!();
        success!(
            "Switched to {} account: {}",
            configurator.platform_name(),
            account_name
        );
    }

    Ok(())
}

/// 删除账户的通用逻辑
pub fn remove_account_generic<S, A>(
    context: &mut WorkflowContext,
    platform_name: &str,
) -> Result<(), String>
where
    GlobalConfig: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
{
    let settings: &S = context.settings().get_settings();
    if !settings.has_accounts() {
        return Err(format!("No {} accounts available to remove", platform_name));
    }

    br!();
    info!("Removing {} account...", platform_name);
    br!();

    let account_options: Vec<String> = settings
        .accounts()
        .iter()
        .map(|acc| acc.display_with_marker(settings.current() == acc.name()))
        .collect();

    let default_index = settings
        .accounts()
        .iter()
        .position(|acc| acc.name() == settings.current())
        .unwrap_or(0);

    let selected_account = SelectBuilder::new(
        format!("Please select the {} account to remove", platform_name),
        account_options,
    )
    .default(default_index)
    .result_title("Account to remove")
    .prompt()
    .map_err(|e| e.to_string())?;

    let account_name = selected_account
        .split(' ')
        .next()
        .ok_or_else(|| "Failed to parse account name".to_string())?
        .to_string();

    let settings: &S = context.settings().get_settings();
    let account = settings
        .find_account(&account_name)
        .ok_or_else(|| format!("Account '{}' not found", account_name))?;

    info!("Account to remove:");
    info!("  - Name: {}", account.name());
    info!("  - Email: {}", account.email());
    info!("  - Token: {}", account.masked_token());
    br!();

    let confirm_result = confirm!("Are you sure you want to remove this account?")
        .default(false)
        .result_title("Confirm removal")
        .prompt()
        .map_err(|e| e.to_string())?;

    if !confirm_result {
        br!();
        info!("Removal cancelled.");
        return Ok(());
    }

    let was_current = settings.current() == account_name;
    let settings: &mut S = context.settings_mut().get_settings_mut();
    settings.remove_account(&account_name);

    if was_current {
        if !settings.has_accounts() {
            settings.set_current(String::new());
        } else {
            br!();
            info!("The current account was removed. Please select a new current account:");
            br!();

            let account_options: Vec<String> =
                settings.accounts().iter().map(|acc| acc.display()).collect();

            let selected_account = SelectBuilder::new(
                "Please select the account to set as current",
                account_options,
            )
            .result_title("Current account")
            .prompt()
            .map_err(|e| e.to_string())?;

            let new_current_name = selected_account
                .split(' ')
                .next()
                .ok_or_else(|| "Failed to parse account name".to_string())?
                .to_string();

            if !settings.account_exists(&new_current_name) {
                return Err(format!("Account '{}' not found", new_current_name));
            }

            settings.set_current(new_current_name.clone());
            br!();
            success!("Switched to account: {}", new_current_name);
        }
    }

    br!();
    success!(
        "{} account '{}' removed successfully.",
        platform_name,
        account_name
    );
    Ok(())
}

/// 切换到指定账户
pub fn switch_to_account<C, S, A>(
    config: &mut C,
    account_name: &str,
    platform_name: &str,
) -> Result<(), String>
where
    C: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
{
    let settings = config.get_settings_mut();

    if !settings.account_exists(account_name) {
        return Err(format!("Account '{}' not found", account_name));
    }

    settings.set_current(account_name.to_string());
    br!();
    success!("Switched to {} account: {}", platform_name, account_name);
    Ok(())
}
