//! Jira 配置管理子命令
//!
//! Jira 配置管理子命令结构定义

use clap::{ArgAction, Args, Subcommand};

use super::super::args::JiraIdArg;

/// 输出格式（用于 `workflow jira info`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// 人类可读格式（默认）
    #[default]
    HumanReadable,
    /// JSON 格式
    Json,
    /// Markdown 格式
    Markdown,
}

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
    /// 向 Jira ticket 添加评论
    #[cfg(feature = "develop")]
    Comment(CommentArgs),
    /// 设置 Jira 状态配置
    #[cfg(feature = "develop")]
    Status(JiraIdArg),
    /// 过渡 Jira 状态
    #[cfg(feature = "develop")]
    Transition(JiraIdArg),
    /// 分配 Jira ticket 给当前用户
    #[cfg(feature = "develop")]
    Assign(JiraIdArg),
}

/// Info 命令参数
#[derive(Args, Debug, Clone)]
pub struct InfoArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
    /// JSON 格式输出
    #[arg(long, action = ArgAction::SetTrue)]
    pub json: bool,
    /// Markdown 格式输出
    #[arg(long, action = ArgAction::SetTrue)]
    pub markdown: bool,
}

impl InfoArgs {
    /// 将布尔参数转换为输出格式枚举
    pub fn get_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else if self.markdown {
            OutputFormat::Markdown
        } else {
            OutputFormat::HumanReadable
        }
    }
}

/// Attachments 命令参数
#[derive(Args, Debug, Clone)]
pub struct AttachmentsArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
}

/// Comment 命令参数
#[cfg(feature = "develop")]
#[derive(Args, Debug, Clone)]
pub struct CommentArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
    /// 评论内容
    #[arg(long, short = 'm', value_name = "MESSAGE", required = true)]
    pub message: String,
}

/// Clean 命令参数
#[derive(Args, Debug, Clone)]
pub struct CleanArgs {
    /// Jira ticket ID（可选，不提供会交互式输入）
    #[command(flatten)]
    pub jira_id: JiraIdArg,
    /// 清理所有附件目录
    #[arg(long, short = 'a', action = ArgAction::SetTrue)]
    pub all: bool,
}
