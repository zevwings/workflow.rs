//! Repository setup command
//!
//! Interactively initialize repository-level configuration.
//! Similar to CheckCommand, provides a static method for other commands to call.

use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use crate::repo::{ProjectConfig, RepoConfig};
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
    /// ```rust,no_run
    /// use workflow::commands::repo::setup::RepoSetupCommand;
    /// use anyhow::Result;
    ///
    /// pub fn execute() -> Result<()> {
    ///     RepoSetupCommand::ensure()?;
    ///     // ... 继续执行操作
    ///     Ok(())
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
        let should_setup =
            ConfirmDialog::new("Run 'workflow repo setup' to configure this repository?")
                .with_default(true)
                .prompt()
                .context("Failed to get user confirmation")?;

        if should_setup {
            // 5. Run setup
            log_break!();
            log_info!("Running repository setup...");
            log_break!();

            Self::run().context("Failed to run repository setup")?;

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
    /// ```rust,no_run
    /// use workflow::commands::repo::setup::RepoSetupCommand;
    /// use anyhow::Result;
    ///
    /// // Called by other commands
    /// fn example() -> Result<()> {
    ///     RepoSetupCommand::run()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn run() -> Result<()> {
        log_success!("Repository Configuration Setup\n");

        // 1. 检查是否在 Git 仓库中
        let repo_name = GitRepo::extract_repo_name()
            .context("Not in a Git repository. Please run this command in a Git repository.")?;

        log_info!("Repository: {}", repo_name);
        log_break!();

        // 2. 加载现有配置（如果存在）
        let config_path = Paths::project_config()?;
        let existing_config = if config_path.exists() {
            Some(RepoConfig::load()?)
        } else {
            None
        };

        // 3. 收集配置信息
        let config = Self::collect_config(&existing_config)?;

        // 4. 保存配置
        RepoConfig::save(&config)?;

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
        let current_prefix = existing.as_ref().and_then(|c| c.branch.prefix.clone());

        let prefix_prompt = if let Some(ref p) = current_prefix {
            format!("Enter branch prefix (press Enter to keep) ({})", p)
        } else {
            "Enter branch prefix (optional, press Enter to skip, e.g., 'feature', 'fix'):"
                .to_string()
        };

        let prefix_input = {
            let mut dialog = InputDialog::new(&prefix_prompt).allow_empty(true);

            // 如果有现有值，设置默认值
            if let Some(ref current) = current_prefix {
                dialog = dialog.with_default(current.clone());
            }

            dialog.prompt().context("Failed to get branch prefix")?
        };

        // 处理输入：如果有现有值且用户输入为空，保持原值；否则使用新值或清空
        if !prefix_input.trim().is_empty() {
            config.branch.prefix = Some(prefix_input.trim().to_string());
        } else if let Some(ref current) = current_prefix {
            // 用户按 Enter 保持原值
            config.branch.prefix = Some(current.clone());
        }
        // 如果既没有输入也没有现有值，则不设置（跳过）

        log_break!();
        log_message!("Template Configuration");
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

        config
            .template_commit
            .insert("use_scope".to_string(), Value::Boolean(use_scope));

        // 3. Commit template configuration
        let configure_commit_template = ConfirmDialog::new("Configure commit templates?")
            .with_default(false)
            .prompt()
            .context("Failed to get commit template preference")?;

        if configure_commit_template {
            // 询问是否使用默认模板
            let use_default = ConfirmDialog::new("Use default commit template?")
                .with_default(true)
                .prompt()
                .context("Failed to get template preference")?;

            if !use_default {
                // 用户选择自定义模板，询问自定义模板内容
                // 如果使用默认模板，不写入 default 字段，让 serde default 机制处理
                let custom_template = InputDialog::new("Enter custom commit template:")
                    .allow_empty(false)
                    .prompt()
                    .context("Failed to get custom template")?;

                config.template_commit.insert(
                    "default".to_string(),
                    Value::String(custom_template.trim().to_string()),
                );
            }
            // 如果使用默认模板，不写入 default 字段
        }

        // 4. Branch template configuration
        let configure_branch_template = ConfirmDialog::new("Configure branch templates?")
            .with_default(false)
            .prompt()
            .context("Failed to get branch template preference")?;

        if configure_branch_template {
            // 询问是否使用默认模板
            let use_default_branch = ConfirmDialog::new("Use default branch template?")
                .with_default(true)
                .prompt()
                .context("Failed to get branch template preference")?;

            if !use_default_branch {
                let custom_branch_template =
                    InputDialog::new("Enter custom default branch template:")
                        .allow_empty(false)
                        .prompt()
                        .context("Failed to get custom branch template")?;

                config.template_branch.insert(
                    "default".to_string(),
                    Value::String(custom_branch_template.trim().to_string()),
                );
            }
            // 如果使用默认模板，不写入 default 字段
        }

        // 5. Pull request template configuration
        let configure_pr_template = ConfirmDialog::new("Configure pull request templates?")
            .with_default(false)
            .prompt()
            .context("Failed to get PR template preference")?;

        if configure_pr_template {
            // 询问是否使用默认模板
            let use_default_pr = ConfirmDialog::new("Use default PR template?")
                .with_default(true)
                .prompt()
                .context("Failed to get PR template preference")?;

            if !use_default_pr {
                let custom_pr_template = InputDialog::new("Enter custom PR template:")
                    .allow_empty(false)
                    .prompt()
                    .context("Failed to get custom PR template")?;

                config.template_pull_requests.insert(
                    "default".to_string(),
                    Value::String(custom_pr_template.trim().to_string()),
                );
            }
            // 如果使用默认模板，不写入 default 字段
        }

        log_break!();
        log_message!("PR Configuration");
        log_break!('-', 40);

        // 6. Auto-accept change type
        let current_auto_accept =
            existing.as_ref().and_then(|c| c.auto_accept_change_type).unwrap_or(false);

        let auto_accept = ConfirmDialog::new(
            "Auto-accept auto-selected change type in PR creation? (skip confirmation prompt)",
        )
        .with_default(current_auto_accept)
        .prompt()
        .context("Failed to get auto_accept_change_type setting")?;

        config.auto_accept_change_type = Some(auto_accept);

        Ok(config)
    }
}
