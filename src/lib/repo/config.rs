//! Repository configuration management
//!
//! Provides reusable functions for checking repository configuration.
//! This is a library function that can be called by commands.

use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use anyhow::{Context, Result};
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
        Ok(Self::has_required_sections(&config_path)?)
    }

    /// Check if configuration has required sections
    fn has_required_sections(path: &PathBuf) -> Result<bool> {
        use std::fs;
        use toml::Value;

        let content = fs::read_to_string(path)
            .context("Failed to read configuration file")?;

        let value: Value = toml::from_str(&content)
            .context("Failed to parse configuration file")?;

        // 检查是否有 [template.commit] 或 [branch] 节
        let has_template = value
            .get("template")
            .and_then(|t| t.get("commit"))
            .is_some();

        let has_branch = value.get("branch").is_some();

        // 至少需要有一个配置节
        Ok(has_template || has_branch)
    }

    /// Load project configuration from file
    ///
    /// Reads and parses the project-level configuration file.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Returns a `ProjectConfig` with parsed configuration, or default if file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails.
    pub fn load(path: &PathBuf) -> Result<ProjectConfig> {
        let content = fs::read_to_string(path)
            .context("Failed to read existing configuration")?;

        let value: Value = toml::from_str(&content)
            .context("Failed to parse configuration")?;

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
        }

        // 解析 [branch]
        if let Some(branch_section) = value.get("branch") {
            if let Some(table) = branch_section.as_table() {
                for (key, value) in table {
                    config.branch.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(config)
    }

    /// Save project configuration to file
    ///
    /// Merges the provided configuration with existing configuration and writes to file.
    /// Creates the directory structure if it doesn't exist.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to the configuration file
    /// * `config` - Configuration to save
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation, file reading, parsing, or writing fails.
    pub fn save(path: &PathBuf, config: &ProjectConfig) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create .workflow directory")?;
        }

        // 读取现有配置（如果存在）
        let mut existing_value: Value = if path.exists() {
            let content = fs::read_to_string(path)
                .context("Failed to read existing configuration")?;
            toml::from_str(&content)
                .context("Failed to parse existing configuration")?
        } else {
            Value::Table(Map::new())
        };

        // 合并配置
        if let Some(table) = existing_value.as_table_mut() {
            // 更新 [template.commit]
            let template_table = table
                .entry("template".to_string())
                .or_insert_with(|| Value::Table(Map::new()));

            if let Some(template_map) = template_table.as_table_mut() {
                let commit_table = template_map
                    .entry("commit".to_string())
                    .or_insert_with(|| Value::Table(Map::new()));

                if let Some(commit_map) = commit_table.as_table_mut() {
                    for (key, value) in &config.template_commit {
                        commit_map.insert(key.clone(), value.clone());
                    }
                }
            }

            // 更新 [branch]
            if !config.branch.is_empty() {
                let branch_table = table
                    .entry("branch".to_string())
                    .or_insert_with(|| Value::Table(Map::new()));

                if let Some(branch_map) = branch_table.as_table_mut() {
                    for (key, value) in &config.branch {
                        branch_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // 写入文件
        let content = toml::to_string_pretty(&existing_value)
            .context("Failed to serialize configuration")?;

        fs::write(path, content)
            .context("Failed to write configuration file")?;

        Ok(())
    }
}

/// Project configuration structure
///
/// Represents project-level configuration with template and branch settings.
#[derive(Debug, Default)]
pub struct ProjectConfig {
    /// Template commit configuration
    pub template_commit: Map<String, Value>,
    /// Branch configuration
    pub branch: Map<String, Value>,
}

