use prompt::{info, success};
use storage::git::GitContext;
use toolkit::{log_debug, log_info};

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
        log_debug!("push: discovering repository");
        let ctx =
            GitContext::discover().map_err(|e| format!("Failed to open repository: {}", e))?;
        log_debug!("push: repository discovered");

        let repo = ctx.repository();

        log_debug!("push: resolving HEAD");
        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let branch_name = head.shorthand().ok_or("Failed to get branch name")?;
        log_debug!("push: branch = {}", branch_name);

        log_debug!("push: finding remote origin");
        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;
        log_debug!("push: remote origin found");

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        info!("Pushing to origin/{}...", branch_name);

        log_info!("push: creating callbacks and push options");
        let callbacks = GitContext::create_callbacks();
        let mut opts = git2::PushOptions::new();
        opts.remote_callbacks(callbacks);
        log_info!("push: calling remote.push (refspec = {})", refspec);

        remote
            .push(&[&refspec], Some(&mut opts))
            .map_err(|e| format!("Failed to push: {}", e))?;

        log_debug!("push: remote.push returned");
        success!("Successfully pushed to origin/{}", branch_name);
        Ok(())
    }
}
