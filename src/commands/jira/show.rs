use crate::{log_info, log_success, Jira};
use anyhow::{Context, Result};

/// Jira 显示命令
pub struct JiraShow;

impl JiraShow {
    /// 显示 ticket 详细信息
    pub fn show_ticket(ticket: &str) -> Result<()> {
        log_info!("Fetching ticket info: {}", ticket);

        let info = Jira::get_ticket_info(ticket)
            .context(format!("Failed to get ticket info: {}", ticket))?;

        log_success!("Ticket Information");
        log_info!("{}", info);

        Ok(())
    }
}
