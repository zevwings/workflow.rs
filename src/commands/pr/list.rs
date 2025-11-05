use crate::{log_info, log_success, Codeup, Git, GitHub, PlatformProvider, RepoType};
use anyhow::Result;

/// PR 列表命令
#[allow(dead_code)]
pub struct GetPullRequestsCommand;

impl GetPullRequestsCommand {
    /// 列出 PR
    #[allow(dead_code)]
    pub fn list(state: Option<String>, limit: Option<u32>) -> Result<()> {
        let repo_type = Git::detect_repo_type()?;

        match repo_type {
            RepoType::GitHub => {
                log_success!("PR List");
                let output =
                    <GitHub as PlatformProvider>::get_pull_requests(state.as_deref(), limit)?;
                log_info!("{}", output);
            }
            RepoType::Codeup => {
                log_success!("PR List");
                let output =
                    <Codeup as PlatformProvider>::get_pull_requests(state.as_deref(), limit)?;
                log_info!("{}", output);
            }
            _ => {
                anyhow::bail!(
                    "PR list is currently only supported for GitHub and Codeup repositories."
                );
            }
        }

        Ok(())
    }
}
