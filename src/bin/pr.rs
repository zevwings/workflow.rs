//! PR 命令入口
//!
//! 这是独立的 `pr` 命令入口，提供 Pull Request 的创建、合并、关闭、查询等操作。
//! 支持 GitHub 和 Codeup 两种代码托管平台。

use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::pr::{close, create, integrate, list, merge, status, update};

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

/// PR commands enumeration
///
/// Defines all PR-related subcommands.
#[derive(Subcommand)]
enum PRCommands {
    /// Create a new Pull Request
    ///
    /// Supports auto-detection of repository type (GitHub/Codeup), and optionally uses AI to generate PR title.
    /// If a Jira ticket is provided, will automatically update Jira status.
    Create {
        /// Jira ticket ID (optional, e.g., PROJ-123)
        #[arg(value_name = "JIRA_TICKET")]
        jira_ticket: Option<String>,

        /// PR title (optional, will use AI generation if not provided)
        #[arg(short, long)]
        title: Option<String>,

        /// Short description (optional)
        #[arg(short, long)]
        description: Option<String>,

        /// Dry run mode (don't actually create PR, only show what would be done)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    /// Merge a Pull Request
    ///
    /// Auto-detect PR corresponding to current branch, or manually specify PR ID.
    /// Will automatically update corresponding Jira ticket status after merging.
    Merge {
        /// PR ID (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,

        /// Force merge (skip checks)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// Show PR status information
    ///
    /// Display detailed information about a specific PR, including status, author, comments, etc.
    Status {
        /// PR ID or branch name (optional, auto-detect from current branch if not provided)
        #[arg(value_name = "PR_ID_OR_BRANCH")]
        pull_request_id_or_branch: Option<String>,
    },
    /// List Pull Requests
    ///
    /// List all PRs in the repository, supports filtering by status and limiting the number of results.
    List {
        /// Filter by state (open, closed, merged)
        #[arg(short, long)]
        state: Option<String>,

        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Update code (use PR title as commit message)
    ///
    /// Commit current changes to PR branch using PR title as commit message.
    Update,
    /// Integrate branch into current branch
    ///
    /// Merge specified branch into current branch, and optionally push to remote.
    /// This is a local Git operation, different from the `merge` command (which merges PR via API).
    Integrate {
        /// Source branch name to merge (required)
        #[arg(value_name = "SOURCE_BRANCH")]
        source_branch: String,

        /// Only allow fast-forward merge (fail if not possible)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        ff_only: bool,

        /// Use squash merge (compress all commits into one)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        squash: bool,

        /// Don't push to remote (pushes by default)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        no_push: bool,
    },
    /// Close a Pull Request
    ///
    /// Close PR corresponding to current branch, delete remote branch, and switch to default branch.
    Close {
        /// PR ID (optional, auto-detect from current branch if not provided)
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
            list::PullRequestListCommand::list(state, limit)?;
        }
        PRCommands::Update => {
            update::PullRequestUpdateCommand::update()?;
        }
        PRCommands::Integrate {
            source_branch,
            ff_only,
            squash,
            no_push,
        } => {
            // 默认推送，除非指定了 --no-push
            let should_push = !no_push;
            integrate::PullRequestIntegrateCommand::integrate(
                source_branch,
                ff_only,
                squash,
                should_push,
            )?;
        }
        PRCommands::Close { pull_request_id } => {
            close::PullRequestCloseCommand::close(pull_request_id)?;
        }
    }

    Ok(())
}
