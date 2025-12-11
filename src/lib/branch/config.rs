//! Branch configuration management
//!
//! Provides branch-related configuration management, including:
//! - Branch prefix configuration (per repository)
//! - Branch ignore list (per repository)
//! - Configuration file persistence

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

use crate::base::dialog::InputDialog;
use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use crate::jira::config::ConfigManager;
use crate::{log_info, log_success, log_warning};

/// Helper function: for skip_serializing_if, skip serialization when value is false
fn is_false(b: &bool) -> bool {
    !b
}

/// Branch configuration structure
///
/// Manages branch-related configuration, including ignore lists and branch prefixes,
/// grouped by repository name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchConfig {
    /// Configuration grouped by repository name
    /// Key: repository name (e.g., "owner/repo")
    /// Value: repository branch configuration
    #[serde(flatten)]
    pub repositories: HashMap<String, RepositoryConfig>,
}

/// Repository configuration
///
/// Branch-related configuration for each repository, including ignore branch list and branch prefix.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Branch prefix (optional)
    ///
    /// Used to add prefix when generating branch names, e.g., "feature", "fix", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_prefix: Option<String>,
    /// List of branches to ignore
    ///
    /// These branches will not be deleted during branch cleanup.
    /// Supports backward compatibility: can read old "ignore" field name.
    #[serde(default, alias = "ignore")]
    pub branch_ignore: Vec<String>,
    /// Whether branch prefix has been prompted (to avoid duplicate prompts)
    ///
    /// This is an internal field to record whether the user has been prompted to set branch prefix.
    /// When the value is false, it will not be serialized to the configuration file.
    #[serde(default, skip_serializing_if = "is_false")]
    pub branch_prefix_prompted: bool,
}

impl RepositoryConfig {
    /// Validate repository name format
    ///
    /// Validates that the repository name follows the `owner/repo` format.
    /// This is used to ensure repository names are in the correct format before
    /// using them as keys in `BranchConfig::repositories`.
    ///
    /// # Arguments
    /// * `repo_name` - Repository name to validate
    ///
    /// # Returns
    /// Returns `Ok(())` if format is correct, otherwise returns an error.
    ///
    /// # Example
    /// ```
    /// use workflow::branch::config::RepositoryConfig;
    /// assert!(RepositoryConfig::validate_name("owner/repo").is_ok());
    /// assert!(RepositoryConfig::validate_name("invalid").is_err());
    /// ```
    pub fn validate_name(repo_name: &str) -> Result<()> {
        let parts: Vec<&str> = repo_name.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid repository name format: '{}'. Expected format: 'owner/repo'",
                repo_name
            );
        }

        let owner = parts[0];
        let repo = parts[1];

        if owner.is_empty() {
            anyhow::bail!(
                "Invalid repository name format: '{}'. Owner cannot be empty",
                repo_name
            );
        }

        if repo.is_empty() {
            anyhow::bail!(
                "Invalid repository name format: '{}'. Repository name cannot be empty",
                repo_name
            );
        }

        Ok(())
    }
}

impl BranchConfig {
    /// Get configuration file path
    pub fn config_path() -> Result<PathBuf> {
        Paths::branch_config()
    }

