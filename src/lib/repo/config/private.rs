//! Private repository configuration management
//!
//! Manages personal preference configuration that should not be committed to Git.
//! This configuration is stored in `~/.workflow/config/repository.toml` and supports iCloud sync.

use crate::base::fs::{FileReader, FileWriter, PathAccess};
use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::path::Path;
use toml::map::Map;
use toml::Value;

use super::types::{BranchConfig, PullRequestsConfig};

/// Personal configuration manager (global configuration, not committed to Git)
///
/// Manages `~/.workflow/config/repository.toml` file, containing personal preference configuration.
/// Supports iCloud sync (on macOS).
///
/// This struct contains the configuration data and provides methods to load and save it.
///
/// **Note**: This struct should not be used directly from outside the `repo::config` module.
/// Use `RepoConfig` methods instead.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrivateRepoConfig {
    /// Whether the repository has been configured (marks if repo setup has been completed)
    ///
    /// **This is the single source of truth for checking if `repo setup` has been completed.**
    ///
    /// When the user runs `repo setup` and completes the configuration,
    /// this field should be set to `true`.
    ///
    /// All code should check this field (via `RepoConfig::exists()`) to determine
    /// if the repository has been configured, rather than checking file existence
    /// or other methods.
    ///
    /// # Usage
    ///
    /// ```rust,no_run
    /// use workflow::repo::RepoConfig;
    /// use color_eyre::Result;
    ///
    /// fn check_repo_config() -> Result<()> {
    ///     if RepoConfig::exists()? {
    ///         // Repository has been configured
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[serde(default)]
    pub configured: bool,
    /// Branch configuration (personal preference)
    pub branch: Option<BranchConfig>,
    /// PR configuration (personal preference)
    pub pr: Option<PullRequestsConfig>,
}

impl PrivateRepoConfig {
    /// Generate repository identifier
    ///
    /// Generates a unique repository identifier based on Git remote URL.
    /// Format: `{repo_name}_{hash}`, where hash is the first 8 characters of the URL's SHA256.
    ///
    /// # Returns
    ///
    /// Returns a repository identifier string, e.g., `workflow.rs_12345678`
    ///
    /// # Errors
    ///
    /// Returns an error if unable to get remote URL or extract repository name.
    pub fn generate_repo_id() -> Result<String> {
        Self::generate_repo_id_in(
            std::env::current_dir().wrap_err("Failed to get current directory")?,
        )
    }

    /// Generate repository identifier (specified repository path)
    ///
    /// Generates a unique repository identifier based on Git remote URL for the specified repository.
    /// Format: `{repo_name}_{hash}`, where hash is the first 8 characters of the URL's SHA256.
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # Returns
    ///
    /// Returns a repository identifier string, e.g., `workflow.rs_12345678`
    ///
    /// # Errors
    ///
    /// Returns an error if unable to get remote URL or extract repository name.
    pub fn generate_repo_id_in(repo_path: impl AsRef<Path>) -> Result<String> {
        let url =
            GitRepo::get_remote_url_in(repo_path.as_ref()).wrap_err("Failed to get remote URL")?;

        let repo_name_full = GitRepo::extract_repo_name_in(repo_path.as_ref())
            .wrap_err("Failed to extract repository name")?;

        let repo_name = repo_name_full
            .split('/')
            .next_back()
            .ok_or_else(|| eyre!("Failed to extract repo name from: {}", repo_name_full))?;

        // Calculate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);

        // Take first 8 characters (4 bytes, 32 bits), providing sufficient uniqueness while maintaining readability
        Ok(format!("{}_{}", repo_name, &hash_str[..8]))
    }

    /// Load personal preference configuration for current repository
    ///
    /// Loads personal preference configuration for the current repository
    /// from `~/.workflow/config/repository.toml`.
    pub fn load() -> Result<Self> {
        let repo_path = std::env::current_dir().wrap_err("Failed to get current directory")?;
        let home = Paths::home_dir().wrap_err("Failed to get home directory")?;
        Self::load_from(repo_path, home)
    }

