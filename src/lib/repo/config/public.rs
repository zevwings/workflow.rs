//! Public repository configuration management
//!
//! Manages project-level template configuration that should be committed to Git.
//! This configuration is stored in `.workflow/config.toml` in the project root.

use crate::base::settings::paths::Paths;
use crate::base::util::file::{FileReader, FileWriter};
use crate::base::util::path::PathAccess;
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use toml::map::Map;
use toml::Value;

/// Project configuration manager (only manages template project-level configuration, committed to Git)
///
/// Manages `.workflow/config.toml` file, containing only template configuration and project standards.
///
/// This struct contains the configuration data and provides methods to load and save it.
///
/// **Note**: This struct should not be used directly from outside the `repo::config` module.
/// Use `RepoConfig` methods instead.
#[derive(Debug, Default)]
pub struct PublicRepoConfig {
    /// Template commit configuration
    pub template_commit: Map<String, Value>,
    /// Template branch configuration
    pub template_branch: Map<String, Value>,
    /// Template pull requests configuration
    pub template_pull_requests: Map<String, Value>,
}

impl PublicRepoConfig {
    /// Load project configuration
    ///
    /// Loads template configuration from `.workflow/config.toml`.
    pub fn load() -> Result<Self> {
        let path = Paths::project_config().wrap_err("Failed to get project config path")?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let value: Value = FileReader::new(&path).toml()?;

        let mut config = Self::default();

        // Parse [template.commit]
        if let Some(template_section) = value.get("template") {
            if let Some(commit_section) = template_section.get("commit") {
                if let Some(table) = commit_section.as_table() {
                    for (key, value) in table {
                        config.template_commit.insert(key.clone(), value.clone());
                    }
                }
            }
            // Parse [template.branch]
            if let Some(branch_section) = template_section.get("branch") {
                if let Some(table) = branch_section.as_table() {
                    for (key, value) in table {
                        config.template_branch.insert(key.clone(), value.clone());
                    }
                }
            }
            // Parse [template.pull_requests]
            if let Some(pull_requests_section) = template_section.get("pull_requests") {
                if let Some(table) = pull_requests_section.as_table() {
                    for (key, value) in table {
                        config.template_pull_requests.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        Ok(config)
    }

    /// Save project configuration
    ///
    /// Saves to `.workflow/config.toml` (only saves template configuration and project standards).
    pub fn save(&self) -> Result<()> {
        let path = Paths::project_config().wrap_err("Failed to get project config path")?;

        PathAccess::new(&path).ensure_parent_dir_exists()?;

        let mut existing_value: Value =
            if path.exists() { FileReader::new(&path).toml()? } else { Value::Table(Map::new()) };

        if let Some(table) = existing_value.as_table_mut() {
            // Update [template] section
            let template_table =
                table.entry("template".to_string()).or_insert_with(|| Value::Table(Map::new()));

            if let Some(template_map) = template_table.as_table_mut() {
                // Update [template.commit]
                let commit_table = template_map
                    .entry("commit".to_string())
                    .or_insert_with(|| Value::Table(Map::new()));

                if let Some(commit_map) = commit_table.as_table_mut() {
                    for (key, value) in &self.template_commit {
                        commit_map.insert(key.clone(), value.clone());
                    }
                }

                // Update [template.branch]
                if !self.template_branch.is_empty() {
                    let branch_table = template_map
                        .entry("branch".to_string())
                        .or_insert_with(|| Value::Table(Map::new()));

                    if let Some(branch_map) = branch_table.as_table_mut() {
                        for (key, value) in &self.template_branch {
                            branch_map.insert(key.clone(), value.clone());
                        }
                    }
                }

                // Update [template.pull_requests]
                if !self.template_pull_requests.is_empty() {
                    let pull_requests_table = template_map
                        .entry("pull_requests".to_string())
                        .or_insert_with(|| Value::Table(Map::new()));

                    if let Some(pull_requests_map) = pull_requests_table.as_table_mut() {
                        for (key, value) in &self.template_pull_requests {
                            pull_requests_map.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        FileWriter::new(&path).write_toml(&existing_value)?;

        Ok(())
    }
}
