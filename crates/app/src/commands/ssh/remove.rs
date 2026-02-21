//! SSH 密钥移除命令

use std::path::PathBuf;
use std::sync::Arc;

use domain::{SshKeyInfo, SshService};
use prompt::{br, info, success, warning, MultiSelectBuilder};

use crate::bootstrap::get_ssh_service;

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
        let ssh = get_ssh_service();

        if !ssh.is_agent_available() {
            return Err("ssh-agent is not running. Start it with `eval $(ssh-agent)` or add to your shell profile.".into());
        }

        if self.all {
            ssh.remove_all_keys()?;
            br!();
            success!("All keys removed from ssh-agent.");
            return Ok(());
        }

        if let Some(ref fp) = self.fingerprint {
            return remove_by_fingerprint(&ssh, fp);
        }

        remove_interactively(&ssh)
    }
}

/// 根据密钥信息解析出私钥文件路径
///
/// 优先级：comment 作为路径 → 指纹匹配 ~/.ssh/*.pub
fn resolve_key_path(ssh: &Arc<dyn SshService>, key: &SshKeyInfo) -> Option<PathBuf> {
    // comment 可能是文件路径（如 /home/user/.ssh/id_ed25519）
    let path = PathBuf::from(&key.comment);
    if path.exists() {
        return Some(path);
    }

    // comment 不是路径（如 user@host），通过指纹匹配
    ssh.find_key_path_by_fingerprint(&key.fingerprint)
}

fn remove_by_fingerprint(
    ssh: &Arc<dyn SshService>,
    fingerprint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let keys = ssh.list_loaded_keys()?;
    let key = keys.iter().find(|k| k.fingerprint.contains(fingerprint));

    match key {
        Some(k) => match resolve_key_path(ssh, k) {
            Some(path) => {
                ssh.remove_key_by_path(&path)?;
                success!("Key {} removed from ssh-agent.", fingerprint);
            }
            None => {
                return Err(format!(
                    "Cannot find key file for {}. Use `workflow ssh remove --all` to clear all keys.",
                    fingerprint
                ).into());
            }
        },
        None => {
            warning!(
                "No key matching fingerprint '{}' found in agent.",
                fingerprint
            );
        }
    }

    Ok(())
}

fn remove_interactively(ssh: &Arc<dyn SshService>) -> Result<(), Box<dyn std::error::Error>> {
    let keys = ssh.list_loaded_keys()?;

    if keys.is_empty() {
        info!("No keys loaded in ssh-agent.");
        return Ok(());
    }

    let options: Vec<String> = keys
        .iter()
        .map(|k| format!("{} ({}) {}", k.fingerprint, k.algorithm, k.comment))
        .collect();

    let selected = MultiSelectBuilder::new("Select keys to remove", options.clone())
        .result_title("Keys to remove")
        .prompt()?;

    if selected.is_empty() {
        info!("No keys selected for removal.");
        return Ok(());
    }

    let mut removed = 0;
    for sel in &selected {
        if let Some(idx) = options.iter().position(|o| o == sel) {
            let key = &keys[idx];
            match resolve_key_path(ssh, key) {
                Some(path) => {
                    if let Err(e) = ssh.remove_key_by_path(&path) {
                        warning!("Failed to remove {}: {}", key.fingerprint, e);
                    } else {
                        removed += 1;
                    }
                }
                None => {
                    warning!(
                        "Cannot find key file for {}. Use --all to clear all keys.",
                        key.fingerprint
                    );
                }
            }
        }
    }

    br!();
    if removed > 0 {
        success!("{} key(s) removed from ssh-agent.", removed);
    }

    Ok(())
}
