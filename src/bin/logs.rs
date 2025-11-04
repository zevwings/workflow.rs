use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use workflow::{log_error, log_info, log_success, log_warning, Clipboard, Logs, Settings};

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
            let settings = Settings::get();
            let log_output_folder_name = if !settings.log_output_folder_name.is_empty() {
                Some(settings.log_output_folder_name.as_str())
            } else {
                None
            };

            log_success!("Getting attachments for {}...", jira_id);

            let base_dir = Logs::download_from_jira(&jira_id, log_output_folder_name)
                .context("Failed to download logs from Jira")?;

            log_success!("\nDownload completed!");
            log_info!("Files located at: {:?}", base_dir);
        }
        LogsCommands::Find {
            log_file,
            request_id,
            jira_id,
        } => {
            log_success!("Searching for request ID: {}...", request_id);

            let settings = Settings::get();
            let jira_service_address = Some(settings.jira_service_address.as_str());

            let response_content = Logs::find_and_send_to_streamock(
                std::path::Path::new(&log_file),
                &request_id,
                jira_id.as_deref(),
                jira_service_address,
                None, // 使用默认的 Streamock URL
            )
            .map_err(|e| {
                log_error!("Failed to process request: {}", e);
                e
            })?;

            // 复制到剪贴板（CLI特定操作）
            Clipboard::copy(&response_content).context("Failed to copy to clipboard")?;
            log_success!("Response content copied to clipboard");
            log_success!("Response sent to Streamock successfully");
        }
        LogsCommands::Search {
            log_file,
            search_term,
        } => {
            log_success!("Searching for: '{}'...", search_term);

            let results = Logs::search_keyword(std::path::Path::new(&log_file), &search_term)
                .context("Failed to search log file")?;

            if results.is_empty() {
                log_warning!("No matches found for '{}'", search_term);
                return Ok(());
            }

            log_success!("\nFound {} matches:\n", results.len());

            for entry in results {
                if let Some(id) = entry.id {
                    if let Some(url) = entry.url {
                        log_info!("URL: {}, ID: {}", url, id);
                    } else {
                        log_info!("ID: {} (URL not found)", id);
                    }
                }
            }
        }
    }

    Ok(())
}

