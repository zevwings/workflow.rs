use anyhow::Result;
use clap::{Parser, Subcommand};
use workflow::commands::jira::{show as jira_show, status as jira_status};

#[derive(Parser)]
#[command(name = "jira")]
#[command(about = "Jira operations", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    subcommand: JiraCommands,
}

#[derive(Subcommand)]
enum JiraCommands {
    /// Configure Jira status mapping (interactive)
    Status {
        /// Jira project or ticket (e.g., PROJ or PROJ-123)
        #[arg(value_name = "PROJECT_OR_TICKET")]
        project_or_ticket: String,
    },
    /// Show ticket information
    Show {
        /// Jira ticket ID (e.g., PROJ-123)
        #[arg(value_name = "TICKET")]
        ticket: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        JiraCommands::Status { project_or_ticket } => {
            jira_status::JiraStatus::configure_interactive(&project_or_ticket)?;
        }
        JiraCommands::Show { ticket } => {
            jira_show::JiraShow::show_ticket(&ticket)?;
        }
    }

    Ok(())
}

