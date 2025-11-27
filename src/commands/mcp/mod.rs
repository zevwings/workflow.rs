//! MCP 配置管理命令
//!
//! 提供 MCP (Model Context Protocol) 配置的交互式管理功能。

pub mod setup;
pub mod show;
pub mod update;
pub mod verify;

use crate::cli::MCPSubcommand;
use anyhow::Result;

/// 处理 MCP 子命令
pub fn handle_mcp_command(subcommand: MCPSubcommand) -> Result<()> {
    match subcommand {
        MCPSubcommand::Setup => setup::SetupCommand::run(),
        MCPSubcommand::Show { full } => show::ShowCommand::show(full),
        MCPSubcommand::Update => update::UpdateCommand::run(),
        MCPSubcommand::Verify => verify::VerifyCommand::run(),
    }
}
