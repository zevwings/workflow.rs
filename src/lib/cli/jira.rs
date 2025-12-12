//! Jira operations subcommands

use clap::Subcommand;

use super::common::{DryRunArgs, JiraIdArg, OutputFormatArgs};
use super::log::LogSubcommand;

/// Jira operations subcommands
///
/// Used to manage Jira ticket operations.
#[derive(Subcommand)]
pub enum JiraSubcommand {
    /// Show ticket information
    ///
    /// Display detailed information about a Jira ticket.
    Info {
        #[command(flatten)]
        jira_id: JiraIdArg,

        #[command(flatten)]
        output_format: OutputFormatArgs,
    },
    /// Show related PRs and branches for a Jira ticket
    ///
    /// Display all Pull Requests and Git branches associated with a Jira ticket.
    Related {
        #[command(flatten)]
        jira_id: JiraIdArg,

        #[command(flatten)]
        output_format: OutputFormatArgs,
    },
    /// Show changelog (change history) for a Jira ticket
    ///
    /// Display change history for a Jira ticket.
    Changelog {
        #[command(flatten)]
        jira_id: JiraIdArg,

        #[command(flatten)]
        output_format: OutputFormatArgs,
    },
    /// Add a comment to a Jira ticket
    ///
    /// Add a comment to a Jira ticket interactively.
    /// You will be prompted to enter the comment message and optionally attach a file.
    Comment {
        #[command(flatten)]
        jira_id: JiraIdArg,
    },
    /// Show comments for a Jira ticket
    ///
    /// Display all comments for a Jira ticket with filtering and pagination options.
    Comments {
        #[command(flatten)]
        jira_id: JiraIdArg,

        /// Limit number of comments to display
        #[arg(long, value_name = "LIMIT")]
        limit: Option<usize>,

        /// Offset for pagination
        #[arg(long, value_name = "OFFSET")]
        offset: Option<usize>,

        /// Filter comments by author email
        #[arg(long, value_name = "EMAIL")]
        author: Option<String>,

        /// Filter comments since date (ISO 8601 format)
        #[arg(long, value_name = "DATE")]
        since: Option<String>,

        #[command(flatten)]
        output_format: OutputFormatArgs,
    },
    /// Download all attachments from Jira ticket
    ///
    /// Download all attachments from Jira ticket (not just log files).
    Attachments {
        #[command(flatten)]
        jira_id: JiraIdArg,
    },
    /// Clean log directory
    ///
    /// Clean log directory for specified JIRA ID, or clean entire base directory if --all is specified.
    Clean {
        #[command(flatten)]
        jira_id: JiraIdArg,

        /// Clean entire base directory (all tickets)
        #[arg(long, short = 'a')]
        all: bool,

        #[command(flatten)]
        dry_run: DryRunArgs,

        /// Only list what would be deleted
        #[arg(long, short = 'l')]
        list: bool,
    },
    /// Log operations (download, find, search)
    ///
    /// Download log files from Jira tickets, search and find content in logs.
    Log {
        #[command(subcommand)]
        subcommand: LogSubcommand,
    },
}
