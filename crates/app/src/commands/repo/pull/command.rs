use prompt::{info, success};
use toolkit::log_debug;

use crate::{bootstrap, util::ensure_ssh_ready};

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

        ensure_ssh_ready()?;

        let git_repo = bootstrap::get_git_repository();

        // 检查工作区状态
        let status = git_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working tree status: {}", e))?;

        let needs_stash = !status.is_clean();

        // 如果有未提交的更改，先 stash
        if needs_stash {
            info!("Working tree has uncommitted changes, stashing...");
            git_repo
                .stash_push(Some("Auto-stash before creating branch from default"))
                .map_err(|e| format!("Failed to stash changes: {}", e))?;
        }

        log_debug!("pull: getting current branch");
        let branch_name = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;
        log_debug!("pull: branch = {}", branch_name);

        info!("Pulling from origin/{}...", branch_name);

        log_debug!("pull: calling git_repo.pull");
        git_repo.pull(&branch_name).map_err(|e| format!("Failed to pull: {}", e))?;

        if needs_stash {
            info!("Restoring stashed changes...");
            git_repo
                .stash_pop(0)
                .map_err(|e| format!("Failed to restore stashed changes: {}", e))?;
        }

        log_debug!("pull: done");
        success!("Successfully pulled from origin/{}", branch_name);
        Ok(())
    }
}