    /// Load personal preference configuration (specified repository path and home directory)
    ///
    /// Loads personal preference configuration for the specified repository
    /// using the specified home directory, avoiding dependency on global environment variables.
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    /// * `home` - 用户主目录路径
    pub fn load_from(repo_path: impl AsRef<Path>, home: impl AsRef<Path>) -> Result<Self> {
        let repo_id = Self::generate_repo_id_in(repo_path.as_ref())
            .wrap_err("Failed to generate repository ID")?;
        // 从环境变量读取 disable_icloud 设置（测试环境会设置 WORKFLOW_DISABLE_ICLOUD=1）
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        let config_path = Paths::repository_config_in(home, disable_icloud)
            .wrap_err("Failed to get repository config path")?;

        // If file doesn't exist, return default configuration
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let value: Value = FileReader::new(&config_path).toml()?;

        // Find current repository's configuration sections
        // Format: [${repo_id}], [${repo_id}.branch] and [${repo_id}.pr]
        let branch_key = format!("{}.branch", repo_id);
        let pr_key = format!("{}.pr", repo_id);
        let repo_id_str = repo_id.clone();

        let mut config = Self::default();

        // Parse [${repo_id}] (top-level configuration)
        if let Some(repo_section) = value.get(&repo_id_str) {
            if let Some(table) = repo_section.as_table() {
                if let Some(configured) = table.get("configured") {
                    if let Some(configured_bool) = configured.as_bool() {
                        config.configured = configured_bool;
                    }
                }
            }
        }

        // Parse [${repo_id}.branch]
        if let Some(branch_section) = value.get(&branch_key) {
            if let Some(table) = branch_section.as_table() {
                let mut branch_config = BranchConfig::default();
                if let Some(prefix) = table.get("prefix") {
                    if let Some(prefix_str) = prefix.as_str() {
                        branch_config.prefix = Some(prefix_str.to_string());
                    }
                }
                if let Some(ignore) = table.get("ignore") {
                    if let Some(arr) = ignore.as_array() {
                        branch_config.ignore =
                            arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    }
                }
                config.branch = Some(branch_config);
            }
        }

        // Parse [${repo_id}.pr]
        if let Some(pr_section) = value.get(&pr_key) {
            if let Some(table) = pr_section.as_table() {
                let mut pr_config = PullRequestsConfig::default();
                if let Some(auto_accept) = table.get("auto_accept_change_type") {
                    if let Some(auto_accept_bool) = auto_accept.as_bool() {
                        pr_config.auto_accept_change_type = Some(auto_accept_bool);
                        tracing::debug!(
                            "Loaded auto_accept_change_type = {} for repo_id: {}",
                            auto_accept_bool,
                            repo_id
                        );
                    } else {
                        tracing::warn!(
                            "auto_accept_change_type is not a boolean for repo_id: {}",
                            repo_id
                        );
                    }
                } else {
                    tracing::debug!("auto_accept_change_type not found in [{}] section", pr_key);
                }
                config.pr = Some(pr_config);
            } else {
                tracing::warn!("[{}] section is not a table", pr_key);
            }
        } else {
            tracing::debug!("[{}] section not found in config", pr_key);
        }

        Ok(config)
    }

