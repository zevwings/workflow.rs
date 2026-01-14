//! Branch rename command
//!
//! Rename a local branch, optionally update the remote branch.
//! Provides interactive workflow following the implementation document.

use crate::base::constants::validation::branch;
use crate::commands::branch::helpers::{select_branch, BranchSelectionOptions};
use crate::commands::check;
use crate::git::GitBranch;
use crate::{br, info, success, warning};
use color_eyre::{eyre::WrapErr, Result};
use std::process::Command;

// Git 保留分支名称常量
const GIT_RESERVED_NAMES: &[&str] = &["HEAD", "FETCH_HEAD", "MERGE_HEAD", "CHERRY_PICK_HEAD"];

/// Branch rename command
pub struct BranchRenameCommand;

impl BranchRenameCommand {
    /// Execute the branch rename command
    ///
    /// Fully interactive mode - all operations are done through interactive prompts.
    pub fn execute() -> Result<()> {
        // 1. Run checks
        check::CheckCommand::run_all()?;

        br!();
        info!("{}", crate::base::constants::messages::log::BRANCH_RENAME);

        // Select branch to rename (fully interactive)
        let branch_to_rename = Self::select_branch_to_rename()?;

        // Input and validate new branch name (fully interactive)
        let new_branch_name = Self::input_and_validate_new_name(&branch_to_rename)?;

        // Check if new name is same as old name
        if branch_to_rename == new_branch_name {
            info!("New branch name is the same as old branch name, no rename needed");
            return Ok(());
        }

        // Preview and confirm
        Self::preview_and_confirm(&branch_to_rename, &new_branch_name)?;

        // Execute rename (fully interactive remote branch handling)
        Self::execute_rename(&branch_to_rename, &new_branch_name)?;

        br!();
        success!("Branch rename completed!");

        Ok(())
    }

    /// Select branch to rename (fully interactive)
    fn select_branch_to_rename() -> Result<String> {
        // Interactive selection: ask if rename current branch first
        let current_branch =
            GitBranch::current_branch().wrap_err("Failed to get current branch")?;

        let rename_current = crate::confirm!("Rename current branch '{}'?", current_branch)
            .default(true)
            .prompt()
            .wrap_err("Failed to get user confirmation")?;

        if rename_current {
            Ok(current_branch)
        } else {
            // Select from branch list (exclude current branch) using shared helper
            select_branch(
                BranchSelectionOptions::new()
                    .exclude_current()
                    .with_prompt("Select branch to rename"),
            )
        }
    }

    /// Input and validate new branch name
    fn input_and_validate_new_name(old_branch_name: &str) -> Result<String> {
        loop {
            // Input new branch name
            let prompt = format!(
                "Enter new branch name:\n  Current branch: {}\n  New branch name: ",
                old_branch_name
            );

            let new_name =
                crate::input!(prompt).prompt().wrap_err("Failed to get new branch name")?;

            // Validate new branch name
            // 1. Validate branch name format
            if let Err(e) = Self::validate_branch_name(&new_name) {
                warning!("{}", e);
                info!("Please enter a valid branch name");
                continue;
            }

            // 2. Check if exists locally
            let (exists_local, _) =
                GitBranch::is_branch_exists(&new_name).wrap_err("Failed to check branch")?;

            if exists_local {
                warning!("⚠️  Error: Branch '{}' already exists locally", new_name);
                info!("Git does not allow renaming to an existing branch name.");
                info!("Please enter a different branch name.");
                continue; // Re-enter
            }

            // Validation passed
            return Ok(new_name);
        }
    }

