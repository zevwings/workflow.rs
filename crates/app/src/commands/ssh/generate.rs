//! SSH 密钥生成命令

use std::path::PathBuf;

use crate::util::{generate_ssh_key, GenerateOptions};

/// SSH Generate 命令
pub struct SshGenerateCommand {
    output: Option<PathBuf>,
    algorithm: String,
    comment: Option<String>,
    force: bool,
    no_passphrase: bool,
}

impl SshGenerateCommand {
    pub fn new(
        output: Option<PathBuf>,
        algorithm: String,
        comment: Option<String>,
        force: bool,
        no_passphrase: bool,
    ) -> Self {
        Self {
            output,
            algorithm,
            comment,
            force,
            no_passphrase,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        generate_ssh_key(GenerateOptions {
            algorithm: Some(self.algorithm.clone()),
            output: self.output.clone(),
            force: Some(self.force),
            no_passphrase: self.no_passphrase,
            comment: self.comment.clone(),
        })?;
        Ok(())
    }
}
