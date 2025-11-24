use crate::pr::create_provider;
use crate::{log_break, log_info};
use anyhow::Result;

/// PR 列表命令
#[allow(dead_code)]
pub struct PullRequestListCommand;

#[allow(dead_code)]
impl PullRequestListCommand {
    /// 列出 PR
    pub fn list(state: Option<String>, limit: Option<u32>) -> Result<()> {
        log_break!('=', 40, "PR List");
        let provider = create_provider()?;
        let output = provider.get_pull_requests(state.as_deref(), limit)?;
        log_info!("{}", output);

        Ok(())
    }
}
