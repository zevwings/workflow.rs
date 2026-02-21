//! SSH 验证结果格式化实现

use domain::SshVerificationResult;
use prompt::{br, info, warning};

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
            for key in &self.loaded_keys {
                info!(
                    "  - {} ({}) {}",
                    key.fingerprint, key.algorithm, key.comment
                );
            }
        }
    }
}
