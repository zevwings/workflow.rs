use crate::git::{GitBranch, GitCommit};
use crate::pr::create_provider;
use crate::pr::helpers::get_current_branch_pr_id;
use crate::{log_success, log_warning};
use anyhow::Result;

/// 快速更新命令
#[allow(dead_code)]
pub struct PullRequestUpdateCommand;

#[allow(dead_code)]
impl PullRequestUpdateCommand {
    /// 快速更新代码（使用 PR 标题作为 commit 消息）
    ///
    /// 根据仓库类型自动选择对应的平台实现
    pub fn update() -> Result<()> {
        // 获取当前分支的 PR 标题
        let pull_request_title = Self::get_pull_request_title()?;

        // 确定提交消息
        let message = pull_request_title.unwrap_or_else(|| {
            log_warning!("No commit message provided, using default message");
            "update".to_string()
        });

        log_success!("Using commit message: {}", message);

        // 执行 git commit（会自动暂存所有文件）
        log_success!("Staging and committing changes...");
        GitCommit::commit(&message, false)?; // 不使用 --no-verify（commit 方法内部会自动暂存）

        // 执行 git push
        let current_branch = GitBranch::current_branch()?;
        log_success!("Pushing to remote...");
        GitBranch::push(&current_branch, false)?; // 不使用 -u（分支应该已经存在）

        log_success!("Update completed successfully!");
        Ok(())
    }

    /// 根据仓库类型获取当前分支的 PR 标题
    fn get_pull_request_title() -> Result<Option<String>> {
        // 获取当前分支的 PR ID（如果不存在，返回 None 而不是错误）
        let pr_id = match get_current_branch_pr_id() {
            Ok(Some(id)) => id,
            Ok(None) | Err(_) => {
                log_warning!("No PR found for current branch");
                return Ok(None);
            }
        };

        // 获取 PR 标题
        let provider = create_provider()?;
        let title = provider.get_pull_request_title(&pr_id).ok();

        Ok(title)
    }
}
