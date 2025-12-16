//! Repository configuration unified interface
//!
//! Provides a unified configuration access interface, internally calling
//! `PublicRepoConfig` and `PrivateRepoConfig`.

use crate::git::GitRepo;
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use toml::map::Map;
use toml::Value;

use super::private::PrivateRepoConfig;
use super::public::PublicRepoConfig;
use super::types::{BranchConfig, PullRequestsConfig};

/// Repository configuration
///
/// Contains all configuration properties from both public (project template)
/// and private (personal preference) configurations.
///
/// This is the main configuration structure that external code should use.
/// It combines:
/// - **Public configuration** (project template, committed to Git): template configurations
/// - **Private configuration** (personal preference, not committed to Git): personal settings
#[derive(Debug, Clone, Default)]
pub struct RepoConfig {
    // Public configuration (project template, committed to Git)
    /// Template commit configuration
    pub template_commit: Map<String, Value>,
    /// Template branch configuration
    pub template_branch: Map<String, Value>,
    /// Template pull requests configuration
    pub template_pull_requests: Map<String, Value>,

    // Private configuration (personal preference, not committed to Git)
    /// Whether the repository has been configured (marks if repo setup has been completed)
    ///
    /// **This is the single source of truth for checking if `repo setup` has been completed.**
    pub configured: bool,
    /// Branch configuration (personal preference)
    pub branch: Option<BranchConfig>,
    /// PR configuration (personal preference)
    pub pr: Option<PullRequestsConfig>,
}

impl RepoConfig {
    /// Check if repository configuration exists
    ///
    /// **Unified check**: This is the single source of truth for checking if `repo setup` has been completed.
    /// Always checks the `configured` field to determine if the repository has been configured.
    ///
    /// - **Project standard configuration** (`.workflow/config.toml`) is optional and doesn't need to be checked.
    /// - **Personal preference configuration** (`~/.workflow/config/repository.toml`) is checked via the `configured` field.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Repository has been configured (`configured = true`)
    /// - `Ok(false)` - Repository has not been configured (`configured = false` or not set)
    /// - `Err(_)` - Error checking configuration
    ///
    /// # Notes
    ///
    /// - If not in a Git repository, returns `Ok(true)` to skip the check
    /// - All code should use this method to check if `repo setup` has been completed
    /// - Do not check file existence or other methods to determine configuration status
    pub fn exists() -> Result<bool> {
        // 1. Check if in Git repository
        if !GitRepo::is_git_repo() {
            return Ok(true); // Not in Git repository, consider as "configured" (skip check)
        }

        // 2. Load personal preference configuration
        let config =
            PrivateRepoConfig::load().wrap_err("Failed to load repository personal config")?;

        // 3. Check if configured (unified check: always use PrivateRepoConfig.configured)
        Ok(config.configured)
    }

    /// Get branch prefix (only reads from PrivateRepoConfig, personal preference)
    ///
    /// # Returns
    ///
    /// If current repository has branch prefix configured, returns the prefix string;
    /// otherwise returns `None`.
    pub fn get_branch_prefix() -> Option<String> {
        if let Ok(config) = PrivateRepoConfig::load() {
            if let Some(ref branch_config) = config.branch {
                if let Some(ref prefix) = branch_config.prefix {
                    return Some(prefix.clone());
                }
            }
        }
        None
    }

    /// Get ignore branch list (only reads from PrivateRepoConfig, personal preference)
    ///
    /// # Returns
    ///
    /// Returns a vector of branch names that should be ignored for the current repository.
    pub fn get_ignore_branches() -> Vec<String> {
        if let Ok(config) = PrivateRepoConfig::load() {
            if let Some(ref branch_config) = config.branch {
                return branch_config.ignore.clone();
            }
        }
        Vec::new()
    }

    /// Get auto-accept change type (only reads from PrivateRepoConfig, personal preference)
    ///
    /// # Returns
    ///
    /// If auto-accept is enabled, returns `true`; otherwise returns `false`.
    pub fn get_auto_accept_change_type() -> bool {
        match PrivateRepoConfig::load() {
            Ok(config) => {
                if let Some(ref pr_config) = config.pr {
                    if let Some(auto_accept) = pr_config.auto_accept_change_type {
                        tracing::debug!(
                            "get_auto_accept_change_type: returning {}",
                            auto_accept
                        );
                        return auto_accept;
                    } else {
                        tracing::debug!("get_auto_accept_change_type: pr_config.auto_accept_change_type is None");
                    }
                } else {
                    tracing::debug!("get_auto_accept_change_type: config.pr is None");
                }
                false
            }
            Err(e) => {
                // 静默失败，返回默认值 false
                // 如果配置文件不存在或读取失败，不应该影响 PR 创建流程
                tracing::debug!("Failed to load repository config for auto_accept_change_type: {}", e);
                false
            }
        }
    }

    /// Get template commit configuration (only reads from PublicRepoConfig, project standard)
    pub fn get_template_commit() -> Map<String, Value> {
        PublicRepoConfig::load().map(|c| c.template_commit).unwrap_or_default()
    }

    /// Get template branch configuration (only reads from PublicRepoConfig, project standard)
    pub fn get_template_branch() -> Map<String, Value> {
        PublicRepoConfig::load().map(|c| c.template_branch).unwrap_or_default()
    }

    /// Get template pull_requests configuration (only reads from PublicRepoConfig, project standard)
    pub fn get_template_pull_requests() -> Map<String, Value> {
        PublicRepoConfig::load().map(|c| c.template_pull_requests).unwrap_or_default()
    }

    /// Load repository configuration
    ///
    /// Loads both public (project template) and private (personal preference) configurations
    /// and combines them into a single `RepoConfig` structure.
    ///
    /// # Returns
    ///
    /// Returns a `RepoConfig` containing all configuration properties.
    pub fn load() -> Result<Self> {
        // Load public configuration (project template)
        let public_config =
            PublicRepoConfig::load().wrap_err("Failed to load public repository config")?;

        // Load private configuration (personal preference)
        let private_config =
            PrivateRepoConfig::load().wrap_err("Failed to load private repository config")?;

        Ok(Self {
            // Public configuration
            template_commit: public_config.template_commit,
            template_branch: public_config.template_branch,
            template_pull_requests: public_config.template_pull_requests,
            // Private configuration
            configured: private_config.configured,
            branch: private_config.branch,
            pr: private_config.pr,
        })
    }

    /// Save repository configuration
    ///
    /// Saves both public (project template) and private (personal preference) configurations
    /// to their respective files.
    pub fn save(&self) -> Result<()> {
        // Save public configuration (project template)
        let public_config = PublicRepoConfig {
            template_commit: self.template_commit.clone(),
            template_branch: self.template_branch.clone(),
            template_pull_requests: self.template_pull_requests.clone(),
        };
        public_config.save().wrap_err("Failed to save public repository config")?;

        // Save private configuration (personal preference)
        let private_config = PrivateRepoConfig {
            configured: self.configured,
            branch: self.branch.clone(),
            pr: self.pr.clone(),
        };
        private_config.save().wrap_err("Failed to save private repository config")?;

        Ok(())
    }
}
