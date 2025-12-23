//! Repository setup command
//!
//! Interactively initialize repository-level configuration.
//! Similar to CheckCommand, provides a static method for other commands to call.

use crate::base::dialog::{ConfirmDialog, FormBuilder, GroupConfig, InputDialog};
use crate::base::mcp::config::{MCPConfig, MCPConfigManager, MCPServerConfig};
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::{GitHubAccount, Settings};
use crate::base::util::file::FileWriter;
use crate::git::GitRepo;
use crate::repo::config::{BranchConfig, PullRequestsConfig, RepoConfig};
use crate::{log_break, log_debug, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use std::collections::HashMap;
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
    /// Checks if `repo setup` has been completed by calling `RepoConfig::exists()`,
    /// which checks `PrivateRepoConfig.configured` field.
    ///
    /// If configuration doesn't exist, it will:
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
    /// - If configuration exists (checked via `PrivateRepoConfig.configured`), returns immediately
    /// - Uses unified check: `RepoConfig::exists()` which checks `PrivateRepoConfig.configured`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use workflow::commands::repo::setup::RepoSetupCommand;
    /// use color_eyre::Result;
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
                .wrap_err("Failed to get user confirmation")?;

        if should_setup {
            // 5. Run setup
            log_break!();
            log_info!("Running repository setup...");
            log_break!();

            Self::run().wrap_err("Failed to run repository setup")?;

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
    /// use color_eyre::Result;
    ///
    /// // Called by other commands
    /// fn example() -> Result<()> {
    ///     RepoSetupCommand::run()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn run() -> Result<()> {
        // 1. 检查是否在 Git 仓库中
        let repo_name = GitRepo::extract_repo_name()
            .wrap_err("Not in a Git repository. Please run this command in a Git repository.")?;

        log_message!("Repository: {}", repo_name);
        log_break!();

        // 2. 加载现有配置（如果存在）
        let existing_config = RepoConfig::load().ok();

        // 3. 收集配置信息
        let config = Self::collect_config(&existing_config)?;

        // 4. 保存配置
        config.save().wrap_err("Failed to save config")?;

        log_break!();
        log_success!("Repository configuration saved successfully!");
        log_debug!(
            "Project template configuration: {}",
            Paths::project_config()?.display()
        );
        log_debug!(
            "Personal preference configuration: {}",
            Paths::repository_config()?.display()
        );
        log_success!(
            "You can commit the project template configuration to Git to share with your team."
        );

        Ok(())
    }

    /// Collect configuration interactively
    ///
    /// Collects configuration from user input
    ///
    /// Returns unified configuration containing both public (project template) and private (personal preference) settings.
    fn collect_config(existing: &Option<RepoConfig>) -> Result<RepoConfig> {
        let mut config = existing.clone().unwrap_or_default();

        // 准备现有值
        let current_prefix = config.branch.as_ref().and_then(|b| b.prefix.clone());
        let current_use_scope = config
            .template_commit
            .get("use_scope")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let current_auto_accept =
            config.pr.as_ref().and_then(|p| p.auto_accept_change_type).unwrap_or(false);

        // 使用 FormBuilder 收集所有配置
        let form_result = FormBuilder::new()
            // Group 1: Personal Preference Configuration (必填组)
            .add_group(
                "personal_preference",
                |g| {
                    g.step(|f| {
                        // Branch prefix
                        let prefix_prompt = if current_prefix.is_some() {
                            "Enter branch prefix (press Enter to keep)"
                        } else {
                            "Enter branch prefix (optional, press Enter to skip, e.g., 'feature', 'fix'):"
                        };
                        let mut field = f.add_text("branch_prefix", prefix_prompt);
                        if current_prefix.is_some() {
                            field = field.allow_empty(true);
                            if let Some(ref prefix) = current_prefix {
                                field = field.default(prefix.clone());
                            }
                        } else {
                            field = field.allow_empty(true);
                        }
                        field
                    })
                    .step(|f| {
                        // Auto-accept change type
                        f.add_confirmation(
                            "auto_accept_change_type",
                            "Auto-accept auto-selected change type in PR creation? (skip confirmation prompt)",
                        )
                        .default(current_auto_accept)
                    })
                },
                GroupConfig::required()
                    .with_title("Personal Preference Configuration")
                    .with_description("These settings are personal preferences and will be saved to your global config."),
            )
            // Group 2: Project Template Configuration (必填组)
            .add_group(
                "project_template",
                |g| {
                    g.step(|f| {
                        // Use scope
                        f.add_confirmation("use_scope", "Use scope for commit messages?")
                            .default(current_use_scope)
                    })
                    .step(|f| {
                        // Commit template configuration
                        f.add_confirmation("configure_commit_template", "Configure commit templates?")
                            .default(false)
                    })
                    .step_if("configure_commit_template", "yes", |f| {
                        f.add_text("custom_commit_template", "Enter custom commit template:")
                            .allow_empty(true)
                    })
                    .step(|f| {
                        // Branch template configuration
                        f.add_confirmation("configure_branch_template", "Configure branch templates?")
                            .default(false)
                    })
                    .step_if("configure_branch_template", "yes", |f| {
                        f.add_text(
                            "custom_branch_template",
                            "Enter custom default branch template:",
                        )
                        .allow_empty(true)
                    })
                    .step(|f| {
                        // PR template configuration
                        f.add_confirmation("configure_pr_template", "Configure pull request templates?")
                            .default(false)
                    })
                    .step_if("configure_pr_template", "yes", |f| {
                        f.add_text("custom_pr_template", "Enter custom pull request template:")
                            .allow_empty(true)
                    })
                },
                GroupConfig::required()
                    .with_title("Project Template Configuration")
                    .with_description("These settings are project standards and will be saved to .workflow/config.toml (can be committed to Git)."),
            )
            .run()
            .wrap_err("Failed to collect repository configuration")?;

        // 处理结果：Branch prefix
        if let Some(prefix_input) = form_result.get("branch_prefix") {
            let branch_prefix: Option<String> = if !prefix_input.trim().is_empty() {
                Some(prefix_input.trim().to_string())
            } else {
                current_prefix.clone()
            };

            if branch_prefix.is_some() {
                config.branch = Some(BranchConfig {
                    prefix: branch_prefix,
                    ignore: config.branch.as_ref().map(|b| b.ignore.clone()).unwrap_or_default(),
                });
            }
        }

        // 处理结果：Use scope
        if let Some(use_scope) = form_result.get_bool("use_scope") {
            config
                .template_commit
                .insert("use_scope".to_string(), Value::Boolean(use_scope));
        }

        // 处理结果：Commit template
        // 如果用户选择配置且输入了自定义模板，则保存；如果输入为空，则不写入（使用默认模板）
        if form_result.get("configure_commit_template") == Some(&"yes".to_string()) {
            if let Some(custom_template) = form_result.get("custom_commit_template") {
                if !custom_template.trim().is_empty() {
                    config.template_commit.insert(
                        "default".to_string(),
                        Value::String(custom_template.trim().to_string()),
                    );
                }
            }
        }

        // 处理结果：Branch template
        // 如果用户选择配置且输入了自定义模板，则保存；如果输入为空，则不写入（使用默认模板）
        if form_result.get("configure_branch_template") == Some(&"yes".to_string()) {
            if let Some(custom_branch_template) = form_result.get("custom_branch_template") {
                if !custom_branch_template.trim().is_empty() {
                    config.template_branch.insert(
                        "default".to_string(),
                        Value::String(custom_branch_template.trim().to_string()),
                    );
                }
            }
        }

        // 处理结果：PR template
        // 如果用户选择配置且输入了自定义模板，则保存；如果输入为空，则不写入（使用默认模板）
        if form_result.get("configure_pr_template") == Some(&"yes".to_string()) {
            if let Some(custom_pr_template) = form_result.get("custom_pr_template") {
                if !custom_pr_template.trim().is_empty() {
                    config.template_pull_requests.insert(
                        "default".to_string(),
                        Value::String(custom_pr_template.trim().to_string()),
                    );
                }
            }
        }

        // 处理结果：Auto-accept change type
        if let Some(auto_accept) = form_result.get_bool("auto_accept_change_type") {
            config.pr = Some(PullRequestsConfig {
                auto_accept_change_type: Some(auto_accept),
            });
        }

        // MCP Configuration (顺序交互式流程)
        log_break!();
        log_message!("MCP Configuration (Project-level)");
        log_break!('-', 40);
        log_debug!("Configure MCP servers for Cursor IDE integration.");
        log_break!();

        Self::setup_mcp_integration()?;

        // Mark as configured
        config.configured = true;

        Ok(config)
    }

    /// 设置 MCP 集成
    ///
    /// 直接在 RepoSetupCommand 中实现 MCP 配置功能
    fn setup_mcp_integration() -> Result<()> {
        // 1. 检测配置状态
        let mcp_manager = MCPConfigManager::new()?;
        let settings = Settings::load(); // 使用 load() 而不是 get() 来获取最新配置
        let mut new_config = MCPConfig::default();

        // 2. 询问是否配置 JIRA MCP
        log_debug!("Setting up Jira MCP servers...");
        let configure_jira = ConfirmDialog::new("Configure JIRA MCP server?")
            .with_default(true)
            .prompt()
            .wrap_err("Failed to get JIRA MCP configuration preference")?;

        if configure_jira {
            log_break!();
            let server_config = Self::configure_jira_mcp(&settings)?;
            new_config.mcp_servers.insert("jira".to_string(), server_config);
        }

        // 3. 询问是否配置 GitHub MCP
        log_break!();
        log_debug!("Setting up GitHub MCP servers...");
        let configure_github = ConfirmDialog::new("Configure GitHub MCP server?")
            .with_default(true)
            .prompt()
            .wrap_err("Failed to get GitHub MCP configuration preference")?;

        if configure_github {
            log_break!();
            let server_config = Self::configure_github_mcp(&settings)?;
            new_config.mcp_servers.insert("github".to_string(), server_config);
        }

        // 4. 保存配置（如果有配置的服务器）
        if !new_config.mcp_servers.is_empty() {
            log_break!();
            log_message!("Saving MCP configuration...");
            mcp_manager.merge(&new_config)?;
            log_success!(
                "MCP configuration saved to: {:?}",
                mcp_manager.config_path()
            );
        } else {
            log_warning!("No MCP servers configured, skipping save");
        }

        Ok(())
    }

    /// 配置 JIRA MCP
    fn configure_jira_mcp(settings: &Settings) -> Result<MCPServerConfig> {
        log_message!("Configuring JIRA MCP...");
        log_break!('-', 40);

        // 尝试从现有 MCP 配置中读取 JIRA 信息
        let mcp_manager = MCPConfigManager::new()?;
        let existing_mcp_config = mcp_manager.read()?;
        let existing_jira_config = existing_mcp_config.mcp_servers.get("jira");

        // 从 MCP 配置中提取现有值
        let (existing_jira_url, existing_jira_username, existing_jira_token) =
            if let Some(jira_config) = existing_jira_config {
                (
                    jira_config.env.get("JIRA_SERVER_URL").cloned(),
                    jira_config.env.get("JIRA_USERNAME").cloned(),
                    jira_config.env.get("JIRA_API_TOKEN").cloned(),
                )
            } else {
                (None, None, None)
            };

        // 优先级：MCP 配置 > Settings 配置
        let current_jira_address =
            existing_jira_url.or_else(|| settings.jira.service_address.clone());
        let current_jira_email = existing_jira_username.or_else(|| settings.jira.email.clone());
        let current_jira_token = existing_jira_token.or_else(|| settings.jira.api_token.clone());

        let has_jira_address = current_jira_address.is_some();
        let has_jira_email = current_jira_email.is_some();
        let has_jira_token = current_jira_token.is_some();

        let jira_address_prompt = if has_jira_address {
            "JIRA server URL (press Enter to keep)"
        } else {
            "JIRA server URL"
        };
        let jira_email_prompt = if has_jira_email {
            "JIRA email address (press Enter to keep)"
        } else {
            "JIRA email address"
        };
        let jira_token_prompt = if has_jira_token {
            "JIRA API token [current: ***] (press Enter to keep)"
        } else {
            "JIRA API token"
        };

        // 使用 FormBuilder 收集 JIRA 配置
        let jira_form_result = FormBuilder::new()
            .add_group(
                "jira_mcp",
                |g| {
                    g.step(|f| {
                        // JIRA server address
                        let mut field = f.add_text("jira_service_address", jira_address_prompt);
                        if has_jira_address {
                            field = field.allow_empty(true);
                            if let Some(ref addr) = current_jira_address {
                                field = field.default(addr.clone());
                            }
                        } else {
                            field = field.required();
                        }
                        field.validate(move |input: &str| {
                            if input.is_empty() && !has_jira_address {
                                Err("JIRA server URL is required".to_string())
                            } else if !input.is_empty()
                                && !input.starts_with("http://")
                                && !input.starts_with("https://")
                            {
                                Err(
                                    "Please enter a valid URL (must start with http:// or https://)"
                                        .to_string(),
                                )
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .step(|f| {
                        // JIRA email
                        let mut field = f.add_text("jira_email", jira_email_prompt);
                        if has_jira_email {
                            field = field.allow_empty(true);
                            if let Some(ref email) = current_jira_email {
                                field = field.default(email.clone());
                            }
                        } else {
                            field = field.required();
                        }
                        field.validate(move |input: &str| {
                            if input.is_empty() && !has_jira_email {
                                Err("JIRA email address is required".to_string())
                            } else if !input.is_empty() && !input.contains('@') {
                                Err("Please enter a valid email address".to_string())
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .step(|f| {
                        // JIRA API token (不设置默认值，显示 ***)
                        let mut field = f.add_text("jira_api_token", jira_token_prompt);
                        if has_jira_token {
                            field = field.allow_empty(true);
                            // 不设置默认值，这样显示 *** 而不是明文
                        } else {
                            field = field.required();
                        }
                        field.validate(move |input: &str| {
                            if input.is_empty() && !has_jira_token {
                                Err("JIRA API token is required".to_string())
                            } else {
                                Ok(())
                            }
                        })
                    })
                },
                GroupConfig::required().with_title("JIRA MCP Configuration"),
            )
            .run()
            .wrap_err("Failed to collect JIRA configuration")?;

        // 从 form 结果中提取值
        let service_address = if let Some(address) = jira_form_result.get("jira_service_address") {
            if !address.is_empty() {
                address.clone()
            } else if let Some(addr) = &current_jira_address {
                addr.clone()
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "JIRA server URL is required for MCP configuration"
                ));
            }
        } else {
            return Err(color_eyre::eyre::eyre!(
                "JIRA server URL is required for MCP configuration"
            ));
        };

        let username = if let Some(email) = jira_form_result.get("jira_email") {
            if !email.is_empty() {
                email.clone()
            } else if let Some(e) = &current_jira_email {
                e.clone()
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "JIRA email address is required for MCP configuration"
                ));
            }
        } else {
            return Err(color_eyre::eyre::eyre!(
                "JIRA email address is required for MCP configuration"
            ));
        };

        let api_token = if let Some(token) = jira_form_result.get("jira_api_token") {
            if !token.is_empty() {
                token.clone()
            } else if let Some(t) = &current_jira_token {
                t.clone()
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "JIRA API token is required for MCP configuration"
                ));
            }
        } else {
            return Err(color_eyre::eyre::eyre!(
                "JIRA API token is required for MCP configuration"
            ));
        };

        // 检查是否需要更新全局配置
        Self::sync_jira_config_to_settings(&service_address, &username, &api_token)?;

        Ok(MCPServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-jira".to_string(),
            ],
            env: {
                let mut env = HashMap::new();
                env.insert("JIRA_SERVER_URL".to_string(), service_address);
                env.insert("JIRA_USERNAME".to_string(), username);
                env.insert("JIRA_API_TOKEN".to_string(), api_token);
                env
            },
        })
    }

    /// 配置 GitHub MCP
    fn configure_github_mcp(settings: &Settings) -> Result<MCPServerConfig> {
        log_message!("Configuring GitHub MCP...");
        log_break!('-', 40);

        // 尝试从现有 MCP 配置中读取 GitHub 信息
        let mcp_manager = MCPConfigManager::new()?;
        let existing_mcp_config = mcp_manager.read()?;
        let existing_github_config = existing_mcp_config.mcp_servers.get("github");

        // 从 MCP 配置中提取现有 GitHub token
        let existing_github_token = existing_github_config
            .and_then(|config| config.env.get("GITHUB_PERSONAL_ACCESS_TOKEN"))
            .cloned();

        // GitHub API Token（必填）
        // 优先级：MCP 配置 > Settings 配置
        let current_github_token = existing_github_token.or_else(|| {
            settings.github.get_current_account().map(|account| account.api_token.clone())
        });
        let has_github_token = current_github_token.is_some();

        let github_token_prompt = if has_github_token {
            if let Some(account) = settings.github.get_current_account() {
                format!(
                    "GitHub API token from {} ({}) [current: ***] (press Enter to keep)",
                    account.name, account.email
                )
            } else {
                "GitHub API token [current: ***] (press Enter to keep)".to_string()
            }
        } else {
            "GitHub API token".to_string()
        };

        let github_api_token = InputDialog::new(&github_token_prompt)
            .allow_empty(has_github_token)
            .with_validator(move |input: &str| {
                if input.trim().is_empty() && !has_github_token {
                    Err("GitHub API token is required".to_string())
                } else if !input.trim().is_empty() && input.trim().len() < 10 {
                    Err("GitHub API token seems too short".to_string())
                } else {
                    Ok(())
                }
            })
            .prompt()
            .wrap_err("Failed to get GitHub API token")?;

        let api_token = if !github_api_token.is_empty() {
            github_api_token
        } else if let Some(token) = &current_github_token {
            token.clone()
        } else {
            return Err(color_eyre::eyre::eyre!(
                "GitHub API token is required for MCP configuration"
            ));
        };

        // 检查是否需要更新全局配置
        Self::sync_github_config_to_settings(&api_token)?;

        Ok(MCPServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            env: {
                let mut env = HashMap::new();
                env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), api_token);
                env
            },
        })
    }

    /// 同步 JIRA 配置到全局设置
    ///
    /// 如果全局配置中缺少 JIRA 配置，将 MCP 配置中的值保存到全局配置
    fn sync_jira_config_to_settings(
        service_address: &str,
        username: &str,
        api_token: &str,
    ) -> Result<()> {
        // 重新加载最新的全局配置，避免使用缓存
        let latest_settings = Settings::load();
        let mut needs_update = false;
        let mut updated_settings = latest_settings.clone();

        // 检查并更新 JIRA 服务地址（只在为空时更新）
        if updated_settings.jira.service_address.is_none() {
            updated_settings.jira.service_address = Some(service_address.to_string());
            needs_update = true;
            log_info!(
                "Syncing JIRA server URL to global config: {}",
                service_address
            );
        }

        // 检查并更新 JIRA 邮箱（只在为空时更新）
        if updated_settings.jira.email.is_none() {
            updated_settings.jira.email = Some(username.to_string());
            needs_update = true;
            log_info!("Syncing JIRA email to global config: {}", username);
        }

        // 检查并更新 JIRA API Token（只在为空时更新）
        if updated_settings.jira.api_token.is_none() {
            updated_settings.jira.api_token = Some(api_token.to_string());
            needs_update = true;
            log_info!("Syncing JIRA API token to global config");
        }

        // 如果有更新，保存配置
        if needs_update {
            let config_path = Paths::workflow_config()?;
            FileWriter::new(&config_path).write_toml(&updated_settings)?;
            log_success!("Global JIRA configuration updated");
        }

        Ok(())
    }

    /// 同步 GitHub 配置到全局设置
    ///
    /// 如果全局配置中没有 GitHub 账号，提示用户是否要添加
    fn sync_github_config_to_settings(api_token: &str) -> Result<()> {
        // 重新加载最新的全局配置，避免使用缓存
        let latest_settings = Settings::load();

        // 如果已经有 GitHub 账号配置，不需要同步
        if !latest_settings.github.accounts.is_empty() {
            return Ok(());
        }

        // 询问用户是否要将 GitHub token 保存到全局配置
        let github_sync_form_result = FormBuilder::new()
            .add_group(
                "github_sync",
                |g| {
                    g.step(|f| {
                        f.add_confirmation(
                            "should_sync",
                            "No GitHub account found in global config. Save this token to global config?",
                        )
                        .default(true)
                    })
                    .step_if("should_sync", "yes", |f| {
                        f.add_text("account_name", "GitHub account name (required)")
                            .required()
                            .validate(|input: &str| {
                                if input.trim().is_empty() {
                                    Err("Account name is required".to_string())
                                } else {
                                    Ok(())
                                }
                            })
                            .add_text("account_email", "GitHub account email (required)")
                            .required()
                            .validate(|input: &str| {
                                if input.trim().is_empty() {
                                    Err("Email is required".to_string())
                                } else if !input.contains('@') {
                                    Err("Please enter a valid email address".to_string())
                                } else {
                                    Ok(())
                                }
                            })
                    })
                },
                GroupConfig::required(),
            )
            .run()
            .wrap_err("Failed to collect GitHub account information")?;

        if github_sync_form_result.get("should_sync") == Some(&"yes".to_string()) {
            // 获取账号信息
            let account_name = github_sync_form_result
                .get("account_name")
                .ok_or_else(|| color_eyre::eyre::eyre!("Account name is required"))?
                .trim()
                .to_string();

            let account_email = github_sync_form_result
                .get("account_email")
                .ok_or_else(|| color_eyre::eyre::eyre!("Account email is required"))?
                .trim()
                .to_string();

            // 创建新的 GitHub 账号配置
            let mut updated_settings = latest_settings.clone();
            let github_account = GitHubAccount {
                name: account_name,
                email: account_email,
                api_token: api_token.to_string(),
            };

            updated_settings.github.accounts.push(github_account.clone());
            updated_settings.github.current = Some(github_account.name.clone()); // 设置为当前账号

            // 保存配置
            let config_path = Paths::workflow_config()?;
            FileWriter::new(&config_path).write_toml(&updated_settings)?;
            log_success!("GitHub account added to global configuration");
        }

        Ok(())
    }
}
