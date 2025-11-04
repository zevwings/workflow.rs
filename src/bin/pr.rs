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
        pr_id: Option<String>,

        /// Force merge
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// Show PR status information
    Status {
        /// PR ID or branch name
        #[arg(value_name = "PR_ID_OR_BRANCH")]
        pr_id_or_branch: Option<String>,
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
            create::PRCreateCommand::create(jira_ticket, title, description, dry_run)?;
        }
        PRCommands::Merge { pr_id, force } => {
            merge::PRMergeCommand::merge(pr_id, force)?;
        }
        PRCommands::Status { pr_id_or_branch } => {
            status::PRStatusCommand::show(pr_id_or_branch)?;
        }
        PRCommands::List { state, limit } => {
            list::PRListCommand::list(state, limit)?;
        }
        PRCommands::Update => {
            update::PRUpdateCommand::update()?;
        }
    }

    Ok(())
}

