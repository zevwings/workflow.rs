//! SSH 密钥添加命令

use std::path::{Path, PathBuf};

use prompt::{br, info, success, warning, SelectBuilder};

use crate::bootstrap::get_ssh_service;

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
        let ssh = get_ssh_service();

        if !ssh.is_agent_available() {
            return Err("ssh-agent is not running. Start it with `eval $(ssh-agent)` or add to your shell profile.".into());
        }

        let key_path = match &self.key {
            Some(path) => path.clone(),
            None => select_key_interactively()?,
        };

        let lifetime = match self.lifetime {
            Some(lt) => Some(lt),
            None => select_lifetime_interactively()?,
        };

        info!("Adding key: {}", key_path.display());

        ssh.add_key(&key_path, lifetime)?;

        br!();
        success!("Key added to ssh-agent successfully.");

        Ok(())
    }
}

fn select_key_interactively() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let ssh = get_ssh_service();
    let keys = ssh.scan_keys();

    if keys.is_empty() {
        return Err(
            "No SSH keys found in ~/.ssh/. Run `workflow ssh generate` to create one.".into(),
        );
    }

    if keys.len() == 1 {
        info!("Found key: {}", keys[0].display());
        return Ok(keys[0].clone());
    }

    let options: Vec<String> = keys.iter().map(|p| p.display().to_string()).collect();

    let selected = SelectBuilder::new("Select a key to add", options)
        .default(0)
        .result_title("Key")
        .prompt()?;

    Ok(PathBuf::from(selected))
}

fn select_lifetime_interactively() -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let options = vec![
        "1 hour (3600s)".to_string(),
        "8 hours (28800s)".to_string(),
        "Permanent (no expiry)".to_string(),
    ];

    let selected = SelectBuilder::new("Select key lifetime", options)
        .default(1)
        .result_title("Lifetime")
        .prompt()?;

    let lifetime = if selected.contains("1 hour") {
        Some(3600)
    } else if selected.contains("8 hours") {
        Some(28800)
    } else {
        None
    };

    Ok(lifetime)
}

/// 交互式添加密钥（从 stage 调用）
pub fn interactive_add(key_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ssh = get_ssh_service();

    if !ssh.is_agent_available() {
        warning!("ssh-agent is not running. Key was generated but not loaded.");
        info!("Start ssh-agent and run `workflow ssh add` to load the key.");
        return Ok(());
    }

    ssh.add_key(key_path, None)?;
    success!("Key added to ssh-agent.");

    Ok(())
}
