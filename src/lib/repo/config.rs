//! Repository configuration management
//!
//! Provides reusable functions for checking repository configuration.
//! This is a library function that can be called by commands.

use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use toml::map::Map;
use toml::Value;

/// Repository configuration
///
/// Provides reusable functions for checking repository configuration.
/// This is a library function that can be called by commands.
pub struct RepoConfig;

impl RepoConfig {
    /// Check if repository configuration exists
    ///
    /// Returns `true` if configuration exists and is complete, `false` otherwise.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Configuration exists and is complete
    /// - `Ok(false)` - Configuration doesn't exist or is incomplete
    /// - `Err(_)` - Error checking configuration
    pub fn exists() -> Result<bool> {
        // 1. Check if in Git repository
        if !GitRepo::is_git_repo() {
            return Ok(true); // Not in Git repository, consider as "configured" (skip check)
        }

        // 2. Check if project-level configuration exists
        let config_path = Paths::project_config()?;

        if !config_path.exists() {
            return Ok(false); // Configuration doesn't exist
        }

        // 3. Check if configuration has required sections
        Self::has_required_sections(&config_path)
    }

    /// Check if configuration has required sections
    fn has_required_sections(path: &PathBuf) -> Result<bool> {
        use std::fs;
        use toml::Value;

        let content = fs::read_to_string(path).context("Failed to read configuration file")?;

        let value: Value =
            toml::from_str(&content).context("Failed to parse configuration file")?;

        // 检查是否有 [template.commit] 或 [branch] 节
        let has_template = value.get("template").and_then(|t| t.get("commit")).is_some();

        let has_branch = value.get("branch").is_some();

        // 至少需要有一个配置节
        Ok(has_template || has_branch)
    }

