use crate::commands::config::check;
use crate::pr::create_provider;
use crate::{
    confirm, get_current_branch_pr_id, log_debug, log_error, log_info, log_success, log_warning,
    GitBranch, GitCommit, GitRepo, GitStash,
};
use anyhow::{Context, Result};

/// PR 分支集成的命令
#[allow(dead_code)]
pub struct PullRequestIntegrateCommand;

/// 源分支信息
struct SourceBranchInfo {
    /// 分支类型：true 表示远程分支，false 表示本地分支
    is_remote: bool,
    /// 用于合并的分支引用（本地分支名或 origin/branch-name）
    merge_ref: String,
}

impl PullRequestIntegrateCommand {
    /// 将指定分支合并到当前分支并推送到远程
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要合并的源分支名称
    /// * `ff_only` - 是否只允许 fast-forward 合并
    /// * `squash` - 是否使用 squash 合并
    /// * `push` - 是否在合并后自动推送
    #[allow(dead_code)]
    pub fn integrate(source_branch: String, ff_only: bool, squash: bool, push: bool) -> Result<()> {
        // 1. 运行检查（可选，但建议运行）
        log_info!("Running pre-flight checks...");
        if let Err(e) = check::CheckCommand::run_all() {
            log_warning!("Pre-flight checks failed: {}", e);
            confirm(
                "Continue anyway?",
                false,
                Some("Operation cancelled by user"),
            )?;
        }

        // 2. 获取当前分支
        let current_branch = GitBranch::current_branch()?;
        log_success!("Current branch: {}", current_branch);

        // 3. 检查工作区状态并 stash（如果需要）
        let has_stashed = Self::check_working_directory()?;

        // 4. 验证并准备源分支，获取分支信息
        let source_branch_info = Self::prepare_source_branch(&source_branch)?;

        // 5. 确定合并策略
        let strategy = Self::determine_merge_strategy(ff_only, squash);

        // 6. 执行合并
        log_success!("Merging '{}' into '{}'...", source_branch, current_branch);
        let merge_result = GitBranch::merge_branch(&source_branch_info.merge_ref, strategy);

        // 如果合并失败，恢复 stash（如果有）
        if merge_result.is_err() && has_stashed {
            log_info!("Merge failed, attempting to restore stashed changes...");
            let _ = GitStash::stash_pop(); // 日志已在 stash_pop 中处理，忽略错误继续执行
        }

        match merge_result {
            Ok(()) => {
                log_success!("Merge completed successfully");
            }
            Err(e) => {
                // 检查是否是合并冲突
                if let Ok(has_conflicts) = GitBranch::has_merge_conflicts() {
                    if has_conflicts {
                        log_error!("Merge conflicts detected!");
                        log_info!("Please resolve the conflicts manually:");
                        log_info!("  1. Review conflicted files");
                        log_info!("  2. Resolve conflicts");
                        log_info!("  3. Stage resolved files: git add <files>");
                        log_info!("  4. Complete the merge: git commit");
                        log_info!("  5. Push when ready: git push");
                        anyhow::bail!("Merge conflicts detected. Please resolve manually.");
                    }
                }
                return Err(e);
            }
        }

        // 7. 根据分支类型处理合并后的操作
        if source_branch_info.is_remote {
            // 远程分支：检查当前分支是否已创建 PR
            let pr_updated = Self::check_and_update_current_branch_pr(&current_branch)?;
            if pr_updated {
                // 已更新 PR，不需要 stash pop（代码已推送）
                log_info!("PR updated, skipping stash pop");
            } else {
                // 未创建 PR，恢复 stash（如果有）
                if has_stashed {
                    log_info!("Restoring stashed changes...");
                    let _ = GitStash::stash_pop(); // 日志已在 stash_pop 中处理，忽略错误继续执行
                }
            }
        } else {
            // 本地分支：合并后立即恢复 stash（如果有）
            if has_stashed {
                log_info!("Restoring stashed changes...");
                let _ = GitStash::stash_pop(); // 日志已在 stash_pop 中处理
            }

            // 推送到远程（如果需要）
            if push {
                log_success!("Pushing to remote...");
                let exists_remote = GitBranch::has_remote_branch(&current_branch)
                    .context("Failed to check if branch exists on remote")?;

                GitBranch::push(&current_branch, !exists_remote)
                    .context("Failed to push to remote")?;
                log_success!("Pushed to remote successfully");
            } else {
                log_info!("Skipping push (--no-push flag set)");
                log_info!("You can push manually with: git push");
            }
        }

        // 8. 检查并关闭被合并分支的 PR（如果存在）
        Self::check_and_close_source_branch_pr(&source_branch)?;

        // 9. 删除被合并的源分支（如果存在且不是当前分支）
        Self::delete_merged_branch(&source_branch, &current_branch)?;

        log_success!("Integration completed successfully!");
        Ok(())
    }

