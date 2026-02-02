//! 通用平台逻辑模块
//!
//! 提供用于管理平台账号（GitHub、CNB 等）的 trait 和通用函数。

use crate::workflows::core::context::{WorkflowContext, WorkflowMode};
use prompt::{
    br, confirm, info, separator, success, warning, FormBuilder, FormResult, SelectBuilder,
};
use toolkit::Sensitive;

// =================================================================================
// Traits
// =================================================================================

/// 平台账号 Trait
pub trait PlatformAccount: Clone + Sized {
    fn name(&self) -> &str;
    fn email(&self) -> &str;
    fn api_token(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn set_email(&mut self, email: String);
    fn set_api_token(&mut self, token: String);

    /// 获取账号的唯一标识符（用于显示）
    ///
    /// 默认返回 email，子类可以覆盖（如 CNB 使用 login）
    fn identifier(&self) -> &str {
        self.email()
    }

    fn display_with_marker(&self, is_current: bool) -> String {
        let marker = if is_current { " (current)" } else { "" };
        format!("{} ({}){}", self.name(), self.identifier(), marker)
    }

    fn display(&self) -> String {
        format!("{} ({})", self.name(), self.identifier())
    }

    fn masked_token(&self) -> String
    where
        String: Sensitive,
    {
        self.api_token().to_string().mask()
    }
}

/// 平台设置 Trait
pub trait PlatformSettings {
    type Account: PlatformAccount;

    fn accounts_mut(&mut self) -> &mut Vec<Self::Account>;
    fn accounts(&self) -> &Vec<Self::Account>;
    fn current(&self) -> &str;
    fn set_current(&mut self, name: String);

    fn has_accounts(&self) -> bool {
        !self.accounts().is_empty()
    }

    fn find_account(&self, name: &str) -> Option<&Self::Account> {
        self.accounts().iter().find(|acc| acc.name() == name)
    }

    fn find_account_mut(&mut self, name: &str) -> Option<&mut Self::Account> {
        self.accounts_mut()
            .iter_mut()
            .find(|acc| acc.name() == name)
    }

    fn get_current_account(&self) -> Option<&Self::Account> {
        let current = self.current();
        if !current.is_empty() {
            self.find_account(current)
        } else {
            self.accounts().first()
        }
    }

    fn remove_account(&mut self, name: &str) -> bool {
        if let Some(index) = self.accounts().iter().position(|acc| acc.name() == name) {
            self.accounts_mut().remove(index);
            true
        } else {
            false
        }
    }

    fn account_exists(&self, name: &str) -> bool {
        self.accounts().iter().any(|acc| acc.name() == name)
    }
}

/// 平台配置器 Trait
pub trait PlatformConfigurator {
    fn platform_name(&self) -> &str;

    fn build_add_form(&self) -> FormBuilder {
        FormBuilder::new().with_title(format!("New {} Account", self.platform_name()))
    }

    fn build_update_form(&self, _account_name: &str) -> FormBuilder {
        FormBuilder::new().with_title(format!("Update {} Account", self.platform_name()))
    }

    fn extract_basic_fields(&self, form_result: &FormResult) -> (String, String, String) {
        let name = form_result.get_string("name");
        let email = form_result.get_string("email");
        let api_token = form_result.get_string("api_token");
        (name, email, api_token)
    }

    fn verify(&self) -> Result<(), String> {
        Ok(())
    }

    fn auto_verify_in_command_setup(&self) -> bool {
        true
    }
}

/// 全局配置访问器 Trait
pub trait GlobalConfigAccessor<S: PlatformSettings> {
    fn get_settings_mut(&mut self) -> &mut S;
    fn get_settings(&self) -> &S;
}

// =================================================================================
// Generic Logic
// =================================================================================

pub fn configure_platform<S, A, F, U>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
    add_account_fn: F,
    update_account_fn: U,
) -> Result<(), String>
where
    domain::GlobalConfig: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
    F: Fn(&mut WorkflowContext, bool) -> Result<String, String>,
    U: Fn(&mut WorkflowContext) -> Result<(), String>,
{
    let mode = context.mode();
    let (has_accounts, platform_name) = {
        let settings: &S = context.settings().get_settings();
        (
            settings.has_accounts(),
            configurator.platform_name().to_string(),
        )
    };

    separator!('─', 80, format!("{} Configuration", platform_name));
    br!();

    if has_accounts {
        info!("{} configuration is detected!", platform_name);
        {
            let settings: &S = context.settings().get_settings();
            for account in settings.accounts().iter() {
                let is_current = settings.current() == account.name();
                info!("  - Account: {}", account.display_with_marker(is_current));
            }
        }
        br!();

        let menu_options = {
            let settings: &S = context.settings().get_settings();
            build_menu_options(settings, mode, &platform_name)
        };
        let selected_action = SelectBuilder::new("Please select an action", menu_options.clone())
            .result_title("Selected action")
            .prompt()
            .map_err(|e| e.to_string())?;

        handle_action(
            context,
            configurator,
            &selected_action,
            &platform_name,
            update_account_fn,
        )?;
    } else {
        info!("No {} accounts were detected.", platform_name);
        br!();
        add_account_fn(context, true)?;
    }

    Ok(())
}

