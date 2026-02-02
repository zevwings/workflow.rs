//! CNB Workflow Stage (v2)

use crate::workflows::core::context::{WorkflowContext, WorkflowMode};
use crate::workflows::core::platform::{
    add_account_generic, configure_platform, GlobalConfigAccessor, PlatformAccount,
    PlatformConfigurator, PlatformSettings,
};
use crate::workflows::core::stage::WorkflowStage;
use crate::workflows::display::VerificationResultFormatter;
use domain::{CNBAccount, CNBSettings, GlobalConfig, VerificationService};
use prompt::{
    br, info, success, warning, FormBuilder, FormResult, InputFormField, PasswordFormField,
    PromptError, SelectBuilder,
};
use std::error::Error;

/// The CNB workflow stage.
pub struct CnbStage;

impl WorkflowStage for CnbStage {
    fn stage_name(&self) -> &'static str {
        "CNB"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        configure_platform::<CNBSettings, _, _, _>(
            context,
            &CnbConfigurator,
            add_new_cnb_account,
            update_cnb_account,
        )
        .map_err(|e| e.into())
    }

    fn is_configured(&self, settings: &GlobalConfig) -> bool {
        !settings.cnb.current().is_empty()
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_cnb_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// Get the CNB stage instance.
pub fn cnb_stage() -> &'static dyn WorkflowStage {
    &CnbStage
}

// =================================================================================
// CNB Configurator & Accessor
// =================================================================================

struct CnbConfigurator;

impl PlatformConfigurator for CnbConfigurator {
    fn platform_name(&self) -> &str {
        "CNB"
    }
}

impl CnbConfigurator {
    fn build_account_form_fields(&self, builder: FormBuilder) -> FormBuilder {
        builder
            .add_input(
                InputFormField::new("name", "Please enter your CNB account name")
                    .result_title("Your CNB account name")
                    .required(),
            )
            .add_input(
                InputFormField::new("login", "Please enter your CNB username")
                    .result_title("Your CNB username")
                    .required(),
            )
            .add_input(
                InputFormField::new("email", "Please enter your CNB email")
                    .result_title("Your CNB email")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("api_token", "Please enter your CNB Access Token")
                    .result_title("Your CNB Access Token")
                    .required(),
            )
    }

    fn build_update_form_fields(
        &self,
        builder: FormBuilder,
        current_name: String,
        current_login: String,
        current_email: String,
        current_token: String,
    ) -> FormBuilder {
        builder
            .add_input(
                InputFormField::new("name", "Please enter your CNB account name")
                    .default(current_name)
                    .result_title("Your CNB account name")
                    .required(),
            )
            .add_input(
                InputFormField::new("login", "Please enter your CNB username")
                    .default(current_login)
                    .result_title("Your CNB username")
                    .required(),
            )
            .add_input(
                InputFormField::new("email", "Please enter your CNB email")
                    .default(current_email)
                    .result_title("Your CNB email")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("api_token", "Please enter your CNB Access Token")
                    .default(current_token)
                    .result_title("Your CNB Access Token")
                    .required(),
            )
    }

    fn create_account_from_form(&self, form_result: &FormResult) -> Result<CNBAccount, String> {
        let (name, email, api_token) = self.extract_basic_fields(form_result);
        let login = form_result.get_string("login");

        if api_token.trim().is_empty() {
            return Err("CNB API token is required to add a new account.".to_string());
        }

        let account_name = if name.trim().is_empty() {
            "default".to_string()
        } else {
            name.trim().to_string()
        };

        Ok(CNBAccount {
            name: account_name,
            login: login.trim().to_string(),
            email: email.trim().to_string(),
            api_token,
        })
    }

    fn update_account_from_form(
        &self,
        account: &mut CNBAccount,
        form_result: &FormResult,
        old_name: &str,
    ) -> Result<String, String> {
        let (new_name, email, api_token) = self.extract_basic_fields(form_result);
        let login = form_result.get_string("login");

        let new_name_trimmed = new_name.trim().to_string();
        let updated_name = if !new_name_trimmed.is_empty() && new_name_trimmed != old_name {
            account.set_name(new_name_trimmed.clone());
            new_name_trimmed
        } else {
            old_name.to_string()
        };

        if !login.trim().is_empty() {
            account.set_login(login.trim().to_string());
        }

        if !email.trim().is_empty() {
            account.set_email(email.trim().to_string());
        }

        if !api_token.trim().is_empty() {
            account.set_api_token(api_token);
        }

        Ok(updated_name)
    }
}

