//! Branch rename command
//!
//! Rename a local branch, optionally update the remote branch.
//! Provides interactive workflow following the implementation document.

use crate::base::constants::validation::branch;
use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::commands::branch::helpers::{select_branch, BranchSelectionOptions};
use crate::commands::check;
use crate::git::GitBranch;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

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

        log_break!();
        log_message!("{}", crate::base::constants::messages::log::BRANCH_RENAME);

        // Select branch to rename (fully interactive)
        let branch_to_rename = Self::select_branch_to_rename()?;

        // Input and validate new branch name (fully interactive)
        let new_branch_name = Self::input_and_validate_new_name(&branch_to_rename)?;

        // Check if new name is same as old name
        if branch_to_rename == new_branch_name {
            log_info!("New branch name is the same as old branch name, no rename needed");
            return Ok(());
        }

        // Preview and confirm
        Self::preview_and_confirm(&branch_to_rename, &new_branch_name)?;

        // Execute rename (fully interactive remote branch handling)
        Self::execute_rename(&branch_to_rename, &new_branch_name)?;

        log_break!();
        log_success!("Branch rename completed!");

        Ok(())
    }

    /// Select branch to rename (fully interactive)
    fn select_branch_to_rename() -> Result<String> {
        // Interactive selection: ask if rename current branch first
        let current_branch =
            GitBranch::current_branch().wrap_err("Failed to get current branch")?;

        let rename_current =
            ConfirmDialog::new(format!("Rename current branch '{}'?", current_branch))
                .with_default(true)
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
                InputDialog::new(&prompt).prompt().wrap_err("Failed to get new branch name")?;

            // Validate new branch name
            // 1. Validate branch name format
            if let Err(e) = Self::validate_branch_name(&new_name) {
                log_warning!("{}", e);
                log_info!("Please enter a valid branch name");
                continue;
            }

            // 2. Check if exists locally
            let (exists_local, _) =
                GitBranch::is_branch_exists(&new_name).wrap_err("Failed to check branch")?;

            if exists_local {
                log_warning!("⚠️  Error: Branch '{}' already exists locally", new_name);
                log_info!("Git does not allow renaming to an existing branch name.");
                log_info!("Please enter a different branch name.");
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
            log_warning!(
                "⚠️  Warning: You are renaming the default branch '{}'",
                old_branch_name
            );
            log_message!("");
            log_message!("Renaming the default branch may affect:");
            log_message!("  - Repository default branch settings");
            log_message!("  - CI/CD configurations");
            log_message!("  - Other tools that depend on the default branch");
            log_message!("");
            if !ConfirmDialog::new("Confirm to continue renaming the default branch?")
                .with_default(false)
                .with_cancel_message("Operation cancelled")
                .prompt()
                .wrap_err("Failed to get confirmation")?
            {
                return Ok(());
            }
            log_break!();
        }

        // Display preview information
        let current_branch = GitBranch::current_branch().ok();
        let is_current = current_branch.as_deref() == Some(old_branch_name);

        let (exists_local, exists_remote) = GitBranch::is_branch_exists(old_branch_name)
            .wrap_err("Failed to check branch status")?;

        // Check remote tracking
        let has_remote_tracking = Self::check_remote_tracking(old_branch_name)?;

        log_break!();
        log_break!('━', 80, "Branch Rename Preview");
        log_message!("");
        log_message!("  Old branch name:  {}", old_branch_name);
        log_message!("  New branch name:  {}", new_branch_name);
        log_message!(
            "  Is current branch:  {}",
            if is_current { "Yes ✓" } else { "No" }
        );
        log_message!("");
        log_message!("  Remote branch status:");
        log_message!(
            "    - Local branch:  {}",
            if exists_local {
                "Exists ✓"
            } else {
                "Not exists"
            }
        );
        log_message!(
            "    - Remote branch:  {}",
            if exists_remote {
                format!("Exists ✓ (origin/{})", old_branch_name)
            } else {
                "Not exists".to_string()
            }
        );
        log_message!(
            "    - Remote tracking:  {}",
            if has_remote_tracking {
                "Set ✓"
            } else {
                "Not set"
            }
        );
        log_message!("");
        log_break!('━', 80);

        // Final confirmation
        ConfirmDialog::new("Confirm to execute branch rename?")
            .with_default(true)
            .with_cancel_message("Operation cancelled")
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
        log_success!(
            "✓ Renamed local branch: {} -> {}",
            old_branch_name,
            new_branch_name
        );

        // Handle remote branch
        let exists_remote = GitBranch::has_remote_branch(old_branch_name)
            .wrap_err("Failed to check remote branch")?;

        if exists_remote {
            // Display warning information
            log_break!();
            log_warning!(
                "⚠️  Important: Remote branch 'origin/{}' exists",
                old_branch_name
            );
            log_message!("");
            log_message!("Renaming remote branch will affect:");
            log_message!("  - Other collaborators need to update local branch references");
            log_message!("  - Existing PRs may need to be updated");
            log_message!("  - CI/CD configurations may need to be updated");
            log_message!("");
            log_message!("Please ensure team members are notified.");
            log_break!();

            // Ask if update remote branch (fully interactive)
            let should_rename_remote = ConfirmDialog::new("Also rename remote branch?")
                .with_default(false)
                .with_cancel_message("Operation cancelled")
                .prompt()
                .wrap_err("Failed to get user confirmation")?;

            if should_rename_remote {
                // Second confirmation
                log_break!();
                log_warning!("⚠️  Final confirmation: This will perform the following operations:");
                log_message!("");
                log_message!("  1. Push new branch '{}' to remote", new_branch_name);
                log_message!("  2. Delete remote branch 'origin/{}'", old_branch_name);
                log_message!("  3. Update local branch remote tracking settings");
                log_message!("");
                log_message!("This operation cannot be undone. Continue?");
                log_break!();

                if ConfirmDialog::new("Confirm to continue?")
                    .with_default(false)
                    .with_cancel_message("Operation cancelled")
                    .prompt()
                    .wrap_err("Failed to get final confirmation")?
                {
                    GitBranch::rename_remote(old_branch_name, new_branch_name)
                        .wrap_err("Failed to rename remote branch")?;
                    log_success!(
                        "✓ Renamed remote branch: origin/{} -> origin/{}",
                        old_branch_name,
                        new_branch_name
                    );
                } else {
                    log_info!("ℹ️  Remote branch not renamed");
                    log_info!("To manually update remote branch, execute:");
                    log_info!("  git push origin -u {}", new_branch_name);
                    log_info!("  git push origin --delete {}", old_branch_name);
                }
            } else {
                log_info!("ℹ️  Remote branch not renamed");
                log_info!("To manually update remote branch, execute:");
                log_info!("  git push origin -u {}", new_branch_name);
                log_info!("  git push origin --delete {}", old_branch_name);
            }
        }

        // Completion message
        log_break!();
        log_break!('━', 80, "Branch Rename Completed");
        log_message!("");
        log_success!(
            "  ✓ Local branch renamed: {} -> {}",
            old_branch_name,
            new_branch_name
        );
        if exists_remote {
            log_success!(
                "  ✓ Remote branch renamed: origin/{} -> origin/{}",
                old_branch_name,
                new_branch_name
            );
        }
        log_message!("");
        log_info!("  Note:");
        log_info!(
            "    - If other collaborators have checked out this branch, they need to execute:"
        );
        log_info!("      git fetch origin");
        log_info!(
            "      git branch -m {} {}",
            old_branch_name,
            new_branch_name
        );
        log_info!(
            "      git branch -u origin/{} {}",
            new_branch_name,
            new_branch_name
        );
        log_message!("");
        log_break!('━', 80);

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
    ///
    /// 使用 git2 库检查分支是否配置了远程跟踪。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名称
    ///
    /// # 返回
    ///
    /// 如果分支配置了远程跟踪，返回 `true`；否则返回 `false`。
    fn check_remote_tracking(branch_name: &str) -> Result<bool> {
        // 检查 branch.{branch_name}.remote 配置
        let config_key = format!("branch.{}.remote", branch_name);
        match crate::git::GitConfig::get_config_string(&config_key)? {
            Some(remote) => Ok(!remote.is_empty()),
            None => Ok(false), // 配置不存在，返回 false
        }
    }
}
