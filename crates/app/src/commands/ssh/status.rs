//! SSH 状态查看命令

use prompt::{br, info, warning, Alignment, TableBuilder};

use crate::bootstrap::get_ssh_service;

/// SSH Status 命令
pub struct SshStatusCommand;

impl Default for SshStatusCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SshStatusCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ssh = get_ssh_service();

        if !ssh.is_agent_available() {
            warning!("ssh-agent is not running. Start it with `eval $(ssh-agent)` or add to your shell profile.");
            return Ok(());
        }

        let keys = ssh.list_loaded_keys()?;

        if keys.is_empty() {
            info!("No keys loaded in ssh-agent. Run `workflow ssh add` to add keys.");
            return Ok(());
        }

        info!("{} key(s) loaded in ssh-agent:", keys.len());
        br!();

        let mut table = TableBuilder::new(vec!["Fingerprint", "Algorithm", "Comment"])
            .with_alignment(Alignment::Left);

        for key in &keys {
            table = table.add_row(vec![
                key.fingerprint.clone(),
                key.algorithm.clone(),
                key.comment.clone(),
            ]);
        }

        table.print()?;

        Ok(())
    }
}
