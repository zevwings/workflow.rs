use crate::commands::pr::helpers;
use crate::{
    detect_repo_type, log_break, log_error, log_info, log_message, Codeup, Git, GitHub,
    PlatformProvider, RepoType,
};
use anyhow::Result;

/// PR 状态命令
#[allow(dead_code)]
pub struct PullRequestStatusCommand;

#[allow(dead_code)]
impl PullRequestStatusCommand {
    /// 显示 PR 状态信息
    pub fn show(pull_request_id_or_branch: Option<String>) -> Result<()> {
        let repo_type = Git::detect_repo_type()?;

        // 获取 PR ID 或标识符
        let pr_identifier = Self::get_pr_identifier(pull_request_id_or_branch, &repo_type)?;

        // 显示 PR 信息
        Self::show_pr_info(&pr_identifier, &repo_type)?;

        Ok(())
    }

    /// 获取 PR 标识符（从参数或当前分支）
    fn get_pr_identifier(
        pull_request_id_or_branch: Option<String>,
        repo_type: &RepoType,
    ) -> Result<String> {
        if let Some(id) = pull_request_id_or_branch {
            // GitHub 只支持数字 ID，Codeup 支持 ID 或分支名
            match repo_type {
                RepoType::GitHub => {
                    if id.parse::<u32>().is_ok() {
                        Ok(id)
                    } else {
                        anyhow::bail!(
                            "Branch name lookup for GitHub is not yet implemented. Please use PR ID (e.g., #123) or run without arguments to auto-detect from current branch."
                        );
                    }
                }
                RepoType::Codeup => Ok(id),
                _ => Ok(id),
            }
        } else {
            // 从当前分支获取 PR
            helpers::get_current_branch_pull_request(repo_type)
        }
    }

    /// 显示 PR 信息
    fn show_pr_info(pr_identifier: &str, _repo_type: &RepoType) -> Result<()> {
        let info = detect_repo_type(
            |repo_type| match repo_type {
                RepoType::GitHub => GitHub::get_pull_request_info(pr_identifier),
                RepoType::Codeup => Codeup::get_pull_request_info(pr_identifier),
                RepoType::Unknown => {
                    let remote_url =
                        Git::get_remote_url().unwrap_or_else(|_| "unknown".to_string());
                    log_error!("Unsupported repository type detected");
                    log_info!("Remote URL: {}", remote_url);
                    anyhow::bail!(
                        "PR show is currently only supported for GitHub (github.com) and Codeup (codeup.aliyun.com) repositories.\n\
                        Detected remote: {}\n\
                        Please ensure your remote URL contains 'github.com' or 'codeup.aliyun.com'.",
                        remote_url
                    );
                }
            },
            "get pull request info",
        )?;

        log_break!();
        log_break!('=', 40, "PR Information");
        log_message!("{}", info);
        Ok(())
    }
}
