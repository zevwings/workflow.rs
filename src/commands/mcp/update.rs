//! MCP 配置更新命令
//!
//! 交互式更新已配置的 MCP 服务器。

use crate::base::mcp::config::{MCPConfigManager, MCPServerConfig};
use crate::base::settings::settings::Settings;
use crate::{log_break, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Password, Select};
use std::collections::HashMap;

/// MCP 配置更新命令
pub struct UpdateCommand;

impl UpdateCommand {
    /// 运行交互式更新流程
    pub fn run() -> Result<()> {
        log_success!("Update MCP Configuration");
        log_break!('=', 40);
        log_break!();

        let manager = MCPConfigManager::new()?;
        let config = manager.read()?;

        if config.mcp_servers.is_empty() {
            log_warning!("No MCP servers configured");
            log_message!("Use `workflow mcp setup` to configure");
            return Ok(());
        }

        // 让用户选择要更新的服务器
        let server_names: Vec<&String> = config.mcp_servers.keys().collect();
        let selection = Select::new()
            .with_prompt("Select MCP server to update")
            .items(&server_names)
            .interact()
            .context("Failed to get server selection")?;

        let server_name = server_names[selection].clone();
        let existing_server = config.mcp_servers.get(&server_name).unwrap();

        log_break!();
        log_message!(
            "Updating {} MCP configuration...",
            server_name.to_uppercase()
        );
        log_break!('-', 40);

        // 根据服务器类型更新配置
        let updated_server = match server_name.as_str() {
            "jira" => Self::update_jira(existing_server)?,
            "github" => Self::update_github(existing_server)?,
            "apifox" => Self::update_apifox(existing_server)?,
            _ => {
                log_warning!("Unknown MCP server type: {}", server_name);
                return Ok(());
            }
        };

        // 保存更新
        manager.update(|config| {
            config
                .mcp_servers
                .insert(server_name.clone(), updated_server);
        })?;

        log_break!();
        log_success!("Configuration updated!");
        log_message!("Please restart Cursor IDE for changes to take effect");

        Ok(())
    }

    /// 更新 JIRA MCP 配置
    fn update_jira(existing: &MCPServerConfig) -> Result<MCPServerConfig> {
        let settings = Settings::get();

        // 获取现有值
        let current_url = existing.env.get("JIRA_SERVER_URL").cloned();
        let current_username = existing.env.get("JIRA_USERNAME").cloned();
        let current_token = existing.env.get("JIRA_API_TOKEN").cloned();

        // 询问是否使用 TOML 配置
        let use_toml = if settings.jira.service_address.is_some()
            && settings.jira.email.is_some()
            && settings.jira.api_token.is_some()
        {
            Confirm::new()
                .with_prompt("Use JIRA information from config file?")
                .default(true)
                .interact()?
        } else {
            false
        };

        let service_address = if use_toml {
            settings.jira.service_address.clone().unwrap()
        } else {
            let prompt = if let Some(ref url) = current_url {
                format!("JIRA server URL [current: {}]", url)
            } else {
                "JIRA server URL".to_string()
            };
            Input::<String>::new()
                .with_prompt(&prompt)
                .default(current_url.unwrap_or_default())
                .interact_text()?
        };

        let username = if use_toml {
            settings.jira.email.clone().unwrap()
        } else {
            let prompt = if let Some(ref user) = current_username {
                format!("JIRA username/email [current: {}]", user)
            } else {
                "JIRA username/email".to_string()
            };
            Input::<String>::new()
                .with_prompt(&prompt)
                .default(current_username.unwrap_or_default())
                .interact_text()?
        };

        let api_token = if use_toml {
            settings.jira.api_token.clone().unwrap()
        } else {
            let prompt = if current_token.is_some() {
                "JIRA API token (leave empty to keep current, enter new value to update)"
            } else {
                "JIRA API token"
            };
            let new_token = Password::new().with_prompt(prompt).interact()?;
            if new_token.is_empty() {
                current_token.unwrap_or_default()
            } else {
                new_token
            }
        };

        Ok(MCPServerConfig {
            command: existing.command.clone(),
            args: existing.args.clone(),
            env: {
                let mut env = HashMap::new();
                env.insert("JIRA_SERVER_URL".to_string(), service_address);
                env.insert("JIRA_USERNAME".to_string(), username);
                env.insert("JIRA_API_TOKEN".to_string(), api_token);
                env
            },
        })
    }

    /// 更新 GitHub MCP 配置
    fn update_github(existing: &MCPServerConfig) -> Result<MCPServerConfig> {
        let settings = Settings::get();
        let current_token = existing.env.get("GITHUB_PERSONAL_ACCESS_TOKEN").cloned();

        // 询问是否使用 TOML 配置
        if let Some(account) = settings.github.get_current_account() {
            let use_toml = Confirm::new()
                .with_prompt(format!(
                    "Use token from current GitHub account {}?",
                    account.name
                ))
                .default(true)
                .interact()?;

            if use_toml {
                return Ok(MCPServerConfig {
                    command: existing.command.clone(),
                    args: existing.args.clone(),
                    env: {
                        let mut env = HashMap::new();
                        env.insert(
                            "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                            account.api_token.clone(),
                        );
                        env
                    },
                });
            }
        }

        // 交互式输入
        let prompt = if current_token.is_some() {
            "GitHub Personal Access Token (leave empty to keep current, enter new value to update)"
        } else {
            "GitHub Personal Access Token"
        };
        let new_token = Password::new().with_prompt(prompt).interact()?;
        let api_token = if new_token.is_empty() {
            current_token.unwrap_or_default()
        } else {
            new_token
        };

        Ok(MCPServerConfig {
            command: existing.command.clone(),
            args: existing.args.clone(),
            env: {
                let mut env = HashMap::new();
                env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), api_token);
                env
            },
        })
    }

    /// 更新 Apifox MCP 配置
    fn update_apifox(existing: &MCPServerConfig) -> Result<MCPServerConfig> {
        // 从 args 中提取项目 ID
        let current_project_id = existing
            .args
            .iter()
            .find_map(|arg| {
                if arg.starts_with("--project=") {
                    Some(arg.strip_prefix("--project=").unwrap().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let current_token = existing.env.get("APIFOX_ACCESS_TOKEN").cloned();

        let project_id: String = Input::new()
            .with_prompt(format!(
                "Apifox project ID [current: {}]",
                if current_project_id.is_empty() {
                    "not set"
                } else {
                    &current_project_id
                }
            ))
            .default(current_project_id)
            .interact_text()?;

        let prompt = if current_token.is_some() {
            "Apifox access token (leave empty to keep current, enter new value to update)"
        } else {
            "Apifox access token"
        };
        let new_token = Password::new().with_prompt(prompt).interact()?;
        let access_token = if new_token.is_empty() {
            current_token.unwrap_or_default()
        } else {
            new_token
        };

        Ok(MCPServerConfig {
            command: existing.command.clone(),
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
        })
    }
}
