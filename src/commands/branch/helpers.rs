//! Branch command helper functions
//!
//! Provides reusable helper functions for branch-related operations.

use crate::base::dialog::SelectDialog;
use crate::git::GitBranch;
use crate::repo::config::RepoConfig;
use anyhow::{Context, Result};

/// Sort branches with priority
///
/// Sorts branches according to the following priority:
/// 1. main, master (in that order)
/// 2. develop
/// 3. Branches with the same prefix as current branch (if current branch has a prefix)
///    OR branches with the configured repository prefix (if current branch has no prefix)
/// 4. Other branches (alphabetically)
///
/// # Arguments
///
/// * `branches` - List of branch names to sort
///
/// # Returns
///
/// Returns sorted branch list
pub fn sort_branches_with_priority(mut branches: Vec<String>) -> Result<Vec<String>> {
    // Get current branch to extract prefix
    let current_prefix = GitBranch::current_branch().ok().and_then(|current| {
        // Extract prefix from current branch (e.g., "zw/feature-branch" -> "zw")
        if current.contains('/') {
            current.split('/').next().map(|s| s.to_string())
        } else {
            None
        }
    });

    // If current branch has no prefix, try to get prefix from repository configuration
    let prefix_to_use = if current_prefix.is_some() {
        current_prefix
    } else {
        // Try to get repository prefix from configuration
        RepoConfig::get_branch_prefix()
    };

    // Sort branches with priority
    branches.sort_by(|a, b| {
        let priority_a = get_branch_priority(a, prefix_to_use.as_deref());
        let priority_b = get_branch_priority(b, prefix_to_use.as_deref());
        priority_a.cmp(&priority_b)
    });

    Ok(branches)
}

/// Get branch priority for sorting
///
/// Returns a tuple (priority, sort_key) where:
/// - priority: 1 for main/master, 2 for develop, 3 for prefix branches, 4 for others
/// - sort_key: used for secondary sorting within the same priority
fn get_branch_priority(branch: &str, current_prefix: Option<&str>) -> (usize, String) {
    // Priority 1: main, master (in that order)
    match branch {
        "main" => return (1, "0-main".to_string()),
        "master" => return (1, "1-master".to_string()),
        _ => {}
    }

    // Priority 2: develop
    if branch == "develop" {
        return (2, branch.to_string());
    }

    // Priority 3: branches with the same prefix as current branch or configured prefix
    if let Some(prefix) = current_prefix {
        let prefix_with_slash = format!("{}/", prefix);
        if branch.starts_with(&prefix_with_slash) {
            return (3, branch.to_string());
        }
    }

    // Priority 4: other branches (alphabetically)
    (4, branch.to_string())
}

/// Options for branch selection
#[derive(Debug, Clone)]
pub struct BranchSelectionOptions {
    /// Whether to include current branch in the list
    pub include_current: bool,
    /// Whether to mark current branch with [current] marker
    pub mark_current: bool,
    /// Custom prompt message
    pub prompt: Option<String>,
    /// Default selection index (None means no default)
    pub default_index: Option<usize>,
}

impl Default for BranchSelectionOptions {
    fn default() -> Self {
        Self {
            include_current: true,
            mark_current: false,
            prompt: None,
            default_index: None,
        }
    }
}

impl BranchSelectionOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Exclude current branch from the list
    pub fn exclude_current(mut self) -> Self {
        self.include_current = false;
        self
    }

    /// Mark current branch with [current] marker
    pub fn mark_current_branch(mut self) -> Self {
        self.mark_current = true;
        self
    }

    /// Set custom prompt message
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Set default selection index
    pub fn with_default_index(mut self, index: usize) -> Self {
        self.default_index = Some(index);
        self
    }
}

/// Select a branch interactively
///
/// This function provides a unified way to select branches across different commands.
/// Fuzzy matching is always enabled for better search experience.
///
/// # Arguments
///
/// * `options` - Selection options (use `BranchSelectionOptions::new()` for defaults)
///
/// # Returns
///
/// Returns the selected branch name (without any markers)
///
/// # Behavior
///
/// - Gets all branches (local + remote, deduplicated)
/// - Filters branches based on options
/// - Always uses fuzzy matching (SkimMatcherV2) for search
/// - Returns the selected branch name
///
/// # Examples
///
/// ## Basic usage (for switch command)
/// ```rust,no_run
/// use workflow::commands::branch::helpers::{select_branch, BranchSelectionOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let branch = select_branch(
///     BranchSelectionOptions::new()
///         .mark_current_branch()
///         .with_default_index(0)
///         .with_prompt("Select branch to switch to")
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// ## Exclude current branch (for rename command)
/// ```rust,no_run
/// use workflow::commands::branch::helpers::{select_branch, BranchSelectionOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let branch = select_branch(
///     BranchSelectionOptions::new()
///         .exclude_current()
///         .with_prompt("Select branch to rename")
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn select_branch(options: BranchSelectionOptions) -> Result<String> {
    // Get all branches (local + remote, deduplicated)
    let all_branches = GitBranch::get_all_branches(false).context("Failed to get branch list")?;

    if all_branches.is_empty() {
        anyhow::bail!("No branches available");
    }

    // Get current branch (if needed)
    let current_branch = if !options.include_current || options.mark_current {
        GitBranch::current_branch().ok()
    } else {
        None
    };

    // Prepare branch list
    let mut branch_options: Vec<String> = if options.include_current {
        all_branches
    } else {
        // Exclude current branch
        all_branches
            .into_iter()
            .filter(|b| {
                if let Some(ref current) = current_branch {
                    b != current
                } else {
                    true
                }
            })
            .collect()
    };

    // Sort branches with priority
    branch_options =
        sort_branches_with_priority(branch_options).context("Failed to sort branches")?;

    // Mark current branch if needed
    if options.mark_current {
        if let Some(ref current) = current_branch {
            let current_marker = format!("{} [current]", current);
            // Remove current branch from its original position
            branch_options.retain(|b| b != current);
            // Insert at the top
            branch_options.insert(0, current_marker);
        }
    }

    // Build prompt message (fuzzy matching is always enabled)
    let prompt = if let Some(custom_prompt) = &options.prompt {
        format!("{} (type to search)", custom_prompt)
    } else {
        "Select branch (type to search)".to_string()
    };

    // Create dialog (fuzzy matching is enabled by default)
    let mut dialog = SelectDialog::new(&prompt, branch_options);

    // Set default index if provided
    if let Some(default_idx) = options.default_index {
        dialog = dialog.with_default(default_idx);
    }

    // Prompt user
    let selected = dialog.prompt().context("Failed to select branch")?;

    // Extract branch name (remove [current] marker if present)
    let branch_name = selected.strip_suffix(" [current]").unwrap_or(&selected).to_string();

    Ok(branch_name)
}
