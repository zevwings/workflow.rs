//! Branch push command
//!
//! Push a branch to remote repository using git2.

use crate::git::GitBranch;
use crate::{log_info, log_success};
use color_eyre::{eyre::WrapErr, Result};

/// Branch push command
pub struct PushCommand;

impl PushCommand {
    /// Execute the branch push command
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Optional branch name (defaults to current branch)
    /// * `set_upstream` - Whether to set upstream tracking branch
    /// * `force_with_lease` - Whether to use force-with-lease push
    pub fn execute(
        branch_name: Option<String>,
        set_upstream: bool,
        force_with_lease: bool,
    ) -> Result<()> {
        let target_branch = if let Some(name) = branch_name {
            name
        } else {
            // Use current branch if not specified
            GitBranch::current_branch()?
        };

        // Check if branch exists locally
        let (exists_local, _) = GitBranch::is_branch_exists(&target_branch)?;
        if !exists_local {
            return Err(color_eyre::eyre::eyre!(
                "Branch '{}' does not exist locally",
                target_branch
            ));
        }

        log_info!("Pushing branch '{}' to remote...", target_branch);

        if force_with_lease {
            GitBranch::push_force_with_lease(&target_branch)
                .wrap_err_with(|| format!("Failed to force push branch: {}", target_branch))?;
            log_success!("Force pushed branch '{}' to remote", target_branch);
        } else {
            GitBranch::push(&target_branch, set_upstream)
                .wrap_err_with(|| format!("Failed to push branch: {}", target_branch))?;
            if set_upstream {
                log_success!(
                    "Pushed branch '{}' to remote and set upstream tracking",
                    target_branch
                );
            } else {
                log_success!("Pushed branch '{}' to remote", target_branch);
            }
        }

        Ok(())
    }
}
