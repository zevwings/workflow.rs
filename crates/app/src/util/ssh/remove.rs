use crate::bootstrap::get_ssh_service;
use domain::{SshKeyInfo, SshService};
use prompt::{br, info, multiselect, success, warning};
use std::path::PathBuf;

use crate::util::SshOperationError;

pub fn remove_ssh_key(fingerprint: Option<String>, all: bool) -> Result<(), SshOperationError> {
    let ssh = get_ssh_service();
    if !ssh.is_agent_available() {
        return Err(SshOperationError::AgentNotAvailable);
    }

    if all {
        ssh.remove_all_keys()
            .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
        br!();
        success!("All keys removed from ssh-agent.");
        return Ok(());
    }

    if let Some(ref fp) = fingerprint {
        return remove_by_fingerprint(fp);
    }

    interactive_remove()?;

    Ok(())
}

/// 根据密钥信息解析出私钥文件路径
///
/// 优先级：comment 作为路径 → 指纹匹配 ~/.ssh/*.pub
fn resolve_key_path(key: &SshKeyInfo) -> Option<PathBuf> {
    let ssh = get_ssh_service();
    // comment 可能是文件路径（如 /home/user/.ssh/id_ed25519）
    let path = PathBuf::from(&key.comment);
    if path.exists() {
        return Some(path);
    }

    // comment 不是路径（如 user@host），通过指纹匹配
    ssh.find_key_path_by_fingerprint(&key.fingerprint)
}

fn remove_by_fingerprint(fingerprint: &str) -> Result<(), SshOperationError> {
    let ssh = get_ssh_service();
    let keys = ssh
        .list_loaded_keys()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
    let key = keys.iter().find(|k| k.fingerprint.contains(fingerprint));

    match key {
        Some(k) => match resolve_key_path(k) {
            Some(path) => {
                ssh.remove_key_by_path(&path)
                    .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
                success!("Key {} removed from ssh-agent.", fingerprint);
            }
            None => {
                return Err(SshOperationError::OperationFailed(format!(
                    "Cannot find key file for {}. Use `workflow ssh remove --all` to clear all keys.",
                    fingerprint
                )));
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

fn interactive_remove() -> Result<(), SshOperationError> {
    let ssh: std::sync::Arc<dyn SshService> = get_ssh_service();
    let keys = ssh
        .list_loaded_keys()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    if keys.is_empty() {
        info!("No keys loaded in ssh-agent.");
        return Ok(());
    }

    let options: Vec<String> = keys
        .iter()
        .map(|k| format!("{} ({}) {}", k.fingerprint, k.algorithm, k.comment))
        .collect();

    let selected = multiselect!("Select keys to remove", options.clone())
        .result_title("Keys to remove")
        .prompt()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    if selected.is_empty() {
        info!("No keys selected for removal.");
        return Ok(());
    }

    let mut removed = 0;
    for sel in &selected {
        if let Some(idx) = options.iter().position(|o| o == sel) {
            let key = &keys[idx];
            match resolve_key_path(key) {
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
