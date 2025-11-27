//! MCP 配置验证命令
//!
//! 验证 MCP 配置是否正确，检查必要的环境变量和命令是否可用。

use crate::base::mcp::config::MCPConfigManager;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::Result;
use std::process::Command;

/// MCP 配置验证命令
pub struct VerifyCommand;

impl VerifyCommand {
    /// 运行验证流程
    pub fn run() -> Result<()> {
        log_success!("Verify MCP Configuration");
        log_break!('=', 40);
        log_break!();

        let manager = MCPConfigManager::new()?;
        let config = manager.read()?;

        if config.mcp_servers.is_empty() {
            log_warning!("No MCP servers configured");
            log_message!("Use `workflow mcp setup` to configure");
            return Ok(());
        }

        log_info!("Config file path: {:?}\n", manager.config_path());
        log_info!("Configured {} MCP server(s)\n", config.mcp_servers.len());
        log_break!();

        let mut all_valid = true;

        // 检查 npx 是否可用
        if !Self::check_npx()? {
            log_warning!("npx is not installed or not available");
            log_message!("   MCP servers require npx to run");
            log_message!("   Please install Node.js: https://nodejs.org/");
            all_valid = false;
        } else {
            log_success!("npx is available");
        }

        log_break!();

        // 验证每个服务器配置
        for (name, server) in &config.mcp_servers {
            log_break!('-', 40, &format!("Verify {} MCP", name.to_uppercase()));

            let mut server_valid = true;

            // 检查命令
            if server.command != "npx" {
                log_warning!("Unknown command: {}", server.command);
                server_valid = false;
            } else {
                log_success!("Command: {}", server.command);
            }

            // 检查必要的环境变量
            match name.as_str() {
                "jira" => {
                    if !server.env.contains_key("JIRA_SERVER_URL") {
                        log_warning!("Missing environment variable: JIRA_SERVER_URL");
                        server_valid = false;
                    } else {
                        log_success!("JIRA_SERVER_URL is set");
                    }

                    if !server.env.contains_key("JIRA_USERNAME") {
                        log_warning!("Missing environment variable: JIRA_USERNAME");
                        server_valid = false;
                    } else {
                        log_success!("JIRA_USERNAME is set");
                    }

                    if !server.env.contains_key("JIRA_API_TOKEN") {
                        log_warning!("Missing environment variable: JIRA_API_TOKEN");
                        server_valid = false;
                    } else {
                        log_success!("JIRA_API_TOKEN is set");
                    }
                }
                "github" => {
                    if !server.env.contains_key("GITHUB_PERSONAL_ACCESS_TOKEN") {
                        log_warning!("Missing environment variable: GITHUB_PERSONAL_ACCESS_TOKEN");
                        server_valid = false;
                    } else {
                        log_success!("GITHUB_PERSONAL_ACCESS_TOKEN is set");
                    }
                }
                "apifox" => {
                    if !server.env.contains_key("APIFOX_ACCESS_TOKEN") {
                        log_warning!("Missing environment variable: APIFOX_ACCESS_TOKEN");
                        server_valid = false;
                    } else {
                        log_success!("APIFOX_ACCESS_TOKEN is set");
                    }

                    // 检查项目 ID 参数
                    let has_project = server.args.iter().any(|arg| arg.starts_with("--project="));
                    if !has_project {
                        log_warning!("Missing argument: --project=<PROJECT_ID>");
                        server_valid = false;
                    } else {
                        log_success!("Project ID argument is set");
                    }
                }
                _ => {
                    log_warning!("Unknown MCP server type: {}", name);
                }
            }

            if !server_valid {
                all_valid = false;
            }

            log_break!();
        }

        log_break!('=', 40);
        if all_valid {
            log_success!("All configuration verified!");
            log_message!("Next steps:");
            log_message!("  1. Restart Cursor IDE");
            log_message!("  2. Test MCP functionality in Cursor");
        } else {
            log_warning!("Some configuration issues found, please check the warnings above");
            log_message!("Use `workflow mcp update` to update configuration");
        }

        Ok(())
    }

    /// 检查 npx 是否可用
    fn check_npx() -> Result<bool> {
        let output = Command::new("npx").arg("--version").output();

        match output {
            Ok(result) => Ok(result.status.success()),
            Err(_) => Ok(false),
        }
    }
}
