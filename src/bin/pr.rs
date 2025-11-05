use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::pr::{create, list, merge, status, update};

#[derive(Parser)]
#[command(name = "pr")]
#[command(about = "Pull Request operations", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    subcommand: PRCommands,
}

#[derive(Subcommand)]
enum PRCommands {
    /// Create a new Pull Request
    Create {
        /// Jira ticket (optional)
        #[arg(value_name = "JIRA_TICKET")]
        jira_ticket: Option<String>,

        /// PR title
        #[arg(short, long)]
        title: Option<String>,

        /// Short description
        #[arg(short, long)]
        description: Option<String>,

        /// Dry run (don't actually create PR)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    /// Merge a Pull Request
    Merge {
        /// PR ID (if not provided, auto-detect from current branch)
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,

        /// Force merge
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// Show PR status information
    Status {
        /// PR ID or branch name
        #[arg(value_name = "PR_ID_OR_BRANCH")]
        pull_request_id_or_branch: Option<String>,
    },
    /// List PRs
    List {
        /// Filter by state (open, closed, merged)
        #[arg(short, long)]
        state: Option<String>,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Update code (use PR title as commit message)
    Update,
}

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
        PRCommands::Merge { pull_request_id, force } => {
            merge::PullRequestMergeCommand::merge(pull_request_id, force)?;
        }
        PRCommands::Status { pull_request_id_or_branch } => {
            status::PullRequestStatusCommand::show(pull_request_id_or_branch)?;
        }
        PRCommands::List { state, limit } => {
            list::GetPullRequestsCommand::list(state, limit)?;
        }
        PRCommands::Update => {
            update::PullRequestUpdateCommand::update()?;
        }
    }

    Ok(())
}
