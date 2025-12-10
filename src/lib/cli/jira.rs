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
