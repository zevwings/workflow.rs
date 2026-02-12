//! 平台配置流程

use domain::GlobalConfig;
use prompt::{br, info, separator, SelectBuilder};

use crate::interactive::core::context::{WorkflowContext, WorkflowMode};
use crate::interactive::core::platform::{
    account::{remove_account_generic, switch_account_generic, switch_to_account},
    traits::{GlobalConfigAccessor, PlatformAccount, PlatformConfigurator, PlatformSettings},
    types::{AccountAction, AccountSetMode},
};

/// 配置平台账户的主入口
pub fn configure_platform<S, A, F, U>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
    add_account_fn: F,
    update_account_fn: U,
) -> Result<(), String>
where
    GlobalConfig: GlobalConfigAccessor<S>,
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

/// 构建菜单选项
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

/// 处理用户选择的操作
fn handle_action<S, A, U>(
    context: &mut WorkflowContext,
    configurator: &impl PlatformConfigurator,
    selected_action: &AccountAction,
    platform_name: &str,
    update_account_fn: U,
) -> Result<(), String>
where
    GlobalConfig: GlobalConfigAccessor<S>,
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
