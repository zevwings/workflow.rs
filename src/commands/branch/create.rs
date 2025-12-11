//! Branch create command
//!
//! Create a new branch, optionally from a JIRA ticket or a specific branch.
//!
//! New workflow:
//! 1. Determine branch type (feature/bugfix/refactoring/hotfix/chore)
//! 2. Determine branch name (from JIRA ticket with LLM, or user input)
//! 3. Format using template: {type}/{jira-ticket}-{branch-name}
//! 4. Apply repository prefix if configured

use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::base::indicator::Spinner;
use crate::branch::{BranchNaming, BranchType};
use crate::commands::pr::helpers::handle_stash_pop_result;
use crate::git::{GitBranch, GitCommit, GitStash};
use crate::jira::helpers::validate_jira_ticket_format;
use crate::jira::Jira;
use crate::pr::llm::PullRequestLLM;
use crate::{log_info, log_success, log_warning};
use anyhow::{Context, Result};

/// Branch create command
pub struct CreateCommand;

impl CreateCommand {
    /// Execute the branch create command
    ///
    /// New workflow:
    /// 1. Determine branch type (feature/bugfix/refactoring/hotfix/chore)
    /// 2. Determine branch name (from JIRA ticket with LLM, or user input)
    /// 3. Format using template: {type}/{jira-ticket}-{branch-name}
    ///
    /// # Arguments
    ///
    /// * `ticket_id` - Optional JIRA ticket ID
    /// * `from_default` - Whether to create from default branch
    /// * `dry_run` - Whether to run in dry-run mode (preview without executing)
    pub fn execute(ticket_id: Option<String>, from_default: bool, dry_run: bool) -> Result<()> {
        // Step 1: Resolve ticket ID (optional, interactive if not provided)
        let ticket_id = Self::resolve_ticket_id(ticket_id)?;

        // Step 2: Determine branch type
        // If repository prefix exists, use it as branch type (skip selection)
        let branch_type = Self::resolve_branch_type()?;

        // Step 3: Determine branch name
        let branch_name_slug = if let Some(ticket) = &ticket_id {
            // Generate branch name from JIRA ticket using LLM
            Self::generate_branch_name_from_jira(ticket)?
        } else {
            // Prompt for branch name interactively
            let user_input = Self::resolve_branch_name()?;
            // Convert to slug (handle non-English input)
            BranchNaming::sanitize_and_translate_branch_name(&user_input)?
        };

        // Step 4: Format branch name using template
        let final_branch_name = BranchNaming::from_type_and_slug(
            branch_type.as_str(),
            &branch_name_slug,
            ticket_id.as_deref(),
        )?;

        // Step 5: Determine base branch
        let base_branch = if from_default {
            // --from-default: create from default branch
            Some(GitBranch::get_default_branch()?)
        } else {
            // Otherwise: create from current branch
            None
        };

        // Step 6: In dry-run mode, only verify that we can switch to base branch if needed
        if dry_run {
            if let Some(base) = &base_branch {
                // Verify that the base branch exists
                let (exists_local, exists_remote) = GitBranch::is_branch_exists(base)
                    .context("Failed to check if base branch exists")?;
                if !exists_local && !exists_remote {
                    anyhow::bail!("[DRY RUN] Base branch '{}' does not exist", base);
                }
                log_info!("[DRY RUN] Would switch to base branch: {}", base);
            }
            log_info!("[DRY RUN] Would create branch: {}", final_branch_name);
            return Ok(());
        }

        // Step 6: Switch to base branch if specified, or pull current branch if not
        if let Some(base) = &base_branch {
            Self::switch_to_base_branch(base)?;
        } else {
            // Pull latest changes from current branch
            Self::pull_current_branch()?;
        }

        log_success!("Creating branch: {}", final_branch_name);
        GitBranch::checkout_branch(&final_branch_name)?;

        log_success!("Branch '{}' created successfully!", final_branch_name);
        Ok(())
    }

    /// Resolve branch type (interactive)
    ///
    /// Priority:
    /// 1. If repository prefix exists, use it as branch type (skip selection)
    /// 2. Otherwise, prompt user to select interactively
    fn resolve_branch_type() -> Result<BranchType> {
        BranchType::resolve_with_repo_prefix()
    }

    /// Resolve ticket ID (optional, interactive if not provided)
    fn resolve_ticket_id(ticket_id: Option<String>) -> Result<Option<String>> {
        let ticket = if let Some(t) = ticket_id {
            let trimmed = t.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        } else {
            // Interactive prompt (optional)
            let input = InputDialog::new("Jira ticket (optional)")
                .allow_empty(true)
                .prompt()
                .context("Failed to get Jira ticket")?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        };

        // Validate format (if ticket provided)
        if let Some(ref ticket) = ticket {
            validate_jira_ticket_format(ticket)?;
        }

        Ok(ticket)
    }

    /// Resolve branch name (interactive, required if no ticket_id)
    fn resolve_branch_name() -> Result<String> {
        // Interactive prompt (required)
        let name = InputDialog::new("Branch name (required)")
            .with_validator(|input: &str| {
                if input.trim().is_empty() {
                    Err("Branch name is required and cannot be empty".to_string())
                } else {
                    Ok(())
                }
            })
            .prompt()
            .context("Failed to get branch name")?;
        Ok(name.trim().to_string())
    }

