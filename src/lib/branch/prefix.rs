//! Branch prefix management
//!
//! Provides functionality for applying branch name prefixes (JIRA ticket and repository prefix).

use crate::branch::config::BranchConfig;
use anyhow::{Context, Result};

/// Branch prefix service
///
/// Provides methods for applying branch name prefixes.
/// Handles both JIRA ticket prefixes and repository-level branch prefixes.
pub struct BranchPrefix;

impl BranchPrefix {
    /// Apply branch name prefixes
    ///
    /// Unified handling of branch name prefix logic to avoid code duplication.
    /// First adds JIRA ticket prefix (if provided), then adds repository branch prefix (if configured).
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Base branch name
    /// * `jira_ticket` - Optional JIRA ticket ID
    /// * `repo_prefix_used_as_type` - Whether repository prefix was already used as branch type
    ///
    /// # Returns
    ///
    /// Complete branch name with prefixes applied
    ///
    /// # Example
    ///
    /// ```
    /// use workflow::branch::prefix::BranchPrefix;
    ///
    /// // Only base branch name
    /// let name = BranchPrefix::apply("feature-branch".to_string(), None, false)?;
    /// // Returns: "feature-branch"
    ///
    /// // With JIRA ticket
    /// let name = BranchPrefix::apply("feature-branch".to_string(), Some("PROJ-123"), false)?;
    /// // Returns: "PROJ-123-feature-branch"
    ///
    /// // With JIRA ticket and repository prefix (not used as type)
    /// // Assuming repository prefix = "user" is configured
    /// let name = BranchPrefix::apply("feature-branch".to_string(), Some("PROJ-123"), false)?;
    /// // Returns: "user/PROJ-123-feature-branch"
    ///
    /// // Repository prefix already used as branch type
    /// let name = BranchPrefix::apply("feature/PROJ-123-branch".to_string(), Some("PROJ-123"), true)?;
    /// // Returns: "feature/PROJ-123-branch" (no duplicate prefix)
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn apply(
        mut branch_name: String,
        jira_ticket: Option<&str>,
        repo_prefix_used_as_type: bool,
    ) -> Result<String> {
        // Check and prompt for branch_prefix if needed
        // Note: This function won't interrupt the flow even if it errors
        let _ = BranchConfig::check_and_prompt_prefix();

        // If JIRA ticket exists, add as prefix
        // But skip if branch_name already contains the ticket (to avoid duplication)
        // e.g., if template already generated "feature/WEW-801-{slug}", don't add "WEW-801-" again
        if let Some(ticket) = jira_ticket {
            // Check if branch_name already starts with the ticket
            // Format could be: "WEW-801-..." or "feature/WEW-801-..." or "WEW-801/..."
            let ticket_prefix1 = format!("{}-", ticket);
            let ticket_prefix2 = format!("{}/", ticket);
            if !branch_name.starts_with(&ticket_prefix1)
                && !branch_name.starts_with(&ticket_prefix2)
            {
                // Also check if it contains the ticket in the middle (e.g., "feature/WEW-801-...")
                if !branch_name.contains(&ticket_prefix1) && !branch_name.contains(&ticket_prefix2)
                {
                    branch_name = format!("{}-{}", ticket, branch_name);
                }
            }
        }

        // If repository-level branch_prefix exists, add prefix
        // But skip if:
        // 1. Repository prefix was already used as branch type (to avoid duplication)
        // 2. Branch name already starts with the prefix (to avoid duplication)
        if !repo_prefix_used_as_type {
            let branch_config = BranchConfig::load().context("Failed to load branch config")?;
            if let Some(prefix) = branch_config.get_current_repo_branch_prefix()? {
                let trimmed = prefix.trim();
                if !trimmed.is_empty() {
                    // Check if branch_name already starts with the prefix
                    // e.g., if prefix is "feature" and branch_name is "feature/WEW-801-...", skip
                    let prefix_with_slash = format!("{}/", trimmed);
                    if !branch_name.starts_with(&prefix_with_slash) {
                        branch_name = format!("{}/{}", trimmed, branch_name);
                    }
                }
            }
        }

        Ok(branch_name)
    }
}
