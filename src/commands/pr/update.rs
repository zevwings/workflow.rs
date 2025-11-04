use crate::{Codeup, Git, GitHub, Platform, RepoType};
use anyhow::Result;

/// 快速更新命令
pub struct PRUpdateCommand;

impl PRUpdateCommand {
    /// 快速更新代码（使用 PR 标题作为 commit 消息）
    ///
    /// 根据仓库类型自动选择对应的平台实现
    pub fn update() -> Result<()> {
        // 检测仓库类型
        let repo_type = Git::detect_repo_type()?;

        // 获取当前分支的 PR 标题
        let pr_title = Self::get_pr_title_for_repo(&repo_type)?;

        // 执行 Git 更新操作
        Git::update(pr_title)
    }

    /// 根据仓库类型获取当前分支的 PR 标题
    ///
    /// # Arguments
    /// * `repo_type` - 仓库类型（GitHub、Codeup 等）
    ///
    /// # Returns
    /// PR 标题（如果存在），否则返回 None
    fn get_pr_title_for_repo(repo_type: &RepoType) -> Result<Option<String>> {
        use crate::{log_success, log_warning};

        let pr_title = match repo_type {
            RepoType::GitHub => {
                // 获取当前分支的 PR
                match <GitHub as Platform>::get_current_branch_pr()? {
                    Some(pr_id) => {
                        log_success!("Found PR for current branch: #{}", pr_id);
                        Some(<GitHub as Platform>::get_pr_title(&pr_id)?)
                    }
                    None => {
                        log_warning!("No PR found for current branch");
                        None
                    }
                }
            }
            RepoType::Codeup => {
                // 获取当前分支的 PR
                match <Codeup as Platform>::get_current_branch_pr()? {
                    Some(pr_id) => {
                        log_success!("Found PR for current branch: #{}", pr_id);
                        Some(<Codeup as Platform>::get_pr_title(&pr_id)?)
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

        Ok(pr_title)
    }
}
