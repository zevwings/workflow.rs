use crate::{log_info, log_success, Codeup, Git, GitHub, PlatformProvider, RepoType};
use anyhow::Result;

/// PR 列表命令
#[allow(dead_code)]
pub struct GetPullRequestsCommand;

#[allow(dead_code)]
impl GetPullRequestsCommand {
    /// 列出 PR
    pub fn list(state: Option<String>, limit: Option<u32>) -> Result<()> {
        let repo_type = Git::detect_repo_type()?;

        log_success!("PR List");
        let output = Self::get_pull_requests(state.as_deref(), limit, &repo_type)?;
        log_info!("{}", output);

        Ok(())
    }

    /// 根据仓库类型获取 PR 列表
    fn get_pull_requests(
        state: Option<&str>,
        limit: Option<u32>,
        repo_type: &RepoType,
    ) -> Result<String> {
        match repo_type {
            RepoType::GitHub => <GitHub as PlatformProvider>::get_pull_requests(state, limit),
            RepoType::Codeup => <Codeup as PlatformProvider>::get_pull_requests(state, limit),
            _ => {
                anyhow::bail!(
                    "PR list is currently only supported for GitHub and Codeup repositories."
                );
            }
        }
    }
}