fn build_menu_options<S, A>(settings: &S, mode: WorkflowMode, platform_name: &str) -> Vec<String>
where
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
{
    match mode {
        WorkflowMode::Setup => {
            let mut options = vec![];

            if let Some(current_account) = settings.get_current_account() {
                options.push(format!(
                    "Keep current account {}",
                    current_account.display()
                ));
            }

            for account in settings.accounts().iter() {
                if account.name() != settings.current() {
                    options.push(format!("Use exists account {}", account.display()));
                }
            }

            options.push(format!("Add new {} account", platform_name));
            options
        }
        WorkflowMode::Command => vec![
            format!("Add new {} account", platform_name),
            format!("Switch current {} account", platform_name),
            format!("Update {} account information", platform_name),
            format!("Remove {} account", platform_name),
        ],
    }
}

fn handle_action<S, A, U>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
    selected_action: &str,
    platform_name: &str,
    update_account_fn: U,
) -> Result<(), String>
where
    domain::GlobalConfig: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
    U: Fn(&mut WorkflowContext) -> Result<(), String>,
{
    match context.mode() {
        WorkflowMode::Setup => {
            if selected_action.starts_with("Keep current account") {
                br!();
                info!("Keeping current {} account.", platform_name);
                Ok(())
            } else if selected_action.starts_with("Use exists account") {
                let account_name =
                    extract_account_name_from_option(selected_action, "Use exists account")?;
                switch_to_account(context.settings_mut(), &account_name, platform_name)
            } else if selected_action.starts_with(&format!("Add new {} account", platform_name)) {
                Err(format!(
                    "Add new account for {} should be called from platform-specific module",
                    platform_name
                ))
            } else {
                Err("Invalid action selected".to_string())
            }
        }
        WorkflowMode::Command => {
            if selected_action.starts_with(&format!("Add new {} account", platform_name)) {
                Err(format!(
                    "Add new account for {} should be called from platform-specific module",
                    platform_name
                ))
            } else if selected_action
                .starts_with(&format!("Switch current {} account", platform_name))
            {
                switch_account_generic(context, configurator)
            } else if selected_action
                .starts_with(&format!("Update {} account information", platform_name))
            {
                update_account_fn(context)
            } else if selected_action.starts_with(&format!("Remove {} account", platform_name)) {
                remove_account_generic(context, platform_name)
            } else {
                Err("Invalid action selected".to_string())
            }
        }
    }
}

pub fn add_account_generic<S, A, F>(
    context: &mut WorkflowContext,
    account_creator: F,
    set_as_current: bool,
    platform_name: &str,
    verify_fn: Option<fn() -> Result<(), String>>,
) -> Result<String, String>
where
    domain::GlobalConfig: GlobalConfigAccessor<S>,
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

    if set_as_current || settings.current().is_empty() {
        settings.set_current(account_name.clone());
    }

    if context.mode() == WorkflowMode::Command {
        context
            .save()
            .map_err(|e| format!("Failed to save config: {}", e))?;

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

pub fn switch_account_generic<S, A>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
) -> Result<(), String>
where
    domain::GlobalConfig: GlobalConfigAccessor<S>,
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
        context
            .save()
            .map_err(|e| format!("Failed to save config: {}", e))?;

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

pub fn remove_account_generic<S, A>(
    context: &mut WorkflowContext,
    platform_name: &str,
) -> Result<(), String>
where
    domain::GlobalConfig: GlobalConfigAccessor<S>,
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

            let account_options: Vec<String> = settings
                .accounts()
                .iter()
                .map(|acc| acc.display())
                .collect();

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

fn switch_to_account<C, S, A>(
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

pub fn extract_account_name_from_option(option: &str, prefix: &str) -> Result<String, String> {
    let parts: Vec<&str> = option.split_whitespace().collect();
    // Expected format: "Use exists account <name> (<email>)"
    // prefix parts length + name
    let prefix_len = prefix.split_whitespace().count();
    if parts.len() <= prefix_len {
        return Err("Invalid option format".to_string());
    }
    Ok(parts[prefix_len].to_string())
}
