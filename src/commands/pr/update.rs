use crate::{Codeup, Git, GitHub, PlatformProvider, RepoType};
use anyhow::Result;
use crate::{log_success, log_warning};

/// 快速更新命令
#[allow(dead_code)]
pub struct PullRequestUpdateCommand;

impl PullRequestUpdateCommand {
    /// 快速更新代码（使用 PR 标题作为 commit 消息）
    ///
    /// 根据仓库类型自动选择对应的平台实现
    #[allow(dead_code)]
    pub fn update() -> Result<()> {
        // 检测仓库类型
        let repo_type = Git::detect_repo_type()?;

        // 获取当前分支的 PR 标题
        let pull_request_title = Self::get_pull_request_title_for_repo(&repo_type)?;

        // 确定提交消息
        let message = pull_request_title.unwrap_or_else(|| {
            log_warning!("No commit message provided, using default message");
            "update".to_string()
        });

        log_success!("Using commit message: {}", message);

        // 执行 git commit（会自动暂存所有文件）
        log_success!("Staging and committing changes...");
        Git::commit(&message, false)?; // 不使用 --no-verify（commit 方法内部会自动暂存）

        // 执行 git push
        let current_branch = Git::current_branch()?;
        log_success!("Pushing to remote...");
        Git::push(&current_branch, false)?; // 不使用 -u（分支应该已经存在）

        log_success!("Update completed successfully!");
        Ok(())
    }

    /// 根据仓库类型获取当前分支的 PR 标题
    ///
    /// # Arguments
    /// * `repo_type` - 仓库类型（GitHub、Codeup 等）
    ///
    /// # Returns
    /// PR 标题（如果存在），否则返回 None
    #[allow(dead_code)]
    fn get_pull_request_title_for_repo(repo_type: &RepoType) -> Result<Option<String>> {
        use crate::{log_success, log_warning};

        let pull_request_title = match repo_type {
            RepoType::GitHub => {
                // 获取当前分支的 PR
                match <GitHub as PlatformProvider>::get_current_branch_pull_request()? {
                    Some(pull_request_id) => {
                        log_success!("Found PR for current branch: #{}", pull_request_id);
                        Some(<GitHub as PlatformProvider>::get_pull_request_title(
                            &pull_request_id,
                        )?)
                    }
                    None => {
                        log_warning!("No PR found for current branch");
                        None
                    }
                }
            }
            RepoType::Codeup => {
                // 获取当前分支的 PR
                match <Codeup as PlatformProvider>::get_current_branch_pull_request()? {
                    Some(pull_request_id) => {
                        log_success!("Found PR for current branch: #{}", pull_request_id);
                        Some(<Codeup as PlatformProvider>::get_pull_request_title(
                            &pull_request_id,
                        )?)
                    }
                    None => {
                        log_warning!("No PR found for current branch");
                        None
                    }
                }
            }
            RepoType::Unknown => {
                log_warning!("Unknown repository type, cannot get PR title");
                None
            }
        };

        Ok(pull_request_title)
    }
}
