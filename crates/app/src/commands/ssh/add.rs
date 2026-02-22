//! SSH 密钥添加命令

use std::path::PathBuf;

use prompt::{br, success};

use crate::util::add_ssh_key;

/// SSH Add 命令
pub struct SshAddCommand {
    key: Option<PathBuf>,
    lifetime: Option<u64>,
}

impl SshAddCommand {
    pub fn new(key: Option<PathBuf>, lifetime: Option<u64>) -> Self {
        Self { key, lifetime }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        add_ssh_key(self.key.clone(), self.lifetime)?;
        br!();
        success!("Key added to ssh-agent successfully.");

        Ok(())
    }
}
