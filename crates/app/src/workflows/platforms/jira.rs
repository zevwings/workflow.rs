//! Jira Workflow Stage (v2)

use crate::workflows::core::context::{WorkflowContext, WorkflowMode};
use crate::workflows::core::stage::WorkflowStage;
use crate::workflows::display::VerificationResultFormatter;
use domain::{GlobalConfig, VerificationService};
use prompt::{
    br, confirm, info, separator, FormBuilder, InputFormField, PasswordFormField, PromptError,
};
use std::error::Error;
use toolkit::Sensitive;

/// The Jira workflow stage.
pub struct JiraStage;

impl JiraStage {
    /// Run the Jira configuration form.
    fn run_form(settings: &mut GlobalConfig) -> Result<(), String> {
        info!("Configure Jira service address, email, and API token. Leave fields empty to keep defaults or skip.");
        br!();

        let jira = &mut settings.jira;
        let current_service = jira.service_address.clone();
        let current_email = jira.email.clone();
        let current_token = jira.api_token.clone();

        let builder = FormBuilder::new()
            .with_title("Jira Configuration")
            .add_input(
                InputFormField::new("service_address", "Please enter your Jira service address")
                    .default(current_service)
                    .result_title("Your Jira service address")
                    .required(),
            )
            .add_input(
                InputFormField::new("email", "Please enter your Jira email")
                    .default(current_email)
                    .result_title("Your Jira email")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("api_token", "Please enter your Jira API token")
                    .default(current_token)
                    .result_title("Your Jira API token")
                    .required(),
            );

        let result = builder.run().map_err(|e| e.to_string())?;
        let service_address = result.get_string("service_address");
        let email = result.get_string("email");
        let api_token = result.get_string("api_token");

        if !service_address.trim().is_empty() {
            jira.service_address = service_address.trim().to_string();
        }
        if !email.trim().is_empty() {
            jira.email = email.trim().to_string();
        }
        if !api_token.trim().is_empty() {
            jira.api_token = api_token;
        }

        Ok(())
    }
}

impl WorkflowStage for JiraStage {
    fn stage_name(&self) -> &'static str {
        "Jira"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        let mode = context.mode();
        let settings = context.settings_mut();

        separator!('─', 80, "Jira Configuration");
        br!();

        let jira = &settings.jira;
        let has_jira = !jira.email.is_empty()
            || !jira.api_token.is_empty()
            || !jira.service_address.is_empty();

        if has_jira {
            info!("Jira configuration is detected!");
            info!("  - Service Address: {}", jira.service_address);
            info!("  - Jira Email: {}", jira.email);
            if !jira.api_token.is_empty() {
                info!("  - API Token: {}", jira.api_token.mask());
            }
            br!();
        }

        // Handle mode-specific interaction
        if mode == WorkflowMode::Setup {
            if has_jira {
                let keep = confirm!(
                    "Existing Jira configuration detected. Do you want to keep the current values?"
                )
                .default(true)
                .result_title("Keep Jira configuration")
                .prompt()
                .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;

                if keep {
                    return Ok(());
                }
            } else {
                let configure = confirm!("Do you want to configure Jira?")
                    .default(false)
                    .result_title("Configure Jira")
                    .prompt()
                    .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;

                if !configure {
                    return Ok(());
                }
            }
        }

        Self::run_form(settings)?;

        Ok(())
    }

    fn is_configured(&self, settings: &GlobalConfig) -> bool {
        !settings.jira.is_empty()
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_jira_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// Get the Jira stage instance.
pub fn jira_stage() -> &'static dyn WorkflowStage {
    &JiraStage
}
