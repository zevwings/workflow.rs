//! Jira operations subcommands

use clap::Subcommand;

/// Jira operations subcommands
///
/// Used to manage Jira ticket operations.
#[derive(Subcommand)]
pub enum JiraSubcommand {
    /// Show ticket information
    ///
    /// Display detailed information about a Jira ticket.
    Info {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Output in table format (default)
        #[arg(long)]
        table: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in YAML format
        #[arg(long)]
        yaml: bool,

        /// Output in Markdown format
        #[arg(long)]
        markdown: bool,
    },
    /// Show changelog (change history) for a Jira ticket
    ///
    /// Display change history for a Jira ticket with filtering options.
    Changelog {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Filter changelog by specific field
        #[arg(long, value_name = "FIELD")]
        field: Option<String>,

        /// Output in table format (default)
        #[arg(long)]
        table: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in YAML format
        #[arg(long)]
        yaml: bool,

        /// Output in Markdown format
        #[arg(long)]
        markdown: bool,
    },
    /// Show comments for a Jira ticket
    ///
    /// Display all comments for a Jira ticket with filtering and pagination options.
    Comments {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

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

        /// Output in table format (default)
        #[arg(long)]
        table: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in YAML format
        #[arg(long)]
        yaml: bool,

        /// Output in Markdown format
        #[arg(long)]
        markdown: bool,
    },
    /// Download all attachments from Jira ticket
    ///
    /// Download all attachments from Jira ticket (not just log files).
    Attachments {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,
    },
    /// Clean log directory
    ///
    /// Clean log directory for specified JIRA ID, or clean entire base directory if --all is specified.
    Clean {
        /// Jira ticket ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        /// Clean entire base directory (all tickets)
        #[arg(long, short = 'a')]
        all: bool,

        /// Preview operation without actually deleting
        #[arg(long, short = 'n')]
        dry_run: bool,

        /// Only list what would be deleted
        #[arg(long, short = 'l')]
        list: bool,
    },
}