    /// Load project configuration from file
    ///
    /// Reads and parses the project-level configuration file from `.workflow/config.toml`
    /// in the project root directory.
    ///
    /// # Returns
    ///
    /// Returns a `ProjectConfig` with parsed configuration, or default if file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails (but not if file doesn't exist).
    pub fn load() -> Result<ProjectConfig> {
        let path = Paths::project_config().context("Failed to get project config path")?;

        // 如果文件不存在，返回默认配置
        if !path.exists() {
            return Ok(ProjectConfig::default());
        }

        let content = fs::read_to_string(&path).context("Failed to read existing configuration")?;

        let value: Value = toml::from_str(&content).context("Failed to parse configuration")?;

        // 解析配置
        let mut config = ProjectConfig::default();

        // 解析 [template.commit]
        if let Some(template_section) = value.get("template") {
            if let Some(commit_section) = template_section.get("commit") {
                if let Some(table) = commit_section.as_table() {
                    for (key, value) in table {
                        config.template_commit.insert(key.clone(), value.clone());
                    }
                }
            }
            // 解析 [template.branch]
            if let Some(branch_section) = template_section.get("branch") {
                if let Some(table) = branch_section.as_table() {
                    for (key, value) in table {
                        config.template_branch.insert(key.clone(), value.clone());
                    }
                }
            }
            // 解析 [template.pull_requests]
            if let Some(pull_requests_section) = template_section.get("pull_requests") {
                if let Some(table) = pull_requests_section.as_table() {
                    for (key, value) in table {
                        config.template_pull_requests.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // 解析 [branch]
        if let Some(branch_section) = value.get("branch") {
            // Try to deserialize as ProjectBranchConfig
            if let Ok(branch_config) = toml::from_str::<ProjectBranchConfig>(
                &toml::to_string(branch_section).context("Failed to serialize branch section")?,
            ) {
                config.branch = branch_config;
            } else {
                // Fallback: manual parsing for backward compatibility
                if let Some(table) = branch_section.as_table() {
                    for (key, value) in table {
                        match key.as_str() {
                            "prefix" => {
                                if let Some(s) = value.as_str() {
                                    config.branch.prefix = Some(s.to_string());
                                }
                            }
                            "ignore" => {
                                if let Some(arr) = value.as_array() {
                                    config.branch.ignore = arr
                                        .iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect();
                                }
                            }
                            _ => {
                                // Ignore unknown keys
                            }
                        }
                    }
                }
            }
        }

        // 解析 [pr] 或顶层的 auto_accept_change_type
        if let Some(pr_section) = value.get("pr") {
            if let Some(table) = pr_section.as_table() {
                if let Some(auto_accept) = table.get("auto_accept_change_type") {
                    if let Some(b) = auto_accept.as_bool() {
                        config.auto_accept_change_type = Some(b);
                    }
                }
            }
        } else if let Some(auto_accept) = value.get("auto_accept_change_type") {
            // 向后兼容：支持顶层配置
            if let Some(b) = auto_accept.as_bool() {
                config.auto_accept_change_type = Some(b);
            }
        }

        Ok(config)
    }

    /// Save project configuration to file
    ///
    /// Merges the provided configuration with existing configuration and writes to file.
    /// Saves to `.workflow/config.toml` in the project root directory.
    /// Creates the directory structure if it doesn't exist.
    ///
    /// # Parameters
    ///
    /// * `config` - Configuration to save
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation, file reading, parsing, or writing fails.
    pub fn save(config: &ProjectConfig) -> Result<()> {
        let path = Paths::project_config().context("Failed to get project config path")?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create .workflow directory")?;
        }

        // 读取现有配置（如果存在）
        let mut existing_value: Value = if path.exists() {
            let content =
                fs::read_to_string(&path).context("Failed to read existing configuration")?;
            toml::from_str(&content).context("Failed to parse existing configuration")?
        } else {
            Value::Table(Map::new())
        };

        // 合并配置
        if let Some(table) = existing_value.as_table_mut() {
            // 更新 [template.commit]
            let template_table =
                table.entry("template".to_string()).or_insert_with(|| Value::Table(Map::new()));

            if let Some(template_map) = template_table.as_table_mut() {
                // 更新 [template.commit]
                let commit_table = template_map
                    .entry("commit".to_string())
                    .or_insert_with(|| Value::Table(Map::new()));

                if let Some(commit_map) = commit_table.as_table_mut() {
                    for (key, value) in &config.template_commit {
                        commit_map.insert(key.clone(), value.clone());
                    }
                }

                // 更新 [template.branch]
                if !config.template_branch.is_empty() {
                    let branch_table = template_map
                        .entry("branch".to_string())
                        .or_insert_with(|| Value::Table(Map::new()));

                    if let Some(branch_map) = branch_table.as_table_mut() {
                        for (key, value) in &config.template_branch {
                            branch_map.insert(key.clone(), value.clone());
                        }
                    }
                }

                // 更新 [template.pull_requests]
                if !config.template_pull_requests.is_empty() {
                    let pull_requests_table = template_map
                        .entry("pull_requests".to_string())
                        .or_insert_with(|| Value::Table(Map::new()));

                    if let Some(pull_requests_map) = pull_requests_table.as_table_mut() {
                        for (key, value) in &config.template_pull_requests {
                            pull_requests_map.insert(key.clone(), value.clone());
                        }
                    }
                }
            }

            // 更新 [branch]
            if config.branch.prefix.is_some() || !config.branch.ignore.is_empty() {
                let branch_table =
                    table.entry("branch".to_string()).or_insert_with(|| Value::Table(Map::new()));

                if let Some(branch_map) = branch_table.as_table_mut() {
                    // Serialize ProjectBranchConfig to TOML
                    let branch_toml = toml::to_string(&config.branch)
                        .context("Failed to serialize branch config")?;
                    let branch_value: Value =
                        toml::from_str(&branch_toml).context("Failed to parse branch config")?;

                    if let Some(branch_value_table) = branch_value.as_table() {
                        for (key, value) in branch_value_table {
                            branch_map.insert(key.clone(), value.clone());
                        }
                    }
                }
            }

            // 更新 [pr]
            if config.auto_accept_change_type.is_some() {
                let pr_table =
                    table.entry("pr".to_string()).or_insert_with(|| Value::Table(Map::new()));

                if let Some(pr_map) = pr_table.as_table_mut() {
                    if let Some(auto_accept) = config.auto_accept_change_type {
                        pr_map.insert(
                            "auto_accept_change_type".to_string(),
                            Value::Boolean(auto_accept),
                        );
                    }
                }
            }
        }

        // 写入文件
        let content =
            toml::to_string_pretty(&existing_value).context("Failed to serialize configuration")?;

        fs::write(&path, content).context("Failed to write configuration file")?;

        Ok(())
    }

    /// Get branch prefix for current repository
    ///
    /// Loads the configuration from project-level config (`.workflow/config.toml`).
    ///
    /// # Returns
    ///
    /// If current repository has branch prefix configured, returns the prefix string;
    /// otherwise returns `None`.
    pub fn get_branch_prefix() -> Option<String> {
        if let Ok(config) = Self::load() {
            return config.branch.prefix;
        }
        None
    }

    /// Get ignore branch list for current repository
    ///
    /// Loads from project-level config (`.workflow/config.toml`).
    ///
    /// # Returns
    ///
    /// Returns a vector of branch names that should be ignored for the current repository.
    pub fn get_ignore_branches() -> Vec<String> {
        if let Ok(config) = Self::load() {
            return config.branch.ignore;
        }
        Vec::new()
    }
}

/// Project-level branch configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectBranchConfig {
    /// Branch prefix (optional)
    ///
    /// Used to add prefix when generating branch names, e.g., "feature", "fix", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// List of branches to ignore (optional)
    ///
    /// These branches will not be deleted during branch cleanup.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
}

/// Project configuration structure
///
/// Represents project-level configuration with template and branch settings.
#[derive(Debug, Default)]
pub struct ProjectConfig {
    /// Template commit configuration
    pub template_commit: Map<String, Value>,
    /// Template branch configuration
    pub template_branch: Map<String, Value>,
    /// Template pull requests configuration
    pub template_pull_requests: Map<String, Value>,
    /// Branch configuration (project-level)
    pub branch: ProjectBranchConfig,
    /// Auto-accept change type selection (optional)
    ///
    /// If set to `true`, automatically accept the auto-selected change type
    /// in PR creation without prompting for confirmation.
    pub auto_accept_change_type: Option<bool>,
}
