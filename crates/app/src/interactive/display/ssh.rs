//! SSH 验证结果格式化实现
//!
//! 用于 check 命令的表格展示，setup 的简洁展示在 platforms/ssh.rs 中实现。

use domain::SshVerificationResult;
use prompt::{br, info, warning, Alignment, TableBuilder};

use crate::interactive::display::formatter::VerificationResultFormatter;

impl VerificationResultFormatter for SshVerificationResult {
    fn format(&self) {
        if !self.agent_available {
            warning!(
                "ssh-agent is not available. Start it with `eval $(ssh-agent)` or add to your shell profile."
            );
            if let Some(ref err) = self.error {
                info!("  Error: {}", err);
            }
            br!();
            return;
        }

        if self.loaded_keys.is_empty() {
            info!("SSH agent: running (no keys loaded)");
            info!("Run `workflow ssh add` to load a key.");
        } else {
            info!(
                "SSH agent: running ({} key(s) loaded)",
                self.loaded_keys.len()
            );
            br!();
            let mut table = TableBuilder::new(vec!["Fingerprint", "Algorithm", "Comment"])
                .with_alignment(Alignment::Left);
            for key in &self.loaded_keys {
                table = table.add_row(vec![
                    key.fingerprint.clone(),
                    key.algorithm.clone(),
                    key.comment.clone(),
                ]);
            }
            table.print().unwrap();
        }
    }
}
