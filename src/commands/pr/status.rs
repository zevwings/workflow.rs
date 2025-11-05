use crate::{log_error, log_info, log_success, Codeup, Git, GitHub, PlatformProvider, RepoType};
use anyhow::Result;

/// PR 状态命令
#[allow(dead_code)]
pub struct PullRequestStatusCommand;

impl PullRequestStatusCommand {
    /// 显示 PR 状态信息
    #[allow(dead_code)]
    pub fn show(pull_request_id_or_branch: Option<String>) -> Result<()> {
        let repo_type = Git::detect_repo_type()?;

        match repo_type {
            RepoType::GitHub => {
                let pull_request_id = if let Some(id) = pull_request_id_or_branch {
                    // 如果是数字，直接使用；否则可能是分支名，需要通过分支获取 PR
                    if id.parse::<u32>().is_ok() {
                        id
                    } else {
                        // 假设是分支名，尝试通过 gh CLI 查找该分支的 PR
                        // GitHub CLI 没有直接通过分支名查找 PR 的命令，但我们可以在输出中查找
                        // 或者提示用户使用 PR ID
                        anyhow::bail!("Branch name lookup for GitHub is not yet implemented. Please use PR ID (e.g., #123) or run without arguments to auto-detect from current branch.");
                    }
                } else {
                    // 从当前分支获取 PR
                    match <GitHub as PlatformProvider>::get_current_branch_pull_request()? {
                        Some(id) => {
                            log_success!("Found PR for current branch: #{}", id);
                            id
                        }
                        None => {
                            anyhow::bail!("No PR found for current branch. Please specify PR ID.");
                        }
                    }
                };

                log_success!("\nPR Information:");
                let info = <GitHub as PlatformProvider>::get_pull_request_info(&pull_request_id)?;
                log_info!("{}", info);
            }
            RepoType::Codeup => {
                let pull_request_id_or_branch = if let Some(id) = pull_request_id_or_branch {
                    id
                } else {
                    // 从当前分支获取 PR
                    match <Codeup as PlatformProvider>::get_current_branch_pull_request()? {
                        Some(id) => {
                            log_success!("Found PR for current branch: #{}", id);
                            id
                        }
                        None => {
                            anyhow::bail!("No PR found for current branch. Please specify PR ID or branch name.");
                        }
                    }
                };

                log_success!("\nPR Information:");
                let info = <Codeup as PlatformProvider>::get_pull_request_info(
                    &pull_request_id_or_branch,
                )?;
                log_info!("{}", info);
            }
            RepoType::Unknown => {
                let remote_url = Git::get_remote_url().unwrap_or_else(|_| "unknown".to_string());
                log_error!("Unsupported repository type detected");
                log_info!("Remote URL: {}", remote_url);
                anyhow::bail!(
                    "PR show is currently only supported for GitHub (github.com) and Codeup (codeup.aliyun.com) repositories.\n\
                    Detected remote: {}\n\
                    Please ensure your remote URL contains 'github.com' or 'codeup.aliyun.com'.",
                    remote_url
                );
            }
        }

        Ok(())
    }
}
