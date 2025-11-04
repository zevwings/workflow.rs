use crate::{log_info, log_success, Codeup, Git, GitHub, Platform, RepoType};
use anyhow::Result;

/// PR 列表命令
pub struct PRListCommand;

impl PRListCommand {
    /// 列出 PR
    pub fn list(state: Option<String>, limit: Option<u32>) -> Result<()> {
        let repo_type = Git::detect_repo_type()?;

        match repo_type {
            RepoType::GitHub => {
                log_success!("PR List");
                let output = <GitHub as Platform>::list_prs(state.as_deref(), limit)?;
                log_info!("{}", output);
            }
            RepoType::Codeup => {
                log_success!("PR List");
                let output = <Codeup as Platform>::list_prs(state.as_deref(), limit)?;
                log_info!("{}", output);
            }
            _ => {
                anyhow::bail!("PR list is currently only supported for GitHub and Codeup repositories.");
            }
        }

        Ok(())
    }
}
