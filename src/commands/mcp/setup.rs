//! MCP 配置设置命令
//!
//! 交互式配置 MCP 服务器，支持从 TOML 配置自动填充。

use crate::base::mcp::config::{MCPConfig, MCPConfigManager, MCPServerConfig};
use crate::base::settings::settings::Settings;
use crate::{log_break, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use std::collections::{HashMap, HashSet};

/// MCP 配置设置命令
pub struct SetupCommand;

impl SetupCommand {
    /// 运行交互式配置流程
    pub fn run() -> Result<()> {
        log_success!("MCP Configuration Wizard");
        log_break!('=', 40);
        log_break!();

        // 1. 检测配置状态（使用项目目录）
        let mcp_manager = MCPConfigManager::new()?;
        let configured_servers = mcp_manager.detect_configured_servers()?;
        let toml_status = Self::detect_toml_config_status();

        // 2. 显示配置状态并让用户选择
        let selected_servers = Self::select_servers(&configured_servers)?;

        if selected_servers.is_empty() {
            log_warning!("No MCP servers selected, configuration cancelled");
            return Ok(());
        }

        // 3. 配置每个选中的 MCP 服务器
        let mut new_config = MCPConfig::default();
        let mut config_sources = HashMap::new(); // 跟踪配置来源

        for server_name in &selected_servers {
            log_break!();
            match server_name.as_str() {
                "jira" => {
                    let (server_config, from_toml) = Self::configure_jira(&toml_status)?;
                    new_config
                        .mcp_servers
                        .insert("jira".to_string(), server_config);
                    config_sources.insert("jira".to_string(), from_toml);
                }
                "github" => {
                    let (server_config, from_toml) = Self::configure_github(&toml_status.github)?;
                    new_config
                        .mcp_servers
                        .insert("github".to_string(), server_config);
                    config_sources.insert("github".to_string(), from_toml);
                }
                "apifox" => {
                    let (server_config, from_toml) = Self::configure_apifox(&toml_status.apifox)?;
                    new_config
                        .mcp_servers
                        .insert("apifox".to_string(), server_config);
                    config_sources.insert("apifox".to_string(), from_toml);
                }
                _ => {
                    log_warning!("Unknown MCP server: {}", server_name);
                    continue;
                }
            }
        }

        // 4. 保存配置
        log_break!();
        log_message!("Saving configuration...");
        mcp_manager.merge(&new_config)?;
        log_success!("Configuration saved to: {:?}", mcp_manager.config_path());

        // 5. 统一提示是否保存到 TOML 配置文件
        Self::prompt_sync_to_toml(&selected_servers, &config_sources, &new_config)?;

        log_break!();
        log_success!("All configuration completed!");
        log_message!("Next steps:");
        log_message!("  1. Restart Cursor IDE");
        log_message!("  2. Use `workflow mcp verify` to verify configuration");

        Ok(())
    }

    /// 检测 TOML 配置状态
    fn detect_toml_config_status() -> TomlConfigStatus {
        let settings = Settings::get();
        TomlConfigStatus {
            jira: JiraConfigStatus {
                has_email: settings.jira.email.is_some(),
                has_api_token: settings.jira.api_token.is_some(),
                has_service_address: settings.jira.service_address.is_some(),
                is_complete: settings.jira.email.is_some()
                    && settings.jira.api_token.is_some()
                    && settings.jira.service_address.is_some(),
            },
            github: GitHubConfigStatus {
                has_accounts: !settings.github.accounts.is_empty(),
                has_current: settings.github.current.is_some(),
                current_account: settings.github.get_current_account().cloned(),
            },
            apifox: ApifoxConfigStatus {
                has_project_id: settings.apifox.project_id.is_some(),
                has_access_token: settings.apifox.access_token.is_some(),
                is_complete: settings.apifox.project_id.is_some()
                    && settings.apifox.access_token.is_some(),
            },
        }
    }

    /// 交互式选择要配置的 MCP 服务器
    fn select_servers(configured: &HashSet<String>) -> Result<Vec<String>> {
        let options = [
            ("JIRA MCP", "jira", configured.contains("jira")),
            ("GitHub MCP", "github", configured.contains("github")),
            ("Apifox MCP", "apifox", configured.contains("apifox")),
        ];

        let items: Vec<&str> = options.iter().map(|(name, _, _)| *name).collect();
        let defaults: Vec<bool> = options
            .iter()
            .map(|(_, _, is_configured)| *is_configured)
            .collect();

        let selections = MultiSelect::new()
            .with_prompt(
                "Select MCP servers to configure (multi-select, space to toggle, enter to confirm)",
            )
            .items(&items)
            .defaults(&defaults)
            .interact()
            .context("Failed to get MCP server selection")?;

        let selected: Vec<String> = selections
            .iter()
            .map(|&i| options[i].1.to_string())
            .collect();

        Ok(selected)
    }

    /// 配置 JIRA MCP
    ///
    /// 返回 (配置, 是否从 TOML 读取)
    fn configure_jira(toml_status: &TomlConfigStatus) -> Result<(MCPServerConfig, bool)> {
        log_message!("Configuring JIRA MCP...");
        log_break!('-', 40);

        let settings = Settings::get();

        // 如果 TOML 配置完整，询问是否使用
        if toml_status.jira.is_complete {
            let use_toml = Confirm::new()
                .with_prompt("Complete JIRA configuration found in config file, use it?")
                .default(true)
                .interact()
                .context("Failed to get confirmation")?;

            if use_toml {
                let jira = &settings.jira;
                return Ok((
                    MCPServerConfig {
                        command: "npx".to_string(),
                        args: vec![
                            "-y".to_string(),
                            "@modelcontextprotocol/server-jira".to_string(),
                        ],
                        env: {
                            let mut env = HashMap::new();
                            env.insert(
                                "JIRA_SERVER_URL".to_string(),
                                jira.service_address.clone().unwrap(),
                            );
                            env.insert("JIRA_USERNAME".to_string(), jira.email.clone().unwrap());
                            env.insert(
                                "JIRA_API_TOKEN".to_string(),
                                jira.api_token.clone().unwrap(),
                            );
                            env
                        },
                    },
                    true, // 从 TOML 读取
                ));
            }
        }

        // 交互式输入（部分或全部手动输入）
        let mut from_toml = true; // 跟踪是否所有值都来自 TOML

        let service_address = if let Some(addr) = &settings.jira.service_address {
            let use_toml = Confirm::new()
                .with_prompt(format!("Use server address from config file ({})?", addr))
                .default(true)
                .interact()?;

            if use_toml {
                addr.clone()
            } else {
                from_toml = false;
                Input::<String>::new()
                    .with_prompt("Enter JIRA server URL")
                    .interact_text()?
            }
        } else {
            from_toml = false;
            Input::<String>::new()
                .with_prompt("Enter JIRA server URL")
                .interact_text()?
        };

        let username = if let Some(email) = &settings.jira.email {
            let use_toml = Confirm::new()
                .with_prompt(format!("Use username from config file ({})?", email))
                .default(true)
                .interact()?;

            if use_toml {
                email.clone()
            } else {
                from_toml = false;
                Input::<String>::new()
                    .with_prompt("Enter JIRA username/email")
                    .interact_text()?
            }
        } else {
            from_toml = false;
            Input::<String>::new()
                .with_prompt("Enter JIRA username/email")
                .interact_text()?
        };

        // API Token 总是需要手动输入（即使 TOML 中有，也需要确认）
        // 如果 TOML 中有 token，可以提示用户是否使用
        let api_token = if let Some(toml_token) = &settings.jira.api_token {
            let use_toml = Confirm::new()
                .with_prompt("Use API token from config file?")
                .default(true)
                .interact()?;

            if use_toml {
                toml_token.clone()
            } else {
                from_toml = false;
                Password::new()
                    .with_prompt("Enter JIRA API token")
                    .interact()
                    .context("Failed to get JIRA API token")?
            }
        } else {
            from_toml = false;
            Password::new()
                .with_prompt("Enter JIRA API token")
                .interact()
                .context("Failed to get JIRA API token")?
        };

        Ok((
            MCPServerConfig {
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
            },
            from_toml,
        ))
    }

    /// 配置 GitHub MCP
    ///
    /// 返回 (配置, 是否从 TOML 读取)
    fn configure_github(toml_status: &GitHubConfigStatus) -> Result<(MCPServerConfig, bool)> {
        log_message!("Configuring GitHub MCP...");
        log_break!('-', 40);

        let settings = Settings::get();

        // 如果 TOML 配置中有账号，询问是否使用
        if toml_status.has_accounts {
            if let Some(account) = &toml_status.current_account {
                let use_toml = Confirm::new()
                    .with_prompt(format!(
                        "Use token from current account {} ({})?",
                        account.name, account.email
                    ))
                    .default(true)
                    .interact()?;

                if use_toml {
                    return Ok((
                        MCPServerConfig {
                            command: "npx".to_string(),
                            args: vec![
                                "-y".to_string(),
                                "@modelcontextprotocol/server-github".to_string(),
                            ],
                            env: {
                                let mut env = HashMap::new();
                                env.insert(
                                    "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                                    account.api_token.clone(),
                                );
                                env
                            },
                        },
                        true, // 从 TOML 读取
                    ));
                }
            }

            // 如果有多个账号，让用户选择
            if settings.github.accounts.len() > 1 {
                let account_names: Vec<&str> = settings
                    .github
                    .accounts
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect();

                let selection = Select::new()
                    .with_prompt("Select GitHub account to use")
                    .items(&account_names)
                    .interact()?;

                let account = &settings.github.accounts[selection];
                return Ok((
                    MCPServerConfig {
                        command: "npx".to_string(),
                        args: vec![
                            "-y".to_string(),
                            "@modelcontextprotocol/server-github".to_string(),
                        ],
                        env: {
                            let mut env = HashMap::new();
                            env.insert(
                                "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                                account.api_token.clone(),
                            );
                            env
                        },
                    },
                    true, // 从 TOML 读取
                ));
            }
        }

        // 交互式输入（手动输入，不是从 TOML 读取）
        let api_token = Password::new()
            .with_prompt("Enter GitHub Personal Access Token")
            .interact()
            .context("Failed to get GitHub token")?;

        Ok((
            MCPServerConfig {
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
            },
            false, // 手动输入，不是从 TOML 读取
        ))
    }

    /// 配置 Apifox MCP
    ///
    /// 返回 (配置, 是否从 TOML 读取)
    fn configure_apifox(toml_status: &ApifoxConfigStatus) -> Result<(MCPServerConfig, bool)> {
        log_message!("Configuring Apifox MCP...");
        log_break!('-', 40);

        let settings = Settings::get();

        // 如果 TOML 配置完整，询问是否使用
        if toml_status.is_complete {
            let use_toml = Confirm::new()
                .with_prompt("Complete Apifox configuration found in config file, use it?")
                .default(true)
                .interact()
                .context("Failed to get confirmation")?;

            if use_toml {
                let apifox = &settings.apifox;
                return Ok((
                    MCPServerConfig {
                        command: "npx".to_string(),
                        args: vec![
                            "-y".to_string(),
                            "apifox-mcp-server@latest".to_string(),
                            format!("--project={}", apifox.project_id.as_ref().unwrap()),
                        ],
                        env: {
                            let mut env = HashMap::new();
                            env.insert(
                                "APIFOX_ACCESS_TOKEN".to_string(),
                                apifox.access_token.clone().unwrap(),
                            );
                            env
                        },
                    },
                    true, // 从 TOML 读取
                ));
            }
        }

        // 交互式输入（部分或全部手动输入）
        let mut from_toml = true; // 跟踪是否所有值都来自 TOML

        let project_id = if let Some(id) = &settings.apifox.project_id {
            let use_toml = Confirm::new()
                .with_prompt(format!("Use project ID from config file ({})?", id))
                .default(true)
                .interact()?;

            if use_toml {
                id.clone()
            } else {
                from_toml = false;
                Input::<String>::new()
                    .with_prompt("Enter Apifox project ID")
                    .interact_text()
                    .context("Failed to get Apifox project ID")?
            }
        } else {
            from_toml = false;
            Input::<String>::new()
                .with_prompt("Enter Apifox project ID")
                .interact_text()
                .context("Failed to get Apifox project ID")?
        };

        let access_token = if let Some(token) = &settings.apifox.access_token {
            let use_toml = Confirm::new()
                .with_prompt("Use access token from config file?")
                .default(true)
                .interact()?;

            if use_toml {
                token.clone()
            } else {
                from_toml = false;
                Password::new()
                    .with_prompt("Enter Apifox access token")
                    .interact()
                    .context("Failed to get Apifox access token")?
            }
        } else {
            from_toml = false;
            Password::new()
                .with_prompt("Enter Apifox access token")
                .interact()
                .context("Failed to get Apifox access token")?
        };

        Ok((
            MCPServerConfig {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "apifox-mcp-server@latest".to_string(),
                    format!("--project={}", project_id),
                ],
                env: {
                    let mut env = HashMap::new();
                    env.insert("APIFOX_ACCESS_TOKEN".to_string(), access_token);
                    env
                },
            },
            from_toml,
        ))
    }

    /// 统一提示是否保存到 TOML 配置文件
    fn prompt_sync_to_toml(
        selected_servers: &[String],
        config_sources: &HashMap<String, bool>,
        mcp_config: &MCPConfig,
    ) -> Result<()> {
        // 收集需要保存的 MCP（未从 TOML 读取的）
        let mut syncable_options = Vec::new(); // 可以保存到 TOML 的
        let mut syncable_server_names = Vec::new();

        for server_name in selected_servers {
            let from_toml = config_sources
                .get(server_name.as_str())
                .copied()
                .unwrap_or(false);

            // 只显示未保存的 MCP（from_toml = false）
            if from_toml {
                continue; // 已保存，跳过
            }

            // 分类：可以保存到 TOML 的
            match server_name.as_str() {
                "jira" => {
                    syncable_options.push("JIRA");
                    syncable_server_names.push("jira");
                }
                "github" => {
                    syncable_options.push("GitHub");
                    syncable_server_names.push("github");
                }
                "apifox" => {
                    syncable_options.push("Apifox");
                    syncable_server_names.push("apifox");
                }
                _ => continue,
            }
        }

        // 如果没有需要保存的选项，跳过
        if syncable_options.is_empty() {
            return Ok(());
        }

        log_break!();
        log_message!("Save configuration to config file");
        log_break!('-', 40);

        // 显示可以保存的配置
        if !syncable_options.is_empty() {
            log_message!(
                "The following configurations are not saved to the config file, save them?"
            );
            log_break!();

            // 显示需要保存的列表
            for option in &syncable_options {
                log_message!("  - {}", option);
            }
            log_break!();

            // 显示多选框
            let selections = MultiSelect::new()
                .with_prompt(
                    "Select MCPs to save to config file (space to toggle, enter to confirm)",
                )
                .items(&syncable_options)
                .interact()
                .context("Failed to get sync selection")?;

            // 根据用户选择保存到 TOML
            for &index in &selections {
                let server_name = syncable_server_names[index];
                match server_name {
                    "jira" => {
                        if let Some(jira_config) = mcp_config.mcp_servers.get("jira") {
                            Self::save_jira_to_toml(jira_config)?;
                        }
                    }
                    "github" => {
                        if let Some(github_config) = mcp_config.mcp_servers.get("github") {
                            Self::save_github_to_toml(github_config)?;
                        }
                    }
                    "apifox" => {
                        if let Some(apifox_config) = mcp_config.mcp_servers.get("apifox") {
                            Self::save_apifox_to_toml(apifox_config)?;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// 保存 JIRA 配置到 TOML
    fn save_jira_to_toml(jira_config: &MCPServerConfig) -> Result<()> {
        use crate::base::settings::paths::Paths;
        use crate::base::settings::settings::{JiraSettings, Settings};
        use crate::jira::config::ConfigManager;

        // 从 MCP 配置中读取值
        let service_address = jira_config.env.get("JIRA_SERVER_URL").cloned();
        let email = jira_config.env.get("JIRA_USERNAME").cloned();
        let api_token = jira_config.env.get("JIRA_API_TOKEN").cloned();

        if service_address.is_none() || email.is_none() || api_token.is_none() {
            log_warning!("JIRA MCP configuration is incomplete, cannot save to config file");
            return Ok(());
        }

        // 读取现有 TOML 配置
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path.clone());
        let mut settings = manager.read()?;

        // 更新 JIRA 配置
        settings.jira = JiraSettings {
            email,
            api_token,
            service_address,
        };

        // 保存到 TOML
        manager.write(&settings)?;
        log_success!("JIRA configuration saved to {}", config_path.display());

        Ok(())
    }

    /// 保存 GitHub 配置到 TOML
    fn save_github_to_toml(github_config: &MCPServerConfig) -> Result<()> {
        use crate::base::settings::paths::Paths;
        use crate::base::settings::settings::{GitHubAccount, Settings};
        use crate::jira::config::ConfigManager;

        // 从 MCP 配置中读取 Token
        let api_token = github_config
            .env
            .get("GITHUB_PERSONAL_ACCESS_TOKEN")
            .cloned();

        if api_token.is_none() {
            log_warning!("GitHub MCP configuration is incomplete, cannot save to config file");
            return Ok(());
        }

        // 读取现有 TOML 配置
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path.clone());
        let mut settings = manager.read()?;

        // 询问账号名称和邮箱
        let account_name: String = Input::new()
            .with_prompt("Enter GitHub account name (for identification)")
            .interact_text()
            .context("Failed to get GitHub account name")?;

        let email: String = Input::new()
            .with_prompt("Enter GitHub account email")
            .interact_text()
            .context("Failed to get GitHub account email")?;

        // 检查账号是否已存在
        let existing_index = settings
            .github
            .accounts
            .iter()
            .position(|a| a.name == account_name);

        let config_path = Paths::workflow_config()?;

        if let Some(index) = existing_index {
            // 更新现有账号
            settings.github.accounts[index].api_token = api_token.unwrap();
            settings.github.accounts[index].email = email;
            // 保存到 TOML
            manager.write(&settings)?;
            log_success!(
                "GitHub account '{}' configuration updated in {}",
                account_name,
                config_path.display()
            );
        } else {
            // 添加新账号
            let new_account = GitHubAccount {
                name: account_name.clone(),
                email,
                api_token: api_token.unwrap(),
                branch_prefix: None,
            };
            settings.github.accounts.push(new_account);

            // 如果当前没有激活账号，设置为当前账号
            if settings.github.current.is_none() {
                settings.github.current = Some(account_name.clone());
            }

            // 保存到 TOML
            manager.write(&settings)?;
            log_success!(
                "GitHub account '{}' added to {}",
                account_name,
                config_path.display()
            );
        }

        Ok(())
    }

    /// 保存 Apifox 配置到 TOML
    fn save_apifox_to_toml(apifox_config: &MCPServerConfig) -> Result<()> {
        use crate::base::settings::paths::Paths;
        use crate::base::settings::settings::{ApifoxSettings, Settings};
        use crate::jira::config::ConfigManager;

        // 从 MCP 配置中读取值
        // 项目 ID 在 args 中：--project=5904781
        let project_id = apifox_config
            .args
            .iter()
            .find(|arg| arg.starts_with("--project="))
            .and_then(|arg| arg.strip_prefix("--project="))
            .map(|s| s.to_string());

        let access_token = apifox_config.env.get("APIFOX_ACCESS_TOKEN").cloned();

        if project_id.is_none() || access_token.is_none() {
            log_warning!("Apifox MCP configuration is incomplete, cannot save to config file");
            return Ok(());
        }

        // 读取现有 TOML 配置
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path.clone());
        let mut settings = manager.read()?;

        // 更新 Apifox 配置
        settings.apifox = ApifoxSettings {
            project_id,
            access_token,
        };

        // 保存到 TOML
        manager.write(&settings)?;
        log_success!("Apifox configuration saved to {}", config_path.display());

        Ok(())
    }
}

/// TOML 配置状态
#[derive(Debug)]
struct TomlConfigStatus {
    jira: JiraConfigStatus,
    github: GitHubConfigStatus,
    apifox: ApifoxConfigStatus,
}

/// JIRA 配置状态
#[derive(Debug)]
struct JiraConfigStatus {
    #[allow(dead_code)]
    has_email: bool,
    #[allow(dead_code)]
    has_api_token: bool,
    #[allow(dead_code)]
    has_service_address: bool,
    is_complete: bool,
}

/// GitHub 配置状态
#[derive(Debug)]
struct GitHubConfigStatus {
    has_accounts: bool,
    #[allow(dead_code)]
    has_current: bool,
    current_account: Option<crate::base::settings::settings::GitHubAccount>,
}

/// Apifox 配置状态
#[derive(Debug)]
struct ApifoxConfigStatus {
    #[allow(dead_code)]
    has_project_id: bool,
    #[allow(dead_code)]
    has_access_token: bool,
    is_complete: bool,
}
