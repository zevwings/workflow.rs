//! Jira configuration management subcommands
//!
//! Jira 配置管理子命令结构定义

use clap::{Args, Subcommand};

use super::args::JiraIdArg;

/// Jira 配置管理子命令
#[derive(Subcommand)]
pub enum JiraCommand {
    /// 检查 Jira 配置（显示服务地址、邮箱、验证状态）
    Check,
    /// 设置 Jira 配置（交互式配置服务地址、邮箱、API Token）
    Setup,
    /// 显示 Jira ticket 信息
    Info(InfoArgs),
    /// 下载 Jira ticket 的所有附件
    Attachments(AttachmentsArgs),
    /// 清理 Jira 附件目录
    Clean(CleanArgs),
    /// 设置 Jira 状态配置
    Status(JiraIdArg),
    /// 过渡 Jira 状态
    Transition(JiraIdArg),
    /// 分配 Jira ticket 给当前用户
    Assign(JiraIdArg),
}

/// Info 命令参数
#[derive(Args, Debug, Clone)]
pub struct InfoArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
    /// JSON 格式输出
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub json: bool,
    /// Markdown 格式输出
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub markdown: bool,
}

/// Attachments 命令参数
#[derive(Args, Debug, Clone)]
pub struct AttachmentsArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
}

/// Clean 命令参数
#[derive(Args, Debug, Clone)]
pub struct CleanArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
    /// 清理所有附件目录
    #[arg(long, short = 'a', action = clap::ArgAction::SetTrue)]
    pub all: bool,
}
