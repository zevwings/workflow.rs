//! 提交本地更改并推送到 PR 命令
//!
//! 该命令获取当前分支关联的 PR 标题作为 commit message，
//! 提交本地更改并推送到远端。

use domain::GitRepository;
use prompt::{info, spinner, success, warning};

use crate::registry;

/// Pull Request Update 命令
///
/// 获取 PR 标题作为 commit message，提交本地更改并推送到远端
pub struct PullRequestUpdateCommand {
    pr_id: Option<String>,
    message: Option<String>,
}

impl PullRequestUpdateCommand {
    /// 创建新的 PullRequestUpdateCommand
    ///
    /// # 参数
    /// * `pr_id` - PR ID（可选，不提供时自动检测当前分支的 PR）
    /// * `message` - 自定义 commit message（可选，不提供时使用 PR 标题）
    pub fn new(pr_id: Option<String>, message: Option<String>) -> Self {
        Self { pr_id, message }
    }

    /// 运行 `workflow pr update` 命令
    ///
    /// 工作流程：
    /// 1. 获取当前分支关联的 PR（如果没有提供 PR ID）
    /// 2. 获取 PR 标题作为 commit message（如果没有提供自定义 message）
    /// 3. 提交本地更改
    /// 4. 推送到远端
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = registry::get_pull_request_service();
        let git_repo = registry::get_git_repository();

        // 1. 获取 PR 状态（包含 PR ID 和标题）
        let pr_status = spinner!("Fetching PR information...")
            .with(|| pr_service.get_pr_status(self.pr_id.as_deref()))
            .map_err(|e| format!("Failed to get PR status: {}", e))?;

        info!("Found PR #{}: {}", pr_status.id, pr_status.title);

        // 2. 确定 commit message
        let commit_message = self.message.clone().unwrap_or_else(|| pr_status.title.clone());

        // 3. 检查是否有更改需要提交
        let status = git_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to get working tree status: {}", e))?;

        if status.is_clean() {
            warning!("No changes to commit, skipping commit step");
        } else {
            // 提交更改
            self.commit_changes(&*git_repo, &commit_message)?;
        }

        // 4. 推送到远端
        self.push_branch(&*git_repo)?;

        success!(
            "Successfully updated PR #{} with commit: {}",
            pr_status.id,
            commit_message
        );

        Ok(())
    }

    /// 提交本地更改
    fn commit_changes(
        &self,
        git_repo: &dyn GitRepository,
        commit_message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Committing changes with message: {}", commit_message);

        let commit_sha = spinner!("Committing changes...")
            .with(|| git_repo.commit(commit_message, true))
            .map_err(|e| {
                let err_msg = e.to_string();
                if err_msg.contains("nothing to commit") {
                    return "No changes to commit".into();
                }
                format!("Failed to commit changes: {}", e)
            })?;

        success!(
            "Committed changes: {}",
            &commit_sha[..7.min(commit_sha.len())]
        );

        Ok(())
    }

    /// 推送分支到远端
    fn push_branch(&self, git_repo: &dyn GitRepository) -> Result<(), Box<dyn std::error::Error>> {
        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        info!("Pushing branch '{}' to remote...", current_branch);

        spinner!("Pushing to remote...")
            .with(|| git_repo.push(&current_branch, false))
            .map_err(|e| format!("Failed to push branch: {}", e))?;

        success!("Pushed branch '{}' to remote", current_branch);

        Ok(())
    }
}
