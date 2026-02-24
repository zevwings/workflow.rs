//! Codeup 工作流阶段 (v2)

use std::error::Error;

use domain::{GlobalConfig, VerificationService};
use prompt::{
    br, confirm, info, separator, FormBuilder, InputFormField, PasswordFormField, PromptError,
};
use toolkit::Sensitive;

use crate::interactive::{
    core::{WorkflowContext, WorkflowMode, WorkflowStage},
    display::VerificationResultFormatter,
};

/// Codeup 工作流阶段
pub struct CodeupStage;

impl CodeupStage {
    /// 运行 Codeup 配置表单
    fn run_form(settings: &mut GlobalConfig) -> Result<(), String> {
        info!("Configure Codeup project ID, CSRF token and cookie. Leave fields empty to keep defaults or skip.");
        br!();

        let codeup = &mut settings.codeup;
        let current_project_id = codeup.project_id.clone();
        let current_csrf_token = codeup.csrf_token.clone();
        let current_cookie = codeup.cookie.clone();

        let builder = FormBuilder::new()
            .with_title("Codeup configuration")
            .add_input(
                InputFormField::new("project_id", "Please enter your Codeup project ID")
                    .default(current_project_id)
                    .result_title("Your Codeup project ID")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("csrf_token", "Please enter your Codeup CSRF token")
                    .default(current_csrf_token)
                    .result_title("Your Codeup CSRF token")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("cookie", "Please enter your Codeup cookie")
                    .default(current_cookie)
                    .result_title("Your Codeup cookie")
                    .required(),
            );

        let result = builder.run().map_err(|e| e.to_string())?;
        let project_id = result.get_string("project_id");
        let csrf_token = result.get_string("csrf_token");
        let cookie = result.get_string("cookie");

        if !project_id.trim().is_empty() {
            codeup.project_id = project_id.trim().to_string();
        }
        if !csrf_token.trim().is_empty() {
            codeup.csrf_token = csrf_token;
        }
        if !cookie.trim().is_empty() {
            codeup.cookie = cookie;
        }

        Ok(())
    }
}

impl WorkflowStage for CodeupStage {
    fn stage_name(&self) -> &'static str {
        "Codeup"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        let mode = context.mode();
        let settings = context.settings_mut();

        separator!('─', 80, "Codeup configuration");
        br!();

        let codeup = &settings.codeup;
        let has_codeup = !codeup.project_id.is_empty()
            || !codeup.csrf_token.is_empty()
            || !codeup.cookie.is_empty();

        if has_codeup {
            info!("Codeup configuration detected!");
            info!("  - Project ID: {}", codeup.project_id);
            if !codeup.csrf_token.is_empty() {
                info!("  - CSRF token: {}", codeup.csrf_token.mask());
            }
            if !codeup.cookie.is_empty() {
                info!("  - Cookie: {}", codeup.cookie.mask());
            }
            br!();
        } else {
            info!("No Codeup configuration detected.");
            br!();
        }

        if !has_codeup {
            let should_configure = confirm!("Do you want to configure Codeup?")
                .default(true)
                .result_title("Configure Codeup")
                .prompt()
                .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;
            if !should_configure {
                return Ok(());
            }
        } else if mode == WorkflowMode::Setup && has_codeup {
            let keep = confirm!("Existing Codeup configuration detected. Keep current values?")
                .default(true)
                .result_title("Keep Codeup configuration")
                .prompt()
                .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;

            if keep {
                return Ok(());
            }
        }

        Self::run_form(settings)?;

        Ok(())
    }

    fn is_configured(&self, settings: &GlobalConfig) -> bool {
        !settings.codeup.is_empty()
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_codeup_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// 获取 Codeup 阶段实例
pub fn codeup_stage() -> &'static dyn WorkflowStage {
    &CodeupStage
}