    /// Preview and confirm
    fn preview_and_confirm(old_branch_name: &str, new_branch_name: &str) -> Result<()> {
        // Check if it's the default branch (needs extra warning)
        let default_branch = GitBranch::get_default_branch().ok();
        let is_default = default_branch.as_deref() == Some(old_branch_name);

        if is_default {
            warning!(
                "⚠️  Warning: You are renaming the default branch '{}'",
                old_branch_name
            );
            info!("");
            info!("Renaming the default branch may affect:");
            info!("  - Repository default branch settings");
            info!("  - CI/CD configurations");
            info!("  - Other tools that depend on the default branch");
            info!("");
            if !crate::confirm!("Confirm to continue renaming the default branch?")
                .default(false)
                .prompt()
                .wrap_err("Failed to get confirmation")?
            {
                return Ok(());
            }
            br!();
        }

        // Display preview information
        let current_branch = GitBranch::current_branch().ok();
        let is_current = current_branch.as_deref() == Some(old_branch_name);

        let (exists_local, exists_remote) = GitBranch::is_branch_exists(old_branch_name)
            .wrap_err("Failed to check branch status")?;

        // Check remote tracking
        let has_remote_tracking = Self::check_remote_tracking(old_branch_name)?;

        br!();
        br!('━', 80, "Branch Rename Preview");
        info!("");
        info!("  Old branch name:  {}", old_branch_name);
        info!("  New branch name:  {}", new_branch_name);
        info!(
            "  Is current branch:  {}",
            if is_current { "Yes ✓" } else { "No" }
        );
        info!("");
        info!("  Remote branch status:");
        info!(
            "    - Local branch:  {}",
            if exists_local {
                "Exists ✓"
            } else {
                "Not exists"
            }
        );
        info!(
            "    - Remote branch:  {}",
            if exists_remote {
                format!("Exists ✓ (origin/{})", old_branch_name)
            } else {
                "Not exists".to_string()
            }
        );
        info!(
            "    - Remote tracking:  {}",
            if has_remote_tracking {
                "Set ✓"
            } else {
                "Not set"
            }
        );
        info!("");
        br!('━', 80);

        // Final confirmation
        crate::confirm!("Confirm to execute branch rename?")
            .default(true)
            .prompt()
            .wrap_err("Failed to get confirmation")?;

        Ok(())
    }

    /// Execute rename (fully interactive)
    fn execute_rename(old_branch_name: &str, new_branch_name: &str) -> Result<()> {
        // Rename local branch
        let is_current_branch = old_branch_name == GitBranch::current_branch()?;
        if is_current_branch {
            GitBranch::rename(None, new_branch_name).wrap_err("Failed to rename current branch")?;
        } else {
            GitBranch::rename(Some(old_branch_name), new_branch_name)
                .wrap_err("Failed to rename branch")?;
        }
        success!(
            "✓ Renamed local branch: {} -> {}",
            old_branch_name,
            new_branch_name
        );

        // Handle remote branch
        let exists_remote = GitBranch::has_remote_branch(old_branch_name)
            .wrap_err("Failed to check remote branch")?;

        if exists_remote {
            // Display warning information
            br!();
            warning!(
                "⚠️  Important: Remote branch 'origin/{}' exists",
                old_branch_name
            );
            info!("");
            info!("Renaming remote branch will affect:");
            info!("  - Other collaborators need to update local branch references");
            info!("  - Existing PRs may need to be updated");
            info!("  - CI/CD configurations may need to be updated");
            info!("");
            info!("Please ensure team members are notified.");
            br!();

            // Ask if update remote branch (fully interactive)
            let should_rename_remote = crate::confirm!("Also rename remote branch?")
                .default(false)
                .prompt()
                .wrap_err("Failed to get user confirmation")?;

            if should_rename_remote {
                // Second confirmation
                br!();
                warning!("⚠️  Final confirmation: This will perform the following operations:");
                info!("");
                info!("  1. Push new branch '{}' to remote", new_branch_name);
                info!("  2. Delete remote branch 'origin/{}'", old_branch_name);
                info!("  3. Update local branch remote tracking settings");
                info!("");
                info!("This operation cannot be undone. Continue?");
                br!();

                if crate::confirm!("Confirm to continue?")
                    .default(false)
                    .prompt()
                    .wrap_err("Failed to get final confirmation")?
                {
                    GitBranch::rename_remote(old_branch_name, new_branch_name)
                        .wrap_err("Failed to rename remote branch")?;
                    success!(
                        "✓ Renamed remote branch: origin/{} -> origin/{}",
                        old_branch_name,
                        new_branch_name
                    );
                } else {
                    info!("ℹ️  Remote branch not renamed");
                    info!("To manually update remote branch, execute:");
                    info!("  git push origin -u {}", new_branch_name);
                    info!("  git push origin --delete {}", old_branch_name);
                }
            } else {
                info!("ℹ️  Remote branch not renamed");
                info!("To manually update remote branch, execute:");
                info!("  git push origin -u {}", new_branch_name);
                info!("  git push origin --delete {}", old_branch_name);
            }
        }

        // Completion message
        br!();
        br!('━', 80, "Branch Rename Completed");
        info!("");
        success!(
            "  ✓ Local branch renamed: {} -> {}",
            old_branch_name,
            new_branch_name
        );
        if exists_remote {
            success!(
                "  ✓ Remote branch renamed: origin/{} -> origin/{}",
                old_branch_name,
                new_branch_name
            );
        }
        info!("");
        info!("  Note:");
        info!("    - If other collaborators have checked out this branch, they need to execute:");
        info!("      git fetch origin");
        info!(
            "      git branch -m {} {}",
            old_branch_name, new_branch_name
        );
        info!(
            "      git branch -u origin/{} {}",
            new_branch_name, new_branch_name
        );
        info!("");
        br!('━', 80);

        Ok(())
    }

