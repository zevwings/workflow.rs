//! MCP 配置管理命令
//!
//! 提供 MCP (Model Context Protocol) 配置的交互式管理功能。

pub mod setup;

use color_eyre::Result;

/// MCP 子命令枚举（简化版，用于内部集成）
pub enum MCPSubcommand {
    Setup,
}

/// 处理 MCP 子命令
pub fn handle_mcp_command(subcommand: MCPSubcommand) -> Result<()> {
    match subcommand {
        MCPSubcommand::Setup => setup::SetupCommand::run(),
    }
}