    /// Save personal preference configuration (specified repository path and home directory)
    ///
    /// Saves personal preference configuration for the specified repository
    /// using the specified home directory, avoiding dependency on global environment variables.
    /// Supports configuration merging, won't overwrite other repositories' configurations.
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    /// * `home` - 用户主目录路径
    pub fn save_in(&self, repo_path: impl AsRef<Path>, home: impl AsRef<Path>) -> Result<()> {
        let repo_id = Self::generate_repo_id_in(repo_path.as_ref())
            .wrap_err("Failed to generate repository ID")?;
        // 从环境变量读取 disable_icloud 设置（测试环境会设置 WORKFLOW_DISABLE_ICLOUD=1）
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        let config_path = Paths::repository_config_in(home, disable_icloud)
            .wrap_err("Failed to get repository config path")?;

        // Ensure config directory exists
        PathAccess::new(&config_path).ensure_parent_exists()?;

        // Open file and acquire exclusive lock to prevent concurrent writes
        // This ensures that multiple tests or processes don't overwrite each other's changes
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&config_path)
            .wrap_err_with(|| format!("Failed to open config file: {:?}", config_path))?;

        // Acquire exclusive lock (blocks until lock is available)
        // This prevents race conditions when multiple tests write to the same file concurrently
        file.lock_exclusive()
            .wrap_err_with(|| format!("Failed to lock config file: {:?}", config_path))?;

        // Read existing configuration (if exists) using the locked file handle
        // Note: On Windows, we must read from the locked file handle, not open a new one
        // We read after acquiring the lock to ensure we have the latest data
        let mut existing_value: Value = if config_path.exists() {
            use std::io::{Read, Seek};
            // Seek to beginning of file before reading
            file.seek(std::io::SeekFrom::Start(0))
                .wrap_err_with(|| format!("Failed to seek config file: {:?}", config_path))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .wrap_err_with(|| format!("Failed to read config file: {:?}", config_path))?;
            toml::from_str(&content)
                .wrap_err_with(|| format!("Failed to parse TOML config: {:?}", config_path))?
        } else {
            Value::Table(Map::new())
        };

        // Merge configuration
        if let Some(table) = existing_value.as_table_mut() {
            // Update [${repo_id}] (top-level configuration)
            let repo_table =
                table.entry(repo_id.to_string()).or_insert_with(|| Value::Table(Map::new()));

            if let Some(repo_map) = repo_table.as_table_mut() {
                repo_map.insert("configured".to_string(), Value::Boolean(self.configured));
            }

            // Update [${repo_id}.branch]
            if let Some(ref branch_config) = self.branch {
                if branch_config.prefix.is_some() || !branch_config.ignore.is_empty() {
                    let branch_key = format!("{}.branch", repo_id);
                    let branch_table =
                        table.entry(branch_key).or_insert_with(|| Value::Table(Map::new()));

                    if let Some(branch_map) = branch_table.as_table_mut() {
                        if let Some(ref prefix) = branch_config.prefix {
                            branch_map.insert("prefix".to_string(), Value::String(prefix.clone()));
                        }
                        if !branch_config.ignore.is_empty() {
                            let ignore_array: Vec<Value> = branch_config
                                .ignore
                                .iter()
                                .map(|s| Value::String(s.clone()))
                                .collect();
                            branch_map.insert("ignore".to_string(), Value::Array(ignore_array));
                        }
                    }
                }
            }

            // Update [${repo_id}.pr]
            if let Some(ref pr_config) = self.pr {
                if pr_config.auto_accept_change_type.is_some() {
                    let pr_key = format!("{}.pr", repo_id);
                    let pr_table = table.entry(pr_key).or_insert_with(|| Value::Table(Map::new()));

                    if let Some(pr_map) = pr_table.as_table_mut() {
                        if let Some(auto_accept) = pr_config.auto_accept_change_type {
                            pr_map.insert(
                                "auto_accept_change_type".to_string(),
                                Value::Boolean(auto_accept),
                            );
                        }
                    }
                }
            }
        }

        // Unlock before writing (FileWriter will open the file again)
        // On Windows, we must unlock before another process can write
        file.unlock()
            .wrap_err_with(|| format!("Failed to unlock config file: {:?}", config_path))?;

        // Write to file (FileWriter will handle its own locking if needed)
        FileWriter::new(&config_path).write_toml(&existing_value)?;

        Ok(())
    }
}
