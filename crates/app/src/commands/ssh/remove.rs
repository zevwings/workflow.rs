//! SSH 密钥移除命令

use crate::util::remove_ssh_key;

/// SSH Remove 命令
pub struct SshRemoveCommand {
    fingerprint: Option<String>,
    all: bool,
}

impl SshRemoveCommand {
    pub fn new(fingerprint: Option<String>, all: bool) -> Self {
        Self { fingerprint, all }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        remove_ssh_key(self.fingerprint.clone(), self.all)?;
        Ok(())
    }
}