    /// 检查工作区状态
    ///
    /// 检查是否有未提交的更改，如果有则提示用户处理。
    /// 返回是否执行了 stash 操作。
    fn check_working_directory() -> Result<bool> {
        let has_uncommitted =
            GitCommit::has_commit().context("Failed to check working directory status")?;

        if has_uncommitted {
            log_warning!("Working directory has uncommitted changes");
            let choice = dialoguer::Select::new()
                .with_prompt("How would you like to proceed?")
                .items(&["Stash changes and continue", "Abort operation"])
                .default(0)
                .interact()
                .context("Failed to get user choice")?;

            match choice {
                0 => {
                    log_info!("Stashing uncommitted changes...");
                    GitStash::stash_push(Some("Auto-stash before branch integration"))?;
                    log_success!("Changes stashed successfully");
                    Ok(true)
                }
                1 => {
                    anyhow::bail!("Operation cancelled by user");
                }
                _ => unreachable!(),
            }
        } else {
            Ok(false)
        }
    }

    /// 验证并准备源分支
    ///
    /// 检查源分支是否存在（本地或远程），如果只在远程则先 fetch。
    /// 检查是否为默认分支，如果是则报错。
    /// 返回源分支信息（类型、是否默认分支、合并引用）。
    fn prepare_source_branch(source_branch: &str) -> Result<SourceBranchInfo> {
        // 检查是否为默认分支
        let default_branch = GitBranch::get_default_branch()?;
        let is_default = source_branch == default_branch;

        if is_default {
            anyhow::bail!(
                "Cannot integrate default branch '{}' into current branch. This operation is not allowed.",
                default_branch
            );
        }

        let (exists_local, exists_remote) = GitBranch::is_branch_exists(source_branch)
            .context("Failed to check if source branch exists")?;

        if !exists_local && !exists_remote {
            anyhow::bail!(
                "Source branch '{}' does not exist locally or remotely",
                source_branch
            );
        }

        if !exists_local && exists_remote {
            // 分支只在远程，需要先 fetch 以确保有最新的引用
            log_info!("Source branch '{}' only exists on remote", source_branch);
            log_info!("Fetching from remote...");
            GitRepo::fetch().context("Failed to fetch from remote")?;
            log_success!("Fetched latest changes from remote");
            // 返回远程分支引用
            Ok(SourceBranchInfo {
                is_remote: true,
                merge_ref: format!("origin/{}", source_branch),
            })
        } else {
            // 分支在本地存在，直接使用分支名
            log_success!("Source branch '{}' is ready", source_branch);
            Ok(SourceBranchInfo {
                is_remote: false,
                merge_ref: source_branch.to_string(),
            })
        }
    }

    /// 确定合并策略
    ///
    /// 根据命令行参数确定使用的合并策略。
    fn determine_merge_strategy(ff_only: bool, squash: bool) -> crate::MergeStrategy {
        if ff_only {
            crate::MergeStrategy::FastForwardOnly
        } else if squash {
            crate::MergeStrategy::Squash
        } else {
            crate::MergeStrategy::Merge
        }
    }

