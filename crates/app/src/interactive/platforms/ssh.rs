//! SSH 工作流阶段

use std::error::Error;

use domain::{GlobalConfig, VerificationService};
use prompt::{br, separator, success, warning, SelectBuilder};

use crate::{
    bootstrap::{get_ssh_service, get_verification_service},
    commands::ssh::{
        add::{interactive_add, SshAddCommand},
        generate::interactive_generate,
        remove::SshRemoveCommand,
    },
    interactive::{
        core::{
            context::{WorkflowContext, WorkflowMode},
            stage::WorkflowStage,
        },
        display::VerificationResultFormatter,
    },
};

/// SSH 工作流阶段
pub struct SshStage;

impl WorkflowStage for SshStage {
    fn stage_name(&self) -> &'static str {
        "SSH"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        let ssh = get_ssh_service();

        separator!('─', 80, "SSH Configuration");

        if !ssh.is_agent_available() {
            warning!(
                "ssh-agent is not running. Start it with `eval $(ssh-agent)` or add to your shell profile."
            );
            br!();

            let configure_now = prompt::confirm!("Continue with SSH key setup anyway?")
                .default(false)
                .result_title("Continue SSH setup")
                .prompt()?;

            if !configure_now {
                return Ok(());
            }
        }

        loop {
            br!();
            let verification_result = get_verification_service().verify_ssh_config()?;
            verification_result.format();
            br!();

            let has_keys = !verification_result.loaded_keys.is_empty();
            let has_key_files = !ssh.scan_keys().is_empty();

            let mut options = Vec::new();
            if has_key_files {
                options.push("Add an existing key to the agent".to_string());
            }
            options.push("Generate a new SSH key".to_string());
            if has_keys {
                options.push("Remove a key from the agent".to_string());
            }

            let exit_option = if context.mode() == WorkflowMode::Setup {
                "Continue to next step"
            } else {
                "Done"
            };
            options.push(exit_option.to_string());

            let selected = SelectBuilder::new("What would you like to do?", options).prompt()?;

            if selected.contains("Generate") {
                br!();
                let key_path = interactive_generate()?;
                br!();
                let add_now = prompt::confirm!("Add the new key to the ssh-agent now?")
                    .default(true)
                    .prompt()?;
                if add_now {
                    interactive_add(&key_path)?;
                }
                break;
            } else if selected.contains("Add an existing") {
                br!();
                if let Err(e) = SshAddCommand::new(None, None).run() {
                    warning!("{}", e);
                } else {
                    break;
                }
            } else if selected.contains("Remove a key") {
                br!();
                if let Err(e) = SshRemoveCommand::new(None, false).run() {
                    warning!("{}", e);
                } else {
                    break;
                }
            } else if selected.contains(exit_option) {
                break;
            }
        }

        if context.mode() == WorkflowMode::Command {
            br!();
            success!("SSH configuration complete.");
        }

        Ok(())
    }

    fn modifies_config(&self) -> bool {
        false
    }

    fn is_configured(&self, _settings: &GlobalConfig) -> bool {
        let ssh = get_ssh_service();
        ssh.is_agent_available()
            && ssh.list_loaded_keys().map(|keys| !keys.is_empty()).unwrap_or(false)
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_ssh_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// 获取 SSH 阶段实例
pub fn ssh_stage() -> &'static dyn WorkflowStage {
    &SshStage
}
