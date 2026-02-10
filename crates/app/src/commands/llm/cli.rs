//! LLM configuration management subcommands
//!
//! LLM 配置管理子命令结构定义

use clap::Subcommand;

/// LLM 配置管理子命令
#[derive(Subcommand)]
pub enum LlmCommand {
    /// 显示当前 LLM 配置（显示提供者、API Key（已掩码）、模型、语言设置）
    Check,
    /// 设置 LLM 配置（交互式配置提供者、代理 URL、API Key、模型、语言设置）
    Setup,
}
