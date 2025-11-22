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
    /// Jira ticket ID (e.g., PROJ-123)
    #[arg(value_name = "JIRA_ID")]
    jira_id: String,

    #[command(subcommand)]
    subcommand: Option<QkCommands>,
}

/// QK commands enumeration
///
/// Defines all quick log operation subcommands.
#[derive(Subcommand)]
enum QkCommands {
    /// Download log files
    ///
    /// Download log files from Jira ticket attachments (supports automatic merging of split files).
    /// Log files will be saved locally with paths automatically resolved based on JIRA ID.
    Download {
        /// Download all attachments (not just log files)
        #[arg(long, short = 'a')]
        all: bool,
    },
    /// Find request ID
    ///
    /// Find specified request ID in log files and extract corresponding response content.
    /// If found, will copy response content to clipboard and automatically open browser.
    Find {
        /// Request ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "REQUEST_ID")]
        request_id: Option<String>,
    },
    /// Search for keywords
    ///
    /// Search for specified keywords in log files and return all matching request information.
    Search {
        /// Search keyword (optional, will prompt interactively if not provided)
        #[arg(value_name = "SEARCH_TERM")]
        search_term: Option<String>,
    },
    /// Clean log directory
    ///
    /// Delete log directory for specified JIRA ID and all its contents.
    /// Confirmation is required before deletion.
    Clean {
        /// Preview operation without actually deleting
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Only list what would be deleted
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
