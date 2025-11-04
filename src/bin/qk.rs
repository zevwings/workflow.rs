use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::qk::QuickCommand;

#[derive(Parser)]
#[command(name = "qk")]
#[command(about = "Quick log operations (unified wrapper)", long_about = None)]
#[command(version)]
struct Cli {
    /// Jira ticket ID (e.g., PROJ-123)
    #[arg(value_name = "JIRA_ID")]
    jira_id: String,

    #[command(subcommand)]
    subcommand: QkCommands,
}

#[derive(Subcommand)]
enum QkCommands {
    /// Download logs (equivalent to qk <JIRA-ID> -d)
    Download,
    /// Find request by ID (equivalent to qk <JIRA-ID> -f [REQUEST_ID])
    Find {
        /// Request ID (optional, will prompt if not provided)
        #[arg(value_name = "REQUEST_ID")]
        request_id: Option<String>,
    },
    /// Search in logs (equivalent to qk <JIRA-ID> -s [SEARCH_TERM])
    Search {
        /// Search term (optional, will prompt if not provided)
        #[arg(value_name = "SEARCH_TERM")]
        search_term: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        QkCommands::Download => {
            QuickCommand::download(&cli.jira_id)?;
        }
        QkCommands::Find { request_id } => {
            QuickCommand::find_request_id(&cli.jira_id, request_id)?;
        }
        QkCommands::Search { search_term } => {
            QuickCommand::search(&cli.jira_id, search_term)?;
        }
    }

    Ok(())
}
