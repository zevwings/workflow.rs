//! SSH 工作流阶段

use std::error::Error;

use domain::{GlobalConfig, VerificationService};
use prompt::{br, info, separator, warning, SelectBuilder};

use crate::bootstrap::get_ssh_service;
use crate::commands::ssh::{add::interactive_add, generate::interactive_generate};
use crate::interactive::{
    core::{context::WorkflowContext, stage::WorkflowStage},
    display::VerificationResultFormatter,
};

/// SSH 工作流阶段
pub struct SshStage;

impl WorkflowStage for SshStage {
    fn stage_name(&self) -> &'static str {
        "SSH"
    }

    fn configure(&self, _context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        let ssh = get_ssh_service();

        separator!('─', 80, "SSH configuration");
        br!();

        if !ssh.is_agent_available() {
            warning!(
                "ssh-agent is not running. Start it with `eval $(ssh-agent)` or add to your shell profile."
            );
            br!();

            let configure_now = prompt::confirm!("Continue with SSH key setup anyway?")
                .default(false)
                .result_title("Continue SSH setup")
                .prompt()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            if !configure_now {
                return Ok(());
            }
        }

        let has_keys = ssh.list_loaded_keys().map(|keys| !keys.is_empty()).unwrap_or(false);

        if has_keys {
            info!("SSH keys detected in ssh-agent.");
            let keys = ssh.list_loaded_keys().unwrap_or_default();
            for key in &keys {
                info!(
                    "  - {} ({}) {}",
                    key.fingerprint, key.algorithm, key.comment
                );
            }
            br!();

            let keep = prompt::confirm!("SSH keys are already loaded. Keep current configuration?")
                .default(true)
                .result_title("Keep SSH configuration")
                .prompt()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            if keep {
                return Ok(());
            }
        }

        // 交互式引导：生成新密钥或添加已有密钥
        let existing_keys = ssh.scan_keys();
        let has_existing_files = !existing_keys.is_empty();

        let options = if has_existing_files {
            vec![
                "Add existing key to agent".to_string(),
                "Generate a new SSH key".to_string(),
                "Skip SSH configuration".to_string(),
            ]
        } else {
            vec![
                "Generate a new SSH key".to_string(),
                "Skip SSH configuration".to_string(),
            ]
        };

        let selected = SelectBuilder::new("How would you like to configure SSH?", options)
            .default(0)
            .result_title("SSH action")
            .prompt()
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        if selected.contains("Skip") {
            return Ok(());
        }

        if selected.contains("Generate") {
            br!();
            let key_path = interactive_generate()?;
            br!();
            interactive_add(&key_path)?;
        } else if selected.contains("Add existing") {
            br!();
            let options: Vec<String> =
                existing_keys.iter().map(|p| p.display().to_string()).collect();

            let selected_path = SelectBuilder::new("Select a key to add", options)
                .default(0)
                .result_title("Key")
                .prompt()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            let path = std::path::PathBuf::from(selected_path);
            interactive_add(&path)?;
        }

        Ok(())
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
