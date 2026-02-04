//! 通用平台逻辑模块
//!
//! 提供用于管理平台账号（GitHub 等）的 trait 和通用函数。

use crate::workflows::core::context::{WorkflowContext, WorkflowMode};
use prompt::{
    br, confirm, info, separator, success, warning, FormBuilder, FormResult, SelectBuilder,
};
use toolkit::Sensitive;

// =================================================================================
// Enums
// =================================================================================

/// 账户设置模式
///
/// 控制新添加的账户是否设为当前账户
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccountSetMode {
    /// 设为当前账户
    #[default]
    SetAsCurrent,
    /// 仅添加，不设为当前
    AddOnly,
}

impl AccountSetMode {
    /// 是否应该设为当前账户
    #[inline]
    pub fn should_set_current(self) -> bool {
        matches!(self, Self::SetAsCurrent)
    }
}

/// 账户操作选项
#[derive(Clone)]
pub enum AccountAction {
    /// 保留当前账户 (Setup 模式)
    KeepCurrent { account_display: String },
    /// 使用已有账户 (Setup 模式)
    UseExisting {
        account_display: String,
        account_name: String,
    },
    /// 添加新账户
    AddNew { platform_name: String },
    /// 切换当前账户 (Command 模式)
    Switch { platform_name: String },
    /// 更新账户信息 (Command 模式)
    Update { platform_name: String },
    /// 删除账户 (Command 模式)
    Remove { platform_name: String },
}

impl std::fmt::Display for AccountAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountAction::KeepCurrent { account_display } => {
                write!(f, "Keep current account {}", account_display)
            }
            AccountAction::UseExisting {
                account_display, ..
            } => {
                write!(f, "Use exists account {}", account_display)
            }
            AccountAction::AddNew { platform_name } => {
                write!(f, "Add new {} account", platform_name)
            }
            AccountAction::Switch { platform_name } => {
                write!(f, "Switch current {} account", platform_name)
            }
            AccountAction::Update { platform_name } => {
                write!(f, "Update {} account information", platform_name)
            }
            AccountAction::Remove { platform_name } => {
                write!(f, "Remove {} account", platform_name)
            }
        }
    }
}

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
    /// 默认返回 email，子类可覆盖
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
        self.accounts_mut().iter_mut().find(|acc| acc.name() == name)
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
    F: Fn(&mut WorkflowContext, AccountSetMode) -> Result<String, String>,
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
        add_account_fn(context, AccountSetMode::SetAsCurrent)?;
    }

    Ok(())
}

fn build_menu_options<S, A>(
    settings: &S,
    mode: WorkflowMode,
    platform_name: &str,
) -> Vec<AccountAction>
where
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
{
    match mode {
        WorkflowMode::Setup => {
            let mut options = vec![];

            if let Some(current_account) = settings.get_current_account() {
                options.push(AccountAction::KeepCurrent {
                    account_display: current_account.display(),
                });
            }

            for account in settings.accounts().iter() {
                if account.name() != settings.current() {
                    options.push(AccountAction::UseExisting {
                        account_display: account.display(),
                        account_name: account.name().to_string(),
                    });
                }
            }

            options.push(AccountAction::AddNew {
                platform_name: platform_name.to_string(),
            });
            options
        }
        WorkflowMode::Command => vec![
            AccountAction::AddNew {
                platform_name: platform_name.to_string(),
            },
            AccountAction::Switch {
                platform_name: platform_name.to_string(),
            },
            AccountAction::Update {
                platform_name: platform_name.to_string(),
            },
            AccountAction::Remove {
                platform_name: platform_name.to_string(),
            },
        ],
    }
}

fn handle_action<S, A, U>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
    selected_action: &AccountAction,
    platform_name: &str,
    update_account_fn: U,
) -> Result<(), String>
where
    domain::GlobalConfig: GlobalConfigAccessor<S>,
    S: PlatformSettings<Account = A>,
    A: PlatformAccount,
    U: Fn(&mut WorkflowContext) -> Result<(), String>,
{
    match selected_action {
        AccountAction::KeepCurrent { .. } => {
            br!();
            info!("Keeping current {} account.", platform_name);
            Ok(())
        }
        AccountAction::UseExisting { account_name, .. } => {
            switch_to_account(context.settings_mut(), account_name, platform_name)
        }
        AccountAction::AddNew { .. } => Err(format!(
            "Add new account for {} should be called from platform-specific module",
            platform_name
        )),
        AccountAction::Switch { .. } => switch_account_generic(context, configurator),
        AccountAction::Update { .. } => update_account_fn(context),
        AccountAction::Remove { .. } => remove_account_generic(context, platform_name),
    }
}

pub fn add_account_generic<S, A, F>(
    context: &mut WorkflowContext,
    account_creator: F,
    set_mode: AccountSetMode,
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
