use crate::bootstrap::get_ssh_service;
use domain::SshService;
use prompt::{info, select};
use std::path::PathBuf;

use crate::util::SshOperationError;

fn select_key_interactively() -> Result<PathBuf, SshOperationError> {
    let ssh = get_ssh_service();
    let keys = ssh.scan_keys();

    if keys.is_empty() {
        return Err(SshOperationError::OperationFailed(
            "No SSH keys found in ~/.ssh/. Run `workflow ssh generate` to create one.".into(),
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
        None => select_key_interactively()
            .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?,
    };

    let lifetime = match lifetime {
        Some(lt) => Some(lt),
        None => select_lifetime_interactively()
            .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?,
    };

    info!("Adding key: {}", key_path.display());

    ssh.add_key(&key_path, lifetime)
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
    Ok(())
}
