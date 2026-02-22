use crate::bootstrap::{get_git_repository, get_ssh_service};
use prompt::{br, confirm, select, warning};

use crate::util::{add_ssh_key, generate_ssh_key, GenerateOptions, SshOperationError};

/// Ensures SSH is ready for remote operations.
/// If not, it provides an interactive prompt to guide the user.
pub fn ensure_ssh_ready() -> Result<(), SshOperationError> {
    let ssh_service = get_ssh_service();
    let git_repo = get_git_repository();
    let repo_info = git_repo.get_repo_info();
    let remote_url = repo_info.origin_url.as_deref();

    if !is_ssh_remote(remote_url) {
        return Ok(());
    }

    if !ssh_service.is_agent_available() {
        warning!("ssh-agent is not running. It's required for SSH operations.");
        return Err(SshOperationError::AgentNotAvailable);
    }

    let keys = ssh_service
        .list_loaded_keys()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
    if !keys.is_empty() {
        return Ok(()); // Keys are already loaded
    }

    br!();
    warning!("No SSH keys are loaded in the agent for the remote operation.");
    br!();

    let scanned_keys = ssh_service.scan_keys();
    let has_existing_keys = !scanned_keys.is_empty();

    let mut options = Vec::new();
    if has_existing_keys {
        options.push("Use an existing SSH key");
    }
    options.push("Generate a new SSH key");
    options.push("Abort the operation");

    let choice = select!("What would you like to do?", options)
        .prompt()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;

    if choice.contains("Generate") {
        let new_key_path = generate_ssh_key(GenerateOptions::default())
            .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
        br!();
        let add_now = confirm!("Add the new key to the ssh-agent now?")
            .default(true)
            .prompt()
            .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
        if add_now {
            add_ssh_key(Some(new_key_path), None)?;
        }
    } else if choice.contains("existing") {
        add_ssh_key(None, None)?;
    } else {
        return Err(SshOperationError::OperationCancelled);
    }

    let keys = ssh_service
        .list_loaded_keys()
        .map_err(|e| SshOperationError::OperationFailed(e.to_string()))?;
    // Final check to ensure a key was actually added
    if keys.is_empty() {
        Err(SshOperationError::NoKeysAvailable)
    } else {
        Ok(())
    }
}

fn is_ssh_remote(remote_url: Option<&str>) -> bool {
    remote_url.is_some_and(|url| url.starts_with("git@") || url.starts_with("ssh://"))
}
