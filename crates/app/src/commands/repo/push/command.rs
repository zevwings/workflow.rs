use prompt::{info, success};
use toolkit::log_debug;

use crate::bootstrap;
use crate::util::ensure_ssh_ready;

/// Push 命令
pub struct PushCommand;

impl Default for PushCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PushCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_debug!("push: ensuring SSH ready");
        ensure_ssh_ready()?;

        log_debug!("push: getting git repository");
        let git_repo = bootstrap::get_git_repository();
        log_debug!("push: getting current branch");
        let branch_name = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;
        log_debug!("push: branch = {}", branch_name);

        info!("Pushing to origin/{}...", branch_name);

        log_debug!("push: calling git_repo.push");
        git_repo
            .push(&branch_name, false)
            .map_err(|e| format!("Failed to push: {}", e))?;

        log_debug!("push: done");
        success!("Successfully pushed to origin/{}", branch_name);
        Ok(())
    }
}