    /// Validate branch name according to Git rules
    ///
    /// # Validation Rules
    /// 1. Cannot be empty
    /// 2. Cannot start or end with `.`
    /// 3. Cannot contain `..`
    /// 4. Cannot contain spaces
    /// 5. Cannot contain special characters: `~ ^ : ? * [ \`
    /// 6. Cannot end with `/`
    /// 7. Cannot contain consecutive slashes `//`
    /// 8. Cannot be reserved names: `HEAD`, `FETCH_HEAD`, `MERGE_HEAD`, `CHERRY_PICK_HEAD`
    pub fn validate_branch_name(name: &str) -> Result<()> {
        // 1. Cannot be empty
        if name.is_empty() {
            color_eyre::eyre::bail!("{}", branch::EMPTY_NAME);
        }

        // 2. Cannot start or end with `.`
        if name.starts_with('.') || name.ends_with('.') {
            color_eyre::eyre::bail!("{}", branch::INVALID_DOT_POSITION);
        }

        // 3. Cannot contain `..`
        if name.contains("..") {
            color_eyre::eyre::bail!("{}", branch::DOUBLE_DOT);
        }

        // 4. Cannot contain spaces
        if name.contains(' ') {
            color_eyre::eyre::bail!("{}", branch::CONTAINS_SPACES);
        }

        // 5. Cannot contain special characters: `~ ^ : ? * [ \`
        let invalid_chars = ['~', '^', ':', '?', '*', '[', '\\'];
        for &ch in &invalid_chars {
            if name.contains(ch) {
                color_eyre::eyre::bail!("{}: '{}'", branch::INVALID_SPECIAL_CHAR, ch);
            }
        }

        // 6. Cannot end with `/`
        if name.ends_with('/') {
            color_eyre::eyre::bail!("{}", branch::TRAILING_SLASH);
        }

        // 7. Cannot contain consecutive slashes `//`
        if name.contains("//") {
            color_eyre::eyre::bail!("{}", branch::DOUBLE_SLASH);
        }

        // 8. Cannot be reserved names
        if GIT_RESERVED_NAMES.contains(&name) {
            color_eyre::eyre::bail!("{}: '{}'", branch::RESERVED_NAME, name);
        }

        Ok(())
    }

    /// 检查分支是否有远程跟踪设置
    fn check_remote_tracking(branch_name: &str) -> Result<bool> {
        let output = Command::new("git")
            .args(["config", "--get", &format!("branch.{}.remote", branch_name)])
            .output()
            .wrap_err("Failed to check remote tracking")?;

        Ok(output.status.success() && !output.stdout.is_empty())
    }
}
