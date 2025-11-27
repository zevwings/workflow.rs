//! MCP 配置查看命令
//!
//! 显示当前 MCP 配置状态。

use crate::base::mcp::config::MCPConfigManager;
use crate::base::util::mask_sensitive_value;
use crate::{log_break, log_info, log_message, log_warning};
use anyhow::Result;

/// MCP 配置查看命令
pub struct ShowCommand;

impl ShowCommand {
    /// 显示 MCP 配置
    pub fn show(full: bool) -> Result<()> {
        log_break!('=', 40, "MCP Configuration");
        log_break!();

        let manager = MCPConfigManager::new()?;
        let config = manager.read()?;

        log_info!("Config file path: {:?}\n", manager.config_path());

        if config.mcp_servers.is_empty() {
            log_warning!("No MCP servers configured");
            log_message!("Use `workflow mcp setup` to configure");
            return Ok(());
        }

        log_info!("Configured MCP servers: {}\n", config.mcp_servers.len());
        log_break!();

        for (name, server) in &config.mcp_servers {
            log_break!('-', 40, &format!("{} MCP", name.to_uppercase()));
            log_message!("Command: {}", server.command);
            log_message!("Arguments: {}", server.args.join(" "));

            if !server.env.is_empty() {
                log_message!("Environment variables:");
                for (key, value) in &server.env {
                    if full {
                        log_message!("  {} = {}", key, value);
                    } else {
                        // 使用已封装的方法掩码敏感信息
                        let masked = if key.contains("TOKEN") || key.contains("PASSWORD") {
                            mask_sensitive_value(value.as_str())
                        } else {
                            value.clone()
                        };
                        log_message!("  {} = {}", key, masked);
                    }
                }
            }
            log_break!();
        }

        Ok(())
    }
}
