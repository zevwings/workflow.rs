//! SSH 工作流阶段

use std::error::Error;

use domain::{GlobalConfig, VerificationService};
use prompt::{br, confirm, info, select, separator, success, warning};

use crate::{
    bootstrap::{get_ssh_service, get_verification_service},
    interactive::{
        core::{WorkflowContext, WorkflowMode, WorkflowStage},
        display::VerificationResultFormatter,
    },
    util::{add_ssh_key, generate_ssh_key, has_unloaded_keys, remove_ssh_key, GenerateOptions},
};

/// SSH 配置操作选项
#[derive(Debug, Clone, PartialEq)]
enum SshAction {
    AddExistingKey,
    GenerateNewKey,
    RemoveKey,
    ContinueToNextStep,
    Done,
}

impl std::fmt::Display for SshAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddExistingKey => write!(f, "Add an existing key to the agent"),
            Self::GenerateNewKey => write!(f, "Generate a new SSH key"),
            Self::RemoveKey => write!(f, "Remove a key from the agent"),
            Self::ContinueToNextStep => write!(f, "Continue to next step"),
            Self::Done => write!(f, "Done"),
        }
    }
}

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

            let configure_now = confirm!("Continue with SSH key setup anyway?")
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
            // Setup 使用简洁展示（与 Jira 一致），check 使用 VerificationResultFormatter 表格
            if verification_result.agent_available {
                if verification_result.loaded_keys.is_empty() {
                    info!("SSH agent: running (no keys loaded)");
                    info!("  - Run `workflow ssh add` to load a key.");
                } else {
                    info!("SSH configuration detected!");
                    info!(
                        "  - SSH agent: running ({} key(s) loaded)",
                        verification_result.loaded_keys.len()
                    );
                    for key in &verification_result.loaded_keys {
                        info!(
                            "  - {} ({}): {}",
                            key.comment, key.algorithm, key.fingerprint
                        );
                    }
                }
            }
            br!();

            let has_keys = !verification_result.loaded_keys.is_empty();
            let has_unloaded_keys = has_unloaded_keys();

            let mut options: Vec<SshAction> = Vec::new();
            if has_unloaded_keys {
                options.push(SshAction::AddExistingKey);
            }
            options.push(SshAction::GenerateNewKey);
            if has_keys && context.mode() == WorkflowMode::Command {
                options.push(SshAction::RemoveKey);
            }
            options.push(match context.mode() {
                WorkflowMode::Setup => SshAction::ContinueToNextStep,
                WorkflowMode::Command => SshAction::Done,
            });

            let default_idx = options.len() - 1;
            let selected =
                select!("What would you like to do?", options).default(default_idx).prompt()?;

            match selected {
                SshAction::AddExistingKey => {
                    br!();
                    add_ssh_key(None, None)?;
                    break;
                }
                SshAction::GenerateNewKey => {
                    br!();
                    let key_path = generate_ssh_key(GenerateOptions::default())?;
                    br!();
                    let add_now = confirm!("Add the new key to the ssh-agent now?")
                        .default(true)
                        .result_title("Add key to agent")
                        .prompt()?;
                    if add_now {
                        add_ssh_key(Some(key_path), None)?;
                    }
                    break;
                }
                SshAction::RemoveKey => {
                    br!();
                    if let Err(e) = remove_ssh_key(None, false) {
                        warning!("{}", e);
                    } else {
                        break;
                    }
                }
                SshAction::ContinueToNextStep | SshAction::Done => break,
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
