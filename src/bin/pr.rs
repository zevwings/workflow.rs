//! PR 命令入口
//!
//! 这是独立的 `pr` 命令入口，提供 Pull Request 的创建、合并、查询等操作。
//! 支持 GitHub 和 Codeup 两种代码托管平台。

use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::pr::{create, list, merge, status, update, close};

/// CLI 主结构体
///
/// 使用 clap 进行命令行参数解析，支持子命令模式。
#[derive(Parser)]
#[command(name = "pr")]
#[command(about = "Pull Request operations", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    subcommand: PRCommands,
}

/// PR 命令枚举
///
/// 定义了所有 PR 相关的子命令。
#[derive(Subcommand)]
enum PRCommands {
    /// 创建新的 Pull Request
    ///
    /// 支持自动检测仓库类型（GitHub/Codeup），并可选择使用 AI 生成 PR 标题。
    /// 如果提供 Jira ticket，会自动更新 Jira 状态。
    Create {
        /// Jira ticket ID（可选，如 PROJ-123）
        #[arg(value_name = "JIRA_TICKET")]
        jira_ticket: Option<String>,

        /// PR 标题（可选，不提供时使用 AI 生成）
        #[arg(short, long)]
        title: Option<String>,

        /// 简短描述（可选）
        #[arg(short, long)]
        description: Option<String>,

        /// 干运行模式（不实际创建 PR，仅显示将要执行的操作）
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    /// 合并 Pull Request
    ///
    /// 自动检测当前分支对应的 PR，或手动指定 PR ID。
    /// 合并后会自动更新对应的 Jira ticket 状态。
    Merge {
        /// PR ID（可选，不提供时自动检测当前分支）
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,

        /// 强制合并（跳过检查）
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// 显示 PR 状态信息
    ///
    /// 显示指定 PR 的详细信息，包括状态、作者、评论等。
    Status {
        /// PR ID 或分支名（可选，不提供时自动检测当前分支）
        #[arg(value_name = "PR_ID_OR_BRANCH")]
        pull_request_id_or_branch: Option<String>,
    },
    /// 列出 PR
    ///
    /// 列出仓库中的所有 PR，支持按状态过滤和限制数量。
    List {
        /// 按状态过滤（open, closed, merged）
        #[arg(short, long)]
        state: Option<String>,

        /// 限制结果数量
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// 更新代码（使用 PR 标题作为提交信息）
    ///
    /// 将当前更改提交到 PR 分支，使用 PR 标题作为提交信息。
    Update,
    /// 关闭 Pull Request
    ///
    /// 关闭当前分支对应的 PR，删除远程分支，并切换到默认分支。
    Close {
        /// PR ID（可选，不提供时自动检测当前分支）
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,
    },
}

/// 主函数
///
/// 解析命令行参数并分发到相应的命令处理函数。
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        PRCommands::Create {
            jira_ticket,
            title,
            description,
            dry_run,
        } => {
            create::PullRequestCreateCommand::create(jira_ticket, title, description, dry_run)?;
        }
        PRCommands::Merge {
            pull_request_id,
            force,
        } => {
            merge::PullRequestMergeCommand::merge(pull_request_id, force)?;
        }
        PRCommands::Status {
            pull_request_id_or_branch,
        } => {
            status::PullRequestStatusCommand::show(pull_request_id_or_branch)?;
        }
        PRCommands::List { state, limit } => {
            list::GetPullRequestsCommand::list(state, limit)?;
        }
        PRCommands::Update => {
            update::PullRequestUpdateCommand::update()?;
        }
        PRCommands::Close { pull_request_id } => {
            close::PullRequestCloseCommand::close(pull_request_id)?;
        }
    }

    Ok(())
}