    /// 检查并更新当前分支的 PR
    ///
    /// 如果当前分支已创建 PR，则推送代码以更新 PR。
    /// 返回是否更新了 PR。
    fn check_and_update_current_branch_pr(current_branch: &str) -> Result<bool> {
        // 检查当前分支是否已创建 PR
        let current_pr_id = get_current_branch_pr_id()?;

        if let Some(pr_id) = current_pr_id {
            log_info!("Current branch '{}' has PR #{}", current_branch, pr_id);
            log_info!("Pushing changes to update PR...");

            // 检查当前分支是否在远程存在
            let exists_remote = GitBranch::has_remote_branch(current_branch)
                .context("Failed to check if current branch exists on remote")?;

            // 推送代码以更新 PR
            GitBranch::push(current_branch, !exists_remote)
                .context("Failed to push to remote to update PR")?;
            log_success!("PR #{} updated successfully", pr_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 检查并关闭被合并分支的 PR
    ///
    /// 如果被合并分支已创建 PR，则关闭它。
    /// 返回是否关闭了 PR。
    fn check_and_close_source_branch_pr(source_branch: &str) -> Result<bool> {
        // 由于通过分支名查找 PR 比较复杂，我们采用更简单的方法：
        // 尝试通过工作历史查找 PR ID
        let remote_url = GitRepo::get_remote_url().ok();
        if let Some(pr_id) = crate::jira::history::JiraWorkHistory::find_pr_id_by_branch(
            source_branch,
            remote_url.as_deref(),
        )? {
            log_debug!("Found PR #{} for source branch '{}'", pr_id, source_branch);
            log_info!("Closing PR #{}...", pr_id);

            let provider = create_provider()?;
            match provider.close_pull_request(&pr_id) {
                Ok(()) => {
                    log_success!("PR #{} closed successfully", pr_id);
                    Ok(true)
                }
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    /// 删除被合并的源分支
    ///
    /// 在合并成功后，删除源分支（本地和远程）。
    /// 如果源分支是当前分支，则跳过删除。
    fn delete_merged_branch(source_branch: &str, current_branch: &str) -> Result<()> {
        // 不能删除当前分支
        if source_branch == current_branch {
            log_info!(
                "Source branch '{}' is the current branch, skipping deletion",
                source_branch
            );
            return Ok(());
        }

        // 检查源分支是否存在
        let (exists_local, exists_remote) = GitBranch::is_branch_exists(source_branch)
            .context("Failed to check if source branch exists")?;

        if !exists_local && !exists_remote {
            log_info!(
                "Source branch '{}' does not exist, nothing to delete",
                source_branch
            );
            return Ok(());
        }

        // 删除本地分支
        // 由于合并已经成功，先尝试普通删除，失败则强制删除
        if exists_local {
            log_info!("Deleting local branch '{}'...", source_branch);
            GitBranch::delete(source_branch, false)
                .or_else(|_| {
                    // 如果普通删除失败（可能因为分支未完全合并），尝试强制删除
                    log_info!("Branch may not be fully merged, attempting force delete...");
                    GitBranch::delete(source_branch, true)
                })
                .with_context(|| format!("Failed to delete local branch: {}", source_branch))?;
            log_success!("Local branch '{}' deleted successfully", source_branch);
        }

        // 删除远程分支
        if exists_remote {
            log_info!("Deleting remote branch '{}'...", source_branch);
            match GitBranch::delete_remote(source_branch) {
                Ok(()) => {
                    log_success!("Remote branch '{}' deleted successfully", source_branch);
                }
                Err(e) => {
                    log_warning!("Failed to delete remote branch '{}': {}", source_branch, e);
                    log_info!(
                        "You may need to delete it manually: git push origin --delete {}",
                        source_branch
                    );
                }
            }
        }

        Ok(())
    }
}
