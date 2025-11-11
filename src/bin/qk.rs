//! 快速日志操作命令入口
//!
//! 这是独立的 `qk` 命令入口，提供快速日志操作功能：
//! - 从 Jira ticket 下载日志文件
//! - 在日志文件中查找请求 ID
//! - 在日志文件中搜索关键词
//!
//! 所有操作都会根据 JIRA ID 自动解析日志文件路径，无需手动指定。

use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::qk::{
    CleanCommand, DownloadCommand, FindCommand, InfoCommand, SearchCommand,
};

/// CLI 主结构体
///
/// 使用 clap 进行命令行参数解析，需要提供 JIRA ID 和可选的子命令。
/// 如果不提供子命令，将显示 ticket 信息。
#[derive(Parser)]
#[command(name = "qk")]
#[command(about = "Quick log operations (unified wrapper)", long_about = None)]
#[command(version)]
struct Cli {
    /// Jira ticket ID（如 PROJ-123）
    #[arg(value_name = "JIRA_ID")]
    jira_id: String,

    #[command(subcommand)]
    subcommand: Option<QkCommands>,
}

/// QK 命令枚举
///
/// 定义了所有快速日志操作的子命令。
#[derive(Subcommand)]
enum QkCommands {
    /// 下载日志文件
    ///
    /// 从 Jira ticket 的附件中下载日志文件（支持分片文件自动合并）。
    /// 日志文件会保存到本地，路径根据 JIRA ID 自动解析。
    Download {
        /// 下载所有附件（不仅仅是日志附件）
        #[arg(long, short = 'a')]
        all: bool,
    },
    /// 查找请求 ID
    ///
    /// 在日志文件中查找指定的请求 ID，并提取对应的响应内容。
    /// 如果找到，会将响应内容复制到剪贴板并自动打开浏览器查看。
    Find {
        /// 请求 ID（可选，不提供时会交互式输入）
        #[arg(value_name = "REQUEST_ID")]
        request_id: Option<String>,
    },
    /// 搜索关键词
    ///
    /// 在日志文件中搜索指定的关键词，返回所有匹配的请求信息。
    Search {
        /// 搜索关键词（可选，不提供时会交互式输入）
        #[arg(value_name = "SEARCH_TERM")]
        search_term: Option<String>,
    },
    /// 清理日志目录
    ///
    /// 删除指定 JIRA ID 的日志目录及其所有内容。
    /// 需要确认才能执行删除操作。
    Clean {
        /// 预览操作，不实际删除
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// 只列出将要删除的内容
        #[arg(long, short = 'l')]
        list: bool,
    },
}

/// 主函数
///
/// 解析命令行参数并分发到相应的命令处理函数。
/// 如果不提供子命令，将显示 ticket 信息。
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        Some(QkCommands::Download { all }) => {
            DownloadCommand::download(&cli.jira_id, all)?;
        }
        Some(QkCommands::Find { request_id }) => {
            FindCommand::find_request_id(&cli.jira_id, request_id)?;
        }
        Some(QkCommands::Search { search_term }) => {
            SearchCommand::search(&cli.jira_id, search_term)?;
        }
        Some(QkCommands::Clean { dry_run, list }) => {
            CleanCommand::clean(&cli.jira_id, dry_run, list)?;
        }
        None => {
            // 如果没有提供子命令，显示 ticket 信息
            InfoCommand::show(&cli.jira_id)?;
        }
    }

    Ok(())
}
