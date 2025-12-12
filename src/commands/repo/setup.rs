//! Repository setup command
//!
//! Interactively initialize repository-level configuration.
//! Similar to CheckCommand, provides a static method for other commands to call.

use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use crate::repo::{RepoConfig, ProjectConfig};
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use std::io::{self, IsTerminal};
use toml::Value;

/// Repository setup command
///
/// Similar to CheckCommand, provides a static method that can be called by other commands.
pub struct RepoSetupCommand;

impl RepoSetupCommand {
    /// Ensure repository configuration exists
    ///
    /// This function should be called at the beginning of branch/commit/pr operations.
    /// If project-level configuration doesn't exist, it will:
    /// 1. Check if in interactive environment
    /// 2. Prompt user to run setup
    /// 3. Run setup automatically if user confirms
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if configuration exists or user chooses to continue without setup.
    /// Returns error only if setup is required and fails.
    ///
    /// # Notes
    ///
    /// - Only prompts in interactive environment (checks if TTY)
    /// - Does not interrupt calling flow: Even if user cancels, returns Ok(())
    /// - If configuration exists, returns immediately
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::commands::repo::setup::RepoSetupCommand;
    ///
    /// pub fn execute(...) -> Result<()> {
    ///     RepoSetupCommand::ensure()?;
    ///     // ... 继续执行操作
    /// }
    /// ```
    pub fn ensure() -> Result<()> {
        // 1. Check if in interactive environment
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Ok(()); // Non-interactive environment, skip check
        }

        // 2. Check if configuration exists
        if RepoConfig::exists()? {
            return Ok(()); // Configuration exists, no need to setup
        }

        // 3. Configuration doesn't exist or is incomplete
        log_break!();
        log_warning!("Repository configuration not found or incomplete.");
        log_info!("Project-level configuration helps:");
        log_info!("  - Share branch prefix and commit template settings with your team");
        log_info!("  - Automatically configure commit message format");
        log_info!("  - Manage ignored branches");
        log_break!();

        // 4. Ask user if they want to run setup
        let should_setup = ConfirmDialog::new("Run 'workflow repo setup' to configure this repository?")
            .with_default(true)
            .prompt()
            .context("Failed to get user confirmation")?;

        if should_setup {
            // 5. Run setup
            log_break!();
            log_info!("Running repository setup...");
            log_break!();

            Self::run()
                .context("Failed to run repository setup")?;

            log_break!();
            log_success!("Repository configuration completed!");
            log_break!();
        } else {
            log_info!("Skipping repository setup. You can run 'workflow repo setup' later.");
        }

        Ok(())
    }

    /// Run repository setup
    ///
    /// This method can be called:
    /// 1. Directly by users: `workflow repo setup`
    /// 2. By other commands: `RepoSetupCommand::run()`
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::commands::repo::setup::RepoSetupCommand;
    ///
    /// // Called by other commands
    /// RepoSetupCommand::run()?;
    /// ```
    pub fn run() -> Result<()> {
        log_success!("Repository Configuration Setup\n");

        // 1. 检查是否在 Git 仓库中
        let repo_name = GitRepo::extract_repo_name()
            .context("Not in a Git repository. Please run this command in a Git repository.")?;

        log_info!("Repository: {}", repo_name);
        log_break!();

        // 2. 检查项目级配置文件是否存在
        let config_path = Paths::project_config()?;
        let existing_config = if config_path.exists() {
            Some(RepoConfig::load(&config_path)?)
        } else {
            None
        };

        // 3. 收集配置信息
        let config = Self::collect_config(&existing_config)?;

        // 4. 保存配置
        RepoConfig::save(&config_path, &config)?;

        log_break!();
        log_success!("Repository configuration saved successfully!");
        log_info!("Configuration file: {}", config_path.display());
        log_info!("You can commit this file to Git to share with your team.");

        Ok(())
    }

    /// Collect configuration interactively
    fn collect_config(existing: &Option<ProjectConfig>) -> Result<ProjectConfig> {
        let mut config = ProjectConfig::default();

        log_message!("Branch Configuration");
        log_break!('-', 40);

        // 1. Branch prefix
        let current_prefix = existing
            .as_ref()
            .and_then(|c| c.branch.get("prefix"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let prefix_prompt = if let Some(ref p) = current_prefix {
            format!("Enter branch prefix (press Enter to keep) ({})", p)
        } else {
            "Enter branch prefix (optional, press Enter to skip, e.g., 'feature', 'fix'):".to_string()
        };

        let prefix_input = {
            let mut dialog = InputDialog::new(&prefix_prompt).allow_empty(true);

            // 如果有现有值，设置默认值
            if let Some(ref current) = current_prefix {
                dialog = dialog.with_default(current.clone());
            }

            dialog
                .prompt()
                .context("Failed to get branch prefix")?
        };

        // 处理输入：如果有现有值且用户输入为空，保持原值；否则使用新值或清空
        if !prefix_input.trim().is_empty() {
            config
                .branch
                .insert("prefix".to_string(), Value::String(prefix_input.trim().to_string()));
        } else if let Some(ref current) = current_prefix {
            // 用户按 Enter 保持原值
            config
                .branch
                .insert("prefix".to_string(), Value::String(current.clone()));
        }
        // 如果既没有输入也没有现有值，则不设置（跳过）

        log_break!();
        log_message!("Commit Template Configuration");
        log_break!('-', 40);

        // 2. Use scope
        let current_use_scope = existing
            .as_ref()
            .and_then(|c| c.template_commit.get("use_scope"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let use_scope = ConfirmDialog::new("Use scope for commit messages?")
            .with_default(current_use_scope)
            .prompt()
            .context("Failed to get use_scope setting")?;

        config.template_commit.insert(
            "use_scope".to_string(),
            Value::Boolean(use_scope),
        );

        // 3. Auto-extract scope
        let current_auto_extract = existing
            .as_ref()
            .and_then(|c| c.template_commit.get("auto_extract_scope"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let auto_extract = ConfirmDialog::new("Auto-extract scope from git diff?")
            .with_default(current_auto_extract)
            .prompt()
            .context("Failed to get auto_extract_scope setting")?;

        config.template_commit.insert(
            "auto_extract_scope".to_string(),
            Value::Boolean(auto_extract),
        );

        // 4. Use default template
        let use_default = ConfirmDialog::new("Use default commit template?")
            .with_default(true)
            .prompt()
            .context("Failed to get template preference")?;

        if use_default {
            // 使用默认模板
            let default_template = r#"{{#if jira_key}}{{jira_key}}: {{subject}}{{else}}{{#if use_scope}}{{commit_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira_key}}Closes {{jira_key}}{{/if}}"#;

            config.template_commit.insert(
                "default".to_string(),
                Value::String(default_template.to_string()),
            );
        }

        Ok(config)
    }
}
