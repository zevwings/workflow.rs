//! Log operations subcommands

use clap::Subcommand;

use super::args::JiraIdArg;

/// Log operations subcommands
///
/// Used to manage log file operations.
#[derive(Subcommand)]
pub enum LogSubcommand {
    /// Download log files from Jira ticket
    ///
    /// Download log files from Jira ticket attachments (supports automatic merging of split files).
    /// Log files will be saved locally with paths automatically resolved based on JIRA ID.
    Download {
        #[command(flatten)]
        jira_id: JiraIdArg,
    },
    /// Find request ID in log files
    ///
    /// Find specified request ID in log files and extract corresponding response content.
    /// If found, will copy response content to clipboard and automatically open browser.
    Find {
        #[command(flatten)]
        jira_id: JiraIdArg,

        /// Request ID (optional, will prompt interactively if not provided)
        #[arg(value_name = "REQUEST_ID")]
        request_id: Option<String>,
    },
    /// Search for keywords in log files
    ///
    /// Search for specified keywords in log files and return all matching request information.
    Search {
        #[command(flatten)]
        jira_id: JiraIdArg,

        /// Search keyword (optional, will prompt interactively if not provided)
        #[arg(value_name = "SEARCH_TERM")]
        search_term: Option<String>,
    },
}
