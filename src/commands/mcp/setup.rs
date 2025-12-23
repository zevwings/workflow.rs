//! MCP 配置设置命令
//!
//! 交互式配置 MCP 服务器，支持从 TOML 配置自动填充。

use crate::base::dialog::{
    ConfirmDialog, InputDialog, MultiSelectDialog, FormBuilder, GroupConfig,
};
use crate::base::mcp::config::{MCPConfig, MCPConfigManager, MCPServerConfig};
use crate::base::settings::settings::Settings;
use crate::{log_break, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use dialoguer::Password;
use std::collections::{HashMap, HashSet};

/// MCP 配置设置命令
pub struct SetupCommand;

impl SetupCommand {
    /// 运行交互式配置流程
    pub fn run() -> Result<()> {
        Self::run_with_options(true, true)
    }

    /// 运行配置流程（用于 repo setup 集成）
    ///
    /// # 参数
    /// - `show_header`: 是否显示标题和分隔线
    /// - `show_completion`: 是否显示完成信息
    pub fn run_with_options(show_header: bool, show_completion: bool) -> Result<()> {
        if show_header {
            log_success!("MCP Configuration Wizard");
            log_break!('=', 40);
            log_break!();
        }

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

        for server_name in &selected_servers {
            log_break!();
            match server_name.as_str() {
                "jira" => {
                    let server_config = Self::configure_jira(&toml_status)?;
                    new_config
                        .mcp_servers
                        .insert("jira".to_string(), server_config);
                }
                "github" => {
                    let server_config = Self::configure_github(&toml_status)?;
                    new_config
                        .mcp_servers
                        .insert("github".to_string(), server_config);
                }
                _ => {
                    log_warning!("Unknown MCP server: {}", server_name);
                    continue;
                }
            }
        }

        // 4. 保存配置
        log_break!();
        log_message!("Saving MCP configuration...");
        mcp_manager.merge(&new_config)?;
        log_success!("MCP configuration saved to: {:?}", mcp_manager.config_path());

        if show_completion {
            log_break!();
            log_success!("MCP configuration completed!");
            log_message!("Next steps:");
            log_message!("  1. Restart Cursor IDE");
            log_message!("  2. Use `workflow mcp verify` to verify configuration");
        }

        Ok(())
    }

    /// 检测 TOML 配置状态
    fn detect_toml_config_status() -> TomlConfigStatus {
        let settings = Settings::get();
        TomlConfigStatus {
            jira: JiraConfigStatus {
                is_complete: settings.jira.email.is_some()
                    && settings.jira.api_token.is_some()
                    && settings.jira.service_address.is_some(),
            },
            github: GitHubConfigStatus {
                has_accounts: !settings.github.accounts.is_empty(),
                current_account: settings.github.get_current_account().cloned(),
            },
        }
    }

    /// 交互式选择要配置的 MCP 服务器
    fn select_servers(configured: &HashSet<String>) -> Result<Vec<String>> {
        let options = [
            ("JIRA MCP", "jira", configured.contains("jira")),
            ("GitHub MCP", "github", configured.contains("github")),
        ];

        let items: Vec<&str> = options.iter().map(|(name, _, _)| *name).collect();

        // 将 bool 默认值转换为索引列表
        let default_indices: Vec<usize> = options
            .iter()
            .enumerate()
            .filter_map(|(i, (_, _, is_configured))| {
                if *is_configured {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        let selected_items = MultiSelectDialog::new(
            "Select MCP servers to configure (multi-select, space to toggle, enter to confirm)",
            items,
        )
        .with_default(default_indices)
        .prompt()
        .wrap_err("Failed to get MCP server selection")?;

        // 将选中的选项名称映射回服务器名称
        let selected: Vec<String> = options
            .iter()
            .filter_map(|(name, server_name, _)| {
                if selected_items.contains(name) {
                    Some(server_name.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(selected)
    }

    /// 配置 JIRA MCP
    fn configure_jira(toml_status: &TomlConfigStatus) -> Result<MCPServerConfig> {
        log_message!("Configuring JIRA MCP...");
        log_break!('-', 40);

        let settings = Settings::get();

        // 如果 TOML 配置完整，询问是否使用
        if toml_status.jira.is_complete {
            let use_toml = ConfirmDialog::new("Complete JIRA configuration found in config file, use it?")
                .with_default(true)
                .prompt()
                .wrap_err("Failed to get confirmation")?;

            if use_toml {
                let jira = &settings.jira;
                return Ok(MCPServerConfig {
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
                });
            }
        }

        // 使用 FormBuilder 收集交互式输入
        let has_service_address = settings.jira.service_address.is_some();
        let has_email = settings.jira.email.is_some();
        let has_api_token = settings.jira.api_token.is_some();

        let jira_form_result = FormBuilder::new()
            .add_group(
                "jira_mcp",
                |g| {
                    g.step_if_dynamic(
                        |_| has_service_address,
                        |f| {
                            // 如果有 TOML 配置，先询问是否使用
                            f.add_confirmation(
                                "use_service_address",
                                format!(
                                    "Use server address from config file ({})?",
                                    settings.jira.service_address.as_ref().unwrap()
                                ),
                            )
                            .default(true)
                        },
                    )
                    .step_if_dynamic(
                        |result| {
                            // 如果没有 TOML 配置，或者用户选择不使用 TOML 配置
                            !has_service_address
                                || result.get("use_service_address") == Some(&"no".to_string())
                        },
                        |f| {
                            f.add_text("service_address", "Enter JIRA server URL").required()
                        },
                    )
                    .step_if_dynamic(
                        |_| has_email,
                        |f| {
                            // 如果有 TOML 配置，先询问是否使用
                            f.add_confirmation(
                                "use_username",
                                format!(
                                    "Use username from config file ({})?",
                                    settings.jira.email.as_ref().unwrap()
                                ),
                            )
                            .default(true)
                        },
                    )
                    .step_if_dynamic(
                        |result| {
                            // 如果没有 TOML 配置，或者用户选择不使用 TOML 配置
                            !has_email || result.get("use_username") == Some(&"no".to_string())
                        },
                        |f| {
                            f.add_text("username", "Enter JIRA username/email").required()
                        },
                    )
                    .step_if_dynamic(
                        |_| has_api_token,
                        |f| {
                            // 如果有 TOML 配置，先询问是否使用
                            f.add_confirmation("use_api_token", "Use API token from config file?")
                                .default(true)
                        },
                    )
                    .step_if_dynamic(
                        |result| {
                            // 如果没有 TOML 配置，或者用户选择不使用 TOML 配置
                            !has_api_token || result.get("use_api_token") == Some(&"no".to_string())
                        },
                        |f| {
                            f.add_password("api_token", "Enter JIRA API token").required()
                        },
                    )
                },
                GroupConfig::required()
                    .with_title("JIRA MCP Configuration"),
            )
            .run()
            .wrap_err("Failed to collect JIRA configuration")?;

        // 从 form 结果中提取值
        let service_address = if has_service_address
            && jira_form_result.get("use_service_address") == Some(&"yes".to_string())
        {
            settings.jira.service_address.as_ref().unwrap().clone()
        } else {
            jira_form_result
                .get_required("service_address")
                .wrap_err("JIRA server URL is required")?
        };

        let username = if has_email
            && jira_form_result.get("use_username") == Some(&"yes".to_string())
        {
            settings.jira.email.as_ref().unwrap().clone()
        } else {
            jira_form_result
                .get_required("username")
                .wrap_err("JIRA username is required")?
        };

        let api_token = if has_api_token
            && jira_form_result.get("use_api_token") == Some(&"yes".to_string())
        {
            settings.jira.api_token.as_ref().unwrap().clone()
        } else {
            jira_form_result
                .get_required("api_token")
                .wrap_err("JIRA API token is required")?
        };

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
    fn configure_github(toml_status: &TomlConfigStatus) -> Result<MCPServerConfig> {
        log_message!("Configuring GitHub MCP...");
        log_break!('-', 40);

        // 使用 FormBuilder 收集交互式输入
        let has_account = toml_status.github.has_accounts
            && toml_status.github.current_account.is_some();

        let github_form_result = FormBuilder::new()
            .add_group(
                "github_mcp",
                |g| {
                    g.step_if_dynamic(
                        |_| has_account,
                        |f| {
                            // 如果有 TOML 配置，先询问是否使用
                            if let Some(account) = &toml_status.github.current_account {
                                f.add_confirmation(
                                    "use_api_token",
                                    format!(
                                        "Use token from current account {} ({})?",
                                        account.name, account.email
                                    ),
                                )
                                .default(true)
                            } else {
                                // 这个分支不应该到达，但为了类型安全
                                f.add_confirmation("use_api_token", "Enter GitHub token")
                                    .default(false)
                            }
                        },
                    )
                    .step_if_dynamic(
                        |result| {
                            // 如果没有 TOML 配置，或者用户选择不使用 TOML 配置
                            !has_account || result.get("use_api_token") == Some(&"no".to_string())
                        },
                        |f| {
                            f.add_password("api_token", "Enter GitHub Personal Access Token")
                                .required()
                        },
                    )
                },
                GroupConfig::required()
                    .with_title("GitHub MCP Configuration"),
            )
            .run()
            .wrap_err("Failed to collect GitHub configuration")?;

        // 从 form 结果中提取值
        let api_token = if has_account
            && github_form_result.get("use_api_token") == Some(&"yes".to_string())
        {
            // 如果用户选择使用 TOML 配置，直接返回
            let account = toml_status
                .github
                .current_account
                .as_ref()
                .unwrap();
            return Ok(MCPServerConfig {
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
            });
        } else {
            github_form_result
                .get_required("api_token")
                .wrap_err("GitHub API token is required")?
        };

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
}

/// TOML 配置状态
#[derive(Debug)]
struct TomlConfigStatus {
    jira: JiraConfigStatus,
    github: GitHubConfigStatus,
}

/// JIRA 配置状态
#[derive(Debug)]
struct JiraConfigStatus {
    is_complete: bool,
}

/// GitHub 配置状态
#[derive(Debug)]
struct GitHubConfigStatus {
    has_accounts: bool,
    current_account: Option<crate::base::settings::settings::GitHubAccount>,
}
