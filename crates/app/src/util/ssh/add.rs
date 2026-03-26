use std::path::PathBuf;

use domain::SshService;
use prompt::{info, select};

use crate::bootstrap::get_ssh_service;
use crate::util::SshOperationError;

/// 是否存在可添加到 agent 的密钥（磁盘上有且未加载）
pub fn has_unloaded_keys() -> bool {
    let ssh = get_ssh_service();
    let all_keys = ssh.scan_keys();
    if all_keys.is_empty() {
        return false;
    }
    if !ssh.is_agent_available() {
        return true;
    }
    let loaded_paths: std::collections::HashSet<_> = ssh
        .list_loaded_keys()
        .unwrap_or_default()
        .iter()
        .filter_map(|k| ssh.find_key_path_by_fingerprint(&k.fingerprint))
        .collect();
    all_keys.iter().any(|p| !loaded_paths.contains(p))
}

fn select_key_interactively() -> Result<PathBuf, SshOperationError> {
    let ssh = get_ssh_service();
    let all_keys = ssh.scan_keys();

    if all_keys.is_empty() {
        return Err(SshOperationError::OperationFailed(
            "No SSH keys found in ~/.ssh/. Run `workflow ssh generate` to create one.".into(),
        ));
    }

    // 过滤掉已在 agent 中的密钥
    let keys: Vec<PathBuf> = if ssh.is_agent_available() {
        let loaded_paths: std::collections::HashSet<_> = ssh
            .list_loaded_keys()
            .unwrap_or_default()
            .iter()
            .filter_map(|k| ssh.find_key_path_by_fingerprint(&k.fingerprint))
            .collect();
        all_keys.into_iter().filter(|p| !loaded_paths.contains(p)).collect()
    } else {
        all_keys
    };

    if keys.is_empty() {
        return Err(SshOperationError::OperationFailed(
            "All SSH keys are already loaded in the agent.".into(),
        ));
    }

    if keys.len() == 1 {
        info!("Found key: {}", keys[0].display());
        return Ok(keys[0].clone());
    }

    let options: Vec<String> = keys.iter().map(|p| p.display().to_string()).collect();

    let selected = select!("Select a key to add", options)
        .default(0)
        .result_title("Key")
        .prompt()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    Ok(PathBuf::from(selected))
}

fn select_lifetime_interactively() -> Result<Option<u64>, SshOperationError> {
    let options = vec![
        "1 hour (3600s)".to_string(),
        "8 hours (28800s)".to_string(),
        "Permanent (no expiry)".to_string(),
    ];

    let selected = select!("Select key lifetime", options)
        .default(1)
        .result_title("Lifetime")
        .prompt()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    let lifetime = if selected.contains("1 hour") {
        Some(3600)
    } else if selected.contains("8 hours") {
        Some(28800)
    } else {
        None
    };

    Ok(lifetime)
}

pub fn add_ssh_key(key: Option<PathBuf>, lifetime: Option<u64>) -> Result<(), SshOperationError> {
    let ssh: std::sync::Arc<dyn SshService> = get_ssh_service();

    if !ssh.is_agent_available() {
        return Err(SshOperationError::AgentNotAvailable);
    }

    let key_path = match &key {
        Some(path) => path.clone(),
        None => select_key_interactively()?,
    };

    let lifetime = match lifetime {
        Some(lt) => Some(lt),
        None => select_lifetime_interactively()?,
    };

    info!("Adding key: {}", key_path.display());

    ssh.add_key(&key_path, lifetime)
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
    Ok(())
}
