//! Branch switch command
//!
//! Switch to a branch with support for:
//! - Direct branch name switching
//! - Interactive branch selection
//! - Fuzzy matching / search (auto-enabled when branch count > 25)
//! - Auto-create branch if not exists (with user confirmation)
//! - Auto-stash uncommitted changes

use crate::base::dialog::ConfirmDialog;
use crate::commands::branch::helpers::{select_branch, BranchSelectionOptions};
use crate::commands::pr::helpers::handle_stash_pop_result;
use crate::git::{GitBranch, GitCommit, GitStash};
use crate::{log_info, log_success};
use color_eyre::{eyre::WrapErr, Result};

/// Branch switch command
pub struct SwitchCommand;

impl SwitchCommand {
    /// Execute the branch switch command
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Optional branch name
    pub fn execute(branch_name: Option<String>) -> Result<()> {
        let target_branch = if let Some(name) = branch_name {
            // Direct branch name provided
            name
        } else {
            // Interactive selection (fuzzy filter auto-enabled based on branch count)
            select_branch(
                BranchSelectionOptions::new()
                    .mark_current_branch()
                    .with_default_index(0)
                    .with_prompt("Select branch to switch to"),
            )?
        };

        // Check if already on target branch
        let current_branch = GitBranch::current_branch()?;
        if current_branch == target_branch {
            log_info!("Already on branch '{}'", target_branch);
            return Ok(());
        }

        // Check if branch exists
        let (exists_local, exists_remote) = GitBranch::is_branch_exists(&target_branch)?;

        let should_create = if !exists_local && !exists_remote {
            // Branch does not exist, prompt user to confirm creation
            ConfirmDialog::new(format!(
                "Branch '{}' does not exist. Create it?",
                target_branch
            ))
            .prompt()
            .wrap_err("Failed to get user confirmation")?
        } else {
            false
        };

        let has_uncommitted =
            GitCommit::has_commit().wrap_err("Failed to check uncommitted changes")?;

        let has_stashed = if has_uncommitted {
            log_info!("Stashing uncommitted changes before switching branch...");
            GitStash::stash_push(Some(&format!(
                "Auto-stash before switching to {}",
                target_branch
            )))?;
            true
        } else {
            false
        };

        // Switch or create branch
        if should_create {
            log_info!("Creating and switching to branch '{}'...", target_branch);
        } else {
            log_info!("Switching to branch '{}'...", target_branch);
        }

        if let Err(e) = GitBranch::checkout_branch(&target_branch)
            .wrap_err_with(|| format!("Failed to switch to branch: {}", target_branch))
        {
            // If checkout fails, try to restore stash
            if has_stashed {
                handle_stash_pop_result(GitStash::stash_pop(None));
            }
            return Err(e);
        }

        log_success!("Switched to branch '{}'", target_branch);

        // Restore stash if needed
        if has_stashed {
            log_info!("Restoring stashed changes...");
            handle_stash_pop_result(GitStash::stash_pop(None));
        }

        Ok(())
    }
}