impl GlobalConfigAccessor<CNBSettings> for GlobalConfig {
    fn get_settings_mut(&mut self) -> &mut CNBSettings {
        &mut self.cnb
    }

    fn get_settings(&self) -> &CNBSettings {
        &self.cnb
    }
}

impl PlatformAccount for CNBAccount {
    fn name(&self) -> &str {
        &self.name
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn api_token(&self) -> &str {
        &self.api_token
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_email(&mut self, email: String) {
        self.email = email;
    }
    fn set_api_token(&mut self, token: String) {
        self.api_token = token;
    }

    /// CNB 使用 login 作为主要标识符
    fn identifier(&self) -> &str {
        &self.login
    }
}

impl PlatformSettings for CNBSettings {
    type Account = CNBAccount;

    fn accounts_mut(&mut self) -> &mut Vec<Self::Account> {
        &mut self.accounts
    }
    fn accounts(&self) -> &Vec<Self::Account> {
        &self.accounts
    }
    fn current(&self) -> &str {
        &self.current
    }
    fn set_current(&mut self, name: String) {
        self.current = name;
    }
}

// =================================================================================
// Account Actions
// =================================================================================

fn add_new_cnb_account(
    context: &mut WorkflowContext,
    set_as_current: bool,
) -> Result<String, String> {
    let configurator = CnbConfigurator;
    add_account_generic::<CNBSettings, _, _>(
        context,
        || {
            let builder = configurator.build_add_form();
            let builder = configurator.build_account_form_fields(builder);
            let form_result = builder.run().map_err(|e: PromptError| e.to_string())?;
            configurator.create_account_from_form(&form_result)
        },
        set_as_current,
        "CNB",
        None,
    )
}

fn update_cnb_account(context: &mut WorkflowContext) -> Result<(), String> {
    let configurator = CnbConfigurator;
    let settings: &CNBSettings = context.settings().get_settings();

    if !settings.has_accounts() {
        return Err("No CNB accounts available to update".to_string());
    }

    br!();
    info!("Updating CNB account information...");
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

    let selected_account =
        SelectBuilder::new("Please select the CNB account to update", account_options)
            .default(default_index)
            .result_title("Account to update")
            .prompt()
            .map_err(|e: PromptError| e.to_string())?;

    let account_name = selected_account
        .split(' ')
        .next()
        .ok_or_else(|| "Failed to parse account name".to_string())?
        .to_string();

    let settings: &CNBSettings = context.settings().get_settings();
    let account = settings
        .find_account(&account_name)
        .ok_or_else(|| format!("Account '{}' not found", account_name))?;

    let old_name = account.name().to_string();
    let current_name = account.name().to_string();
    let current_login = account.login().to_string();
    let current_email = account.email().to_string();
    let current_token = account.api_token().to_string();
    let was_current = settings.current() == old_name;

    br!();
    info!("Updating account: {}", account_name);
    info!("Leave fields empty to keep current values.");
    br!();

    let builder = configurator.build_update_form(&account_name);
    let builder = configurator.build_update_form_fields(
        builder,
        current_name,
        current_login,
        current_email,
        current_token,
    );
    let form_result = builder.run().map_err(|e: PromptError| e.to_string())?;

    let (new_name, _, _) = configurator.extract_basic_fields(&form_result);
    let new_name_trimmed = new_name.trim().to_string();
    if !new_name_trimmed.is_empty() && new_name_trimmed != old_name {
        let settings: &CNBSettings = context.settings().get_settings();
        if settings.account_exists(&new_name_trimmed) {
            return Err(format!(
                "Account name '{}' already exists. Please choose a different name.",
                new_name_trimmed
            ));
        }
    }

    let settings: &mut CNBSettings = context.settings_mut().get_settings_mut();
    let account = settings
        .find_account_mut(&account_name)
        .ok_or_else(|| format!("Account '{}' not found", account_name))?;

    let updated_name = configurator.update_account_from_form(account, &form_result, &old_name)?;

    if was_current && updated_name != old_name {
        settings.set_current(updated_name.clone());
    }

    if context.mode() == WorkflowMode::Command {
        context
            .save()
            .map_err(|e| format!("Failed to save config: {}", e))?;

        br!();
        success!("CNB account '{}' updated successfully.", updated_name);

        if configurator.auto_verify_in_command_setup() {
            br!();
            if let Err(err) = configurator.verify() {
                warning!("Failed to verify CNB account: {}", err);
            }
        }
    } else {
        br!();
        success!("CNB account '{}' updated successfully.", updated_name);
    }

    Ok(())
}
