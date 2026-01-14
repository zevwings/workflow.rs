use crate::domain::pr::create_provider_auto;
use crate::domain::pr::helpers::get_current_branch_pr_id;
use crate::services::git::{GitBranch, GitCommit, GitPreCommit};
use crate::spinner;
use crate::{br, info, success, warning};
use color_eyre::Result;

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
            warning!("No commit message provided, using default message");
            "update".to_string()
        });

        success!("Using commit message: {}", message);

        // 先执行 pre-commit 检查（如果有），避免与 Spinner 输出冲突
        if GitPreCommit::has_pre_commit() {
            GitPreCommit::run_checks()?;
        }

        // 执行 git commit（会自动暂存所有文件）
        // 使用 --no-verify 跳过 hook，因为我们已经通过 Rust 代码执行了检查
        GitCommit::commit(&message, true)?; // 使用 --no-verify，因为已经执行了检查

        // 执行 git push
        let current_branch = GitBranch::current_branch()?;
        br!();
        info!("Pushing to remote...");
        br!();
        GitBranch::push(&current_branch, false)?; // 不使用 -u（分支应该已经存在）

        br!();
        success!("Update completed successfully!");
        Ok(())
    }

    /// 根据仓库类型获取当前分支的 PR 标题
    fn get_pull_request_title() -> Result<Option<String>> {
        // 获取当前分支的 PR ID（如果不存在，返回 None 而不是错误）
        let pr_id = match get_current_branch_pr_id() {
            Ok(Some(id)) => id,
            Ok(None) | Err(_) => {
                warning!("No PR found for current branch");
                return Ok(None);
            }
        };

        // 获取 PR 标题
        let provider = create_provider_auto()?;
        let title = spinner!("Fetching PR #{} title...", pr_id)
            .with(|| provider.get_pull_request_title(&pr_id))
            .ok();

        Ok(title)
    }
}