    /// Load configuration file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let manager = ConfigManager::<Self>::new(path);
        manager.read()
    }

    /// Save configuration file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let manager = ConfigManager::<Self>::new(path);
        manager.write(self)
    }

    /// Get branch prefix for specified repository
    ///
    /// # Arguments
    /// * `repo_name` - Repository name (format: `owner/repo`)
    ///
    /// # Returns
    /// If the repository has `branch_prefix` configured, returns a reference to the prefix string;
    /// otherwise returns `None`.
    pub fn get_branch_prefix_for_repo(&self, repo_name: &str) -> Option<&str> {
        self.repositories
            .get(repo_name)
            .and_then(|repo_config| repo_config.branch_prefix.as_deref())
    }

    /// Get branch prefix for current repository
    ///
    /// Gets owner/repo from current Git repository, then looks up corresponding configuration.
    /// If current repository has no prefix configured, returns None.
    ///
    /// # Returns
    /// If current repository has `branch_prefix` configured, returns a reference to the prefix string;
    /// otherwise returns `None`.
    ///
    /// # Errors
    /// Returns an error if unable to get current repository information.
    pub fn get_current_repo_branch_prefix(&self) -> Result<Option<&str>> {
        let repo_name = GitRepo::extract_repo_name().context("Failed to extract repo name")?;
        Ok(self.get_branch_prefix_for_repo(&repo_name))
    }

    /// Get ignore branch list for specified repository
    ///
    /// # Arguments
    /// * `repo_name` - Repository name (format: `owner/repo`)
    ///
    /// # Returns
    /// Returns a vector of branch names that should be ignored for the specified repository.
    pub fn get_ignore_branches(&self, repo_name: &str) -> Vec<String> {
        self.repositories
            .get(repo_name)
            .map(|repo| repo.branch_ignore.clone())
            .unwrap_or_default()
    }

    /// Add branch to ignore list for specified repository
    ///
    /// # Arguments
    /// * `repo_name` - Repository name (format: `owner/repo`)
    /// * `branch_name` - Branch name to add to ignore list
    ///
    /// # Returns
    /// Returns `Ok(true)` if the branch was newly added, `Ok(false)` if it already existed.
    pub fn add_ignore_branch(&mut self, repo_name: String, branch_name: String) -> Result<bool> {
        let repo = self.repositories.entry(repo_name).or_default();

        if repo.branch_ignore.contains(&branch_name) {
            return Ok(false); // Already exists
        }

        repo.branch_ignore.push(branch_name);
        Ok(true) // Newly added
    }

    /// Remove branch from ignore list for specified repository
    ///
    /// # Arguments
    /// * `repo_name` - Repository name (format: `owner/repo`)
    /// * `branch_name` - Branch name to remove from ignore list
    ///
    /// # Returns
    /// Returns `Ok(true)` if the branch was removed, `Ok(false)` if it was not found.
    pub fn remove_ignore_branch(&mut self, repo_name: &str, branch_name: &str) -> Result<bool> {
        if let Some(repo) = self.repositories.get_mut(repo_name) {
            if let Some(pos) = repo.branch_ignore.iter().position(|b| b == branch_name) {
                repo.branch_ignore.remove(pos);
                // If list is empty and no other configuration, remove entire repository config
                if repo.branch_ignore.is_empty()
                    && repo.branch_prefix.is_none()
                    && !repo.branch_prefix_prompted
                {
                    self.repositories.remove(repo_name);
                }
                return Ok(true); // Removed
            }
        }
        Ok(false) // Not found
    }

    /// Set branch prefix for specified repository
    ///
    /// # Arguments
    /// * `repo_name` - Repository name (format: `owner/repo`)
    /// * `prefix` - Branch prefix to set, or `None` to remove
    ///
    /// # Returns
    /// Returns `Ok(())` on success, error on failure
    pub fn set_branch_prefix_for_repo(
        &mut self,
        repo_name: String,
        prefix: Option<String>,
    ) -> Result<()> {
        let repo_config = self.repositories.entry(repo_name.clone()).or_default();

        if let Some(p) = prefix {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                repo_config.branch_prefix = None;
                log_success!("Branch prefix removed for repository: {}", repo_name);
            } else {
                repo_config.branch_prefix = Some(trimmed.to_string());
                log_success!(
                    "Branch prefix set to '{}' for repository: {}",
                    trimmed,
                    repo_name
                );
            }
        } else {
            repo_config.branch_prefix = None;
            log_success!("Branch prefix removed for repository: {}", repo_name);
        }

        Ok(())
    }

    // === Convenience methods for current repository operations ===
    // These methods operate on the current repository and handle loading/saving automatically

    /// Set branch prefix for current repository
    ///
    /// Loads the configuration, sets the prefix for the current repository, and saves it.
    ///
    /// # Arguments
    /// * `prefix` - Branch prefix, if None then get via interactive input
    ///
    /// # Returns
    /// Returns `Ok(())` on success, error on failure
    pub fn set_prefix_for_current_repo(prefix: Option<String>) -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        // Validate repository name format
        RepositoryConfig::validate_name(&repo_name).context("Invalid repository name format")?;

        let prefix = if let Some(p) = prefix {
            Some(p)
        } else {
            let input = InputDialog::new(
                "Enter branch prefix (e.g., 'feature', 'fix'), or press Enter to remove:",
            )
            .allow_empty(true)
            .prompt()
            .context("Failed to get branch prefix")?;
            if input.trim().is_empty() {
                None
            } else {
                Some(input)
            }
        };

        let mut config = Self::load().context("Failed to load branch config")?;
        config.set_branch_prefix_for_repo(repo_name, prefix)?;
        config.save().context("Failed to save branch config")?;
        Ok(())
    }

    /// Get branch prefix for current repository
    ///
    /// Loads the configuration and returns the prefix for the current repository.
    ///
    /// # Returns
    /// If current repository has branch prefix configured, returns the prefix string;
    /// otherwise returns None
    pub fn get_prefix_for_current_repo() -> Option<String> {
        let repo_name = match GitRepo::extract_repo_name() {
            Ok(name) => name,
            Err(_) => return None,
        };

        let config = match Self::load() {
            Ok(c) => c,
            Err(_) => return None,
        };

        config.get_branch_prefix_for_repo(&repo_name).map(|s| s.to_string())
    }

    /// Remove branch prefix for current repository
    ///
    /// Loads the configuration, removes the prefix for the current repository, and saves it.
    ///
    /// # Returns
    /// Returns `Ok(())` on success, error on failure
    pub fn remove_prefix_for_current_repo() -> Result<()> {
        Self::set_prefix_for_current_repo(Some(String::new()))
    }

    /// Check and prompt to set branch_prefix for current repository (if needed)
    ///
    /// When using commands that need to generate branch names for the first time in a repository,
    /// if branch_prefix is not configured, directly prompt user to enter prefix.
    ///
    /// # Notes
    /// - Only prompts in interactive environment (checks if TTY)
    /// - Each repository prompts only once (uses `branch_prefix_prompted` field in config file for persistence)
    /// - If already configured or already prompted, returns directly
    /// - **Does not interrupt calling flow**: Even if an error occurs (e.g., user presses Ctrl+C),
    ///   returns Ok(()), does not affect main flow
    pub fn check_and_prompt_prefix() -> Result<()> {
        // Check if in interactive environment
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Ok(()); // Non-interactive environment, skip prompt
        }

        // Get current repository name
        let repo_name = match GitRepo::extract_repo_name() {
            Ok(name) => name,
            Err(_) => return Ok(()), // Not in Git repository, skip
        };

        // Check if already configured
        let mut config = match Self::load() {
            Ok(c) => c,
            Err(_) => return Ok(()), // Load failed, skip prompt
        };
        if config.get_branch_prefix_for_repo(&repo_name).is_some() {
            return Ok(()); // Already configured, no need to prompt
        }

        // Check if already prompted (read from config file)
        let repo_config = config.repositories.get(&repo_name);
        if let Some(repo) = repo_config {
            if repo.branch_prefix_prompted {
                return Ok(()); // Already prompted, skip
            }
        }

        // Prompt user and directly enter input
        log_warning!(
            "Branch prefix is not configured for repository: {}",
            repo_name
        );
        log_info!("Branch prefix helps organize branches (e.g., 'feature/', 'fix/').");

        // Directly guide user input, no confirmation dialog needed
        let prefix = match InputDialog::new(
            "Branch prefix is not set. Enter branch prefix (e.g., 'feature'), or press Enter to skip:",
        )
        .allow_empty(true)
        .prompt()
        {
            Ok(value) => value,
            Err(_) => {
                // User cancelled input, treat as skip, don't output any prompt message
                // Mark as prompted to avoid prompting again next time
                let repo_config = config.repositories.entry(repo_name.clone()).or_default();
                repo_config.branch_prefix_prompted = true;
                let _ = config.save(); // Ignore save error
                return Ok(()); // Don't interrupt flow
            }
        };

        if !prefix.trim().is_empty() {
            // Set prefix (if error, log but don't interrupt flow)
            if let Err(e) = Self::set_prefix_for_current_repo(Some(prefix.trim().to_string())) {
                log_warning!(
                    "Failed to save branch prefix: {}. PR creation will continue without prefix.",
                    e
                );
            } else {
                log_success!("Branch prefix configured successfully!");
            }
        }

        // Whether user entered prefix or not, mark as prompted to avoid prompting again next time
        let repo_config = config.repositories.entry(repo_name).or_default();
        repo_config.branch_prefix_prompted = true;
        let _ = config.save(); // Ignore save error

        Ok(())
    }
}
