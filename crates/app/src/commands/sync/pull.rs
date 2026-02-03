use prompt::{info, success};
use storage::git::services::{RemoteService, RemoteServiceImpl};
use storage::git::GitContext;
use toolkit::{log_debug, log_info};

/// Pull 命令
pub struct PullCommand;

impl PullCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_debug!("pull: discovering repository");
        let ctx = GitContext::discover()
            .map_err(|e| format!("Failed to open repository: {}", e))?;
        log_debug!("pull: repository discovered");

        let branch_name = {
            let repo = ctx.repository();
            log_debug!("pull: resolving HEAD");
            let head = repo
                .head()
                .map_err(|e| format!("Failed to get HEAD: {}", e))?;
            let branch_name = head
                .shorthand()
                .ok_or("Failed to get branch name")?;
            log_debug!("pull: branch = {}", branch_name);
            branch_name.to_string()
        };

        info!("Pulling from origin/{}...", branch_name);

        log_info!("pull: fetching and merging");
        let remote = RemoteServiceImpl::new(ctx);
        remote
            .pull(&branch_name)
            .map_err(|e| format!("Failed to pull: {}", e))?;

        log_debug!("pull: done");
        success!("Successfully pulled from origin/{}", branch_name);
        Ok(())
    }
}
