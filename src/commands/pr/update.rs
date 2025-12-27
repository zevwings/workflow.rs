use crate::base::indicator::Spinner;
use crate::git::{GitBranch, GitCommit, GitPreCommit};
use crate::pr::create_provider_auto;
use crate::pr::helpers::get_current_branch_pr_id;
use crate::{log_success, log_warning};
use color_eyre::Result;

/// 快速更新命令
pub struct PullRequestUpdateCommand;

impl PullRequestUpdateCommand {
    /// 快速更新代码（使用 PR 标题作为 commit 消息）
    ///
    /// 根据仓库类型自动选择对应的平台实现
    pub fn update() -> Result<()> {
        let pull_request_title = Self::get_pull_request_title()?;

        let message = pull_request_title.unwrap_or_else(|| {
            log_warning!("No commit message provided, using default message");
            "update".to_string()
        });

        log_success!("Using commit message: {}", message);

        // 先执行 pre-commit 检查（如果有），避免与 Spinner 输出冲突
        if GitPreCommit::has_pre_commit() {
            GitPreCommit::run_checks()?;
        }

        // 使用 --no-verify 跳过 hook，因为我们已经通过 Rust 代码执行了检查
        GitCommit::commit(&message, true)?;

        let current_branch = GitBranch::current_branch()?;
        // 不使用 -u（分支应该已经存在）
        Spinner::with_output("Pushing to remote...", || {
            GitBranch::push(&current_branch, false)
        })?;

        log_success!("Update completed successfully!");
        Ok(())
    }

    /// 根据仓库类型获取当前分支的 PR 标题
    fn get_pull_request_title() -> Result<Option<String>> {
        let pr_id = match get_current_branch_pr_id() {
            Ok(Some(id)) => id,
            Ok(None) | Err(_) => {
                log_warning!("No PR found for current branch");
                return Ok(None);
            }
        };

        let provider = create_provider_auto()?;
        let title = Spinner::with(format!("Fetching PR #{} title...", pr_id), || {
            provider.get_pull_request_title(&pr_id)
        })
        .ok();

        Ok(title)
    }
}
