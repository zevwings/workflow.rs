use crate::{detect_repo_type, log_break, log_info, Codeup, GitHub, PlatformProvider, RepoType};
use anyhow::Result;

/// PR 列表命令
#[allow(dead_code)]
pub struct GetPullRequestsCommand;

#[allow(dead_code)]
impl GetPullRequestsCommand {
    /// 列出 PR
    pub fn list(state: Option<String>, limit: Option<u32>) -> Result<()> {
        log_break!('=', 40, "PR List");
        let output = detect_repo_type(
            |repo_type| match repo_type {
                RepoType::GitHub => GitHub::get_pull_requests(state.as_deref(), limit),
                RepoType::Codeup => Codeup::get_pull_requests(state.as_deref(), limit),
                RepoType::Unknown => {
                    anyhow::bail!(
                        "PR list is currently only supported for GitHub and Codeup repositories."
                    );
                }
            },
            "get pull requests",
        )?;
        log_info!("{}", output);

        Ok(())
    }
}