    /// Generate branch name from JIRA ticket using LLM
    ///
    /// Gets JIRA ticket info and uses LLM to generate a branch name slug.
    fn generate_branch_name_from_jira(ticket_id: &str) -> Result<String> {
        // Get JIRA ticket info
        let issue = Spinner::with(format!("Getting ticket info for {}...", ticket_id), || {
            Jira::get_ticket_info(ticket_id)
        })
        .with_context(|| format!("Failed to get ticket info for {}", ticket_id))?;

        // Use LLM to generate branch name
        let exists_branches = GitBranch::get_all_branches(true).ok();
        let git_diff = None;

        match PullRequestLLM::generate(&issue.fields.summary, exists_branches, git_diff) {
            Ok(content) => {
                log_success!("Generated branch name using LLM: {}", content.branch_name);
                // Return just the slug part (without prefix)
                Ok(BranchNaming::sanitize(&content.branch_name))
            }
            Err(e) => {
                log_warning!(
                    "Failed to generate branch name using LLM: {}, using summary slug",
                    e
                );
                // Fallback to simple slug from summary
                Ok(BranchNaming::slugify(&issue.fields.summary))
            }
        }
    }

    /// Switch to base branch
    ///
    /// Switches to the specified base branch, stashing uncommitted changes if needed,
    /// and pulls latest changes.
    fn switch_to_base_branch(from_branch: &str) -> Result<()> {
        let current_branch = GitBranch::current_branch()?;

        // If already on target branch, just pull latest changes
        if current_branch == from_branch {
            log_info!("Already on branch '{}'", from_branch);
            // Check if remote branch exists and pull
            if GitBranch::has_remote_branch(from_branch)? {
                Self::pull_with_stash(
                    from_branch,
                    &format!("Auto-stash before pulling {}", from_branch),
                )?;
            }
            return Ok(());
        }

        // Check if branch exists
        let (exists_local, exists_remote) = GitBranch::is_branch_exists(from_branch)?;

        if !exists_local && !exists_remote {
            anyhow::bail!("Branch '{}' does not exist", from_branch);
        }

        // Check if there are uncommitted changes and stash if needed
        let has_uncommitted = GitCommit::has_commit()
            .context("Failed to check uncommitted changes before switching branch")?;
        let has_stashed = if has_uncommitted {
            log_info!("Stashing uncommitted changes before switching branch...");
            GitStash::stash_push(Some(&format!(
                "Auto-stash before switching to {}",
                from_branch
            )))?;
            true
        } else {
            false
        };

        // Switch to base branch
        log_info!("Switching to branch '{}'...", from_branch);
        if let Err(e) = GitBranch::checkout_branch(from_branch)
            .with_context(|| format!("Failed to checkout branch: {}", from_branch))
        {
            // If checkout fails, try to restore stash
            if has_stashed {
                handle_stash_pop_result(GitStash::stash_pop());
            }
            return Err(e);
        }

        // Pull latest changes (if remote branch exists)
        if exists_remote {
            Self::pull_with_stash(
                from_branch,
                &format!("Auto-stash before pulling {}", from_branch),
            )?;
        }

        // Restore stash if we stashed changes
        if has_stashed {
            log_info!("Restoring stashed changes...");
            handle_stash_pop_result(GitStash::stash_pop());
        }

        Ok(())
    }

    /// Pull latest changes from current branch
    ///
    /// Asks user if they want to pull latest changes, then stashes uncommitted changes if needed,
    /// pulls latest changes, and restores stash.
    fn pull_current_branch() -> Result<()> {
        let current_branch = GitBranch::current_branch()?;

        // Check if remote branch exists
        let has_remote = GitBranch::has_remote_branch(&current_branch)
            .context("Failed to check if remote branch exists")?;

        if !has_remote {
            log_info!("No remote branch for '{}', skipping pull", current_branch);
            return Ok(());
        }

        // Ask user if they want to pull latest changes
        let should_pull =
            ConfirmDialog::new(format!("Pull latest changes from '{}'?", current_branch))
                .with_default(true)
                .prompt()
                .context("Failed to confirm pull")?;

        if !should_pull {
            log_info!("Skipping pull, using current branch state");
            return Ok(());
        }

        // Pull with stash handling
        Self::pull_with_stash(
            &current_branch,
            &format!("Auto-stash before pulling {}", current_branch),
        )?;

        Ok(())
    }

    /// Pull latest changes with automatic stash handling
    ///
    /// Stashes uncommitted changes if needed, pulls latest changes, and restores stash.
    /// This is a helper function to avoid code duplication.
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Branch name to pull from
    /// * `stash_message` - Message for stash (if needed)
    fn pull_with_stash(branch_name: &str, stash_message: &str) -> Result<()> {
        // Check if there are uncommitted changes and stash if needed
        let has_uncommitted = GitCommit::has_commit()
            .context("Failed to check uncommitted changes before pulling")?;
        let has_stashed = if has_uncommitted {
            log_info!("Stashing uncommitted changes before pulling latest changes...");
            GitStash::stash_push(Some(stash_message))?;
            true
        } else {
            false
        };

        // Pull latest changes
        log_info!("Pulling latest changes from '{}'...", branch_name);
        if let Err(e) = GitBranch::pull(branch_name)
            .with_context(|| format!("Failed to pull latest changes from {}", branch_name))
        {
            // If pull fails, try to restore stash
            if has_stashed {
                handle_stash_pop_result(GitStash::stash_pop());
            }
            return Err(e);
        }

        // Restore stash if we stashed changes
        if has_stashed {
            log_info!("Restoring stashed changes...");
            handle_stash_pop_result(GitStash::stash_pop());
        }

        Ok(())
    }
}
