use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::logs::{download as logs_download, find as logs_find, search as logs_search};

#[derive(Parser)]
#[command(name = "logs")]
#[command(about = "Log operations", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    subcommand: LogsCommands,
}

#[derive(Subcommand)]
enum LogsCommands {
    /// Download log files from Jira ticket
    Download {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "JIRA_ID")]
        jira_id: String,
    },
    /// Find request ID in log file and extract response
    Find {
        /// Log file path
        #[arg(value_name = "LOG_FILE")]
        log_file: String,

        /// Request ID to search for
        #[arg(value_name = "REQUEST_ID")]
        request_id: String,

        /// Jira ticket ID (optional, for domain)
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,
    },
    /// Search for keyword in log file
    Search {
        /// Log file path
        #[arg(value_name = "LOG_FILE")]
        log_file: String,

        /// Search term
        #[arg(value_name = "SEARCH_TERM")]
        search_term: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        LogsCommands::Download { jira_id } => {
            logs_download::LogsDownloadCommand::download(&jira_id)?;
        }
        LogsCommands::Find {
            log_file,
            request_id,
            jira_id,
        } => {
            logs_find::LogsFindCommand::find(
                std::path::Path::new(&log_file),
                &request_id,
                jira_id.as_deref(),
            )?;
        }
        LogsCommands::Search {
            log_file,
            search_term,
        } => {
            logs_search::LogsSearchCommand::search(std::path::Path::new(&log_file), &search_term)?;
        }
    }

    Ok(())
}

