use prompt::{info, success};
use toolkit::log_debug;

use crate::registry;

/// Pull 命令
pub struct PullCommand;

impl Default for PullCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PullCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_debug!("pull: getting git repository");
        let git_repo = registry::get_git_repository();

        log_debug!("pull: getting current branch");
        let branch_name = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;
        log_debug!("pull: branch = {}", branch_name);

        info!("Pulling from origin/{}...", branch_name);

        log_debug!("pull: calling git_repo.pull");
        git_repo
            .pull(&branch_name)
            .map_err(|e| format!("Failed to pull: {}", e))?;

        log_debug!("pull: done");
        success!("Successfully pulled from origin/{}", branch_name);
        Ok(())
    }
}
