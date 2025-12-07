use crate::base::util::dialog::{ConfirmDialog, SelectDialog};
use crate::commands::check;
use crate::git::{GitBranch, GitCommit, GitRepo, GitStash};
use crate::pr::create_provider;
use crate::pr::helpers::get_current_branch_pr_id;
use crate::{log_debug, log_error, log_info, log_success, log_warning};
use anyhow::{Context, Result};

/// PR 分支同步的命令（合并了 integrate 和 sync 的功能）
#[allow(dead_code)]
pub struct PullRequestSyncCommand;

/// 源分支信息
struct SourceBranchInfo {
    /// 分支类型：true 表示远程分支，false 表示本地分支
    is_remote: bool,
    /// 用于合并的分支引用（本地分支名或 origin/branch-name）
    merge_ref: String,
}

/// 同步策略
#[derive(Debug, Clone, Copy)]
enum SyncStrategy {
    /// Merge 合并
    Merge(crate::MergeStrategy),
    /// Rebase 变基
    Rebase,
}

impl PullRequestSyncCommand {
    /// 将指定分支同步到当前分支
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要同步的源分支名称
    /// * `rebase` - 是否使用 rebase（默认使用 merge）
    /// * `ff_only` - 是否只允许 fast-forward 合并
    /// * `squash` - 是否使用 squash 合并
    /// * `push` - 是否在同步后自动推送
    #[allow(dead_code)]
    pub fn sync(
        source_branch: String,
        rebase: bool,
        ff_only: bool,
        squash: bool,
        push: bool,
    ) -> Result<()> {
        // 1. 运行检查（可选，但建议运行）
        log_info!("Running pre-flight checks...");
        if let Err(e) = check::CheckCommand::run_all() {
            log_warning!("Pre-flight checks failed: {}", e);
            ConfirmDialog::new("Continue anyway?")
                .with_default(false)
                .with_cancel_message("Operation cancelled by user")
                .prompt()?;
        }

        // 2. 获取当前分支
        let current_branch = GitBranch::current_branch()?;
        log_success!("Current branch: {}", current_branch);

        // 3. 检查工作区状态并 stash（如果需要）
        let has_stashed = Self::check_working_directory()?;

        // 4. 验证并准备源分支，获取分支信息
        let source_branch_info = Self::prepare_source_branch(&source_branch)?;

        // 5. 确定同步策略
        let strategy = Self::determine_sync_strategy(rebase, ff_only, squash);

        // 6. 执行同步（merge 或 rebase）
        match strategy {
            SyncStrategy::Merge(merge_strategy) => {
                log_success!("Merging '{}' into '{}'...", source_branch, current_branch);
                let merge_result =
                    GitBranch::merge_branch(&source_branch_info.merge_ref, merge_strategy);

                // 如果合并失败，恢复 stash（如果有）
                if merge_result.is_err() && has_stashed {
                    log_info!("Merge failed, attempting to restore stashed changes...");
                    let _ = GitStash::stash_pop();
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
            }
            SyncStrategy::Rebase => {
                log_success!("Rebasing '{}' onto '{}'...", current_branch, source_branch);
                // 执行 rebase
                let rebase_result = GitBranch::rebase_onto(&source_branch_info.merge_ref);

                // 如果 rebase 失败，恢复 stash（如果有）
                if rebase_result.is_err() && has_stashed {
                    log_info!("Rebase failed, attempting to restore stashed changes...");
                    let _ = GitStash::stash_pop();
                }

                match rebase_result {
                    Ok(()) => {
                        log_success!("Rebase completed successfully");
                    }
                    Err(e) => {
                        // 检查是否是 rebase 冲突
                        let error_msg = e.to_string().to_lowercase();
                        if error_msg.contains("conflict") || error_msg.contains("could not apply") {
                            log_error!("Rebase conflicts detected!");
                            log_info!("Please resolve the conflicts manually:");
                            log_info!("  1. Review conflicted files");
                            log_info!("  2. Resolve conflicts");
                            log_info!("  3. Stage resolved files: git add <files>");
                            log_info!("  4. Continue rebase: git rebase --continue");
                            log_info!("  5. Push when ready: git push --force-with-lease");
                            anyhow::bail!("Rebase conflicts detected. Please resolve manually.");
                        }
                        return Err(e);
                    }
                }
            }
        }

        // 7. 根据分支类型处理同步后的操作
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
                    let _ = GitStash::stash_pop();
                }
            }
        } else {
            // 本地分支：同步后立即恢复 stash（如果有）
            if has_stashed {
                log_info!("Restoring stashed changes...");
                let _ = GitStash::stash_pop();
            }

            // 推送到远程（如果需要）
            if push {
                log_success!("Pushing to remote...");
                let exists_remote = GitBranch::has_remote_branch(&current_branch)
                    .context("Failed to check if branch exists on remote")?;

                // 如果是 rebase，使用 force-with-lease
                let use_force = matches!(strategy, SyncStrategy::Rebase);
                if use_force {
                    GitBranch::push_force_with_lease(&current_branch)
                        .context("Failed to push to remote (force-with-lease)")?;
                } else {
                    GitBranch::push(&current_branch, !exists_remote)
                        .context("Failed to push to remote")?;
                }
                log_success!("Pushed to remote successfully");
            } else {
                log_info!("Skipping push (--no-push flag set)");
                log_info!("You can push manually with: git push");
            }
        }

        // 8. 管理 PR 和分支清理（交互式）
        Self::handle_source_branch_cleanup(&source_branch, &current_branch)?;

        log_success!("Sync completed successfully!");
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
            let options = vec![
                "Stash changes and continue".to_string(),
                "Abort operation".to_string(),
            ];
            let selected = SelectDialog::new("How would you like to proceed?", options)
                .with_default(0)
                .prompt()
                .context("Failed to get user choice")?;
            let choice = if selected == "Stash changes and continue" {
                0
            } else {
                1
            };

            match choice {
                0 => {
                    log_info!("Stashing uncommitted changes...");
                    GitStash::stash_push(Some("Auto-stash before branch sync"))?;
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
    /// 返回源分支信息（类型、合并引用）。
    fn prepare_source_branch(source_branch: &str) -> Result<SourceBranchInfo> {
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

    /// 确定同步策略
    ///
    /// 根据命令行参数确定使用的同步策略。
    fn determine_sync_strategy(rebase: bool, ff_only: bool, squash: bool) -> SyncStrategy {
        if rebase {
            SyncStrategy::Rebase
        } else if ff_only {
            SyncStrategy::Merge(crate::MergeStrategy::FastForwardOnly)
        } else if squash {
            SyncStrategy::Merge(crate::MergeStrategy::Squash)
        } else {
            SyncStrategy::Merge(crate::MergeStrategy::Merge)
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

    /// 查找源分支的 PR ID
    ///
    /// 尝试通过多种方式查找源分支的 PR ID：
    /// 1. 通过工作历史查找（主要方式）
    /// 2. 对于 Codeup，尝试通过 get_pull_request_info API 查找
    ///
    /// 返回 PR ID（如果找到），否则返回 None。
    fn find_source_branch_pr_id(source_branch: &str) -> Result<Option<String>> {
        // 方法 1: 通过工作历史查找（主要方式，支持所有平台）
        let remote_url = GitRepo::get_remote_url().ok();
        if let Some(pr_id) = crate::jira::history::JiraWorkHistory::find_pr_id_by_branch(
            source_branch,
            remote_url.as_deref(),
        )? {
            log_debug!(
                "Found PR #{} for source branch '{}' from work-history",
                pr_id,
                source_branch
            );
            return Ok(Some(pr_id));
        }

        // 方法 2: 对于 Codeup，尝试通过 API 查找（如果工作历史中没有）
        let repo_type = crate::git::GitRepo::detect_repo_type()?;
        if matches!(repo_type, crate::git::RepoType::Codeup) {
            // Codeup 的 get_pull_request_info 支持通过分支名查找
            let provider = create_provider()?;
            // 尝试通过分支名获取 PR 信息（Codeup 支持）
            // 如果成功，说明分支有 PR，但我们需要从返回的信息中提取 PR ID
            // 由于无法直接提取，这里只做验证，实际 PR ID 仍依赖工作历史
            if provider.get_pull_request_info(source_branch).is_ok() {
                log_debug!("Source branch '{}' appears to have a PR (Codeup), but PR ID not found in work-history", source_branch);
                // 返回 None，让用户手动处理
            }
        }

        Ok(None)
    }

    /// 处理源分支的 PR 管理和分支清理
    ///
    /// 根据源分支是否有 PR，进行不同的交互式处理：
    /// - 如果有 PR：询问是否关闭 PR，yes 则关闭 PR 并删除分支，no 则跳过
    /// - 如果没有 PR：询问是否删除分支，yes 则删除，no 则跳过
    fn handle_source_branch_cleanup(source_branch: &str, current_branch: &str) -> Result<()> {
        // 不能删除当前分支
        if source_branch == current_branch {
            log_info!(
                "Source branch '{}' is the current branch, skipping cleanup",
                source_branch
            );
            return Ok(());
        }

        // 检查源分支是否存在
        let (exists_local, exists_remote) = GitBranch::is_branch_exists(source_branch)
            .context("Failed to check if source branch exists")?;

        if !exists_local && !exists_remote {
            log_info!(
                "Source branch '{}' does not exist, nothing to cleanup",
                source_branch
            );
            return Ok(());
        }

        // 检查是否为默认分支
        let default_branch = GitBranch::get_default_branch().ok();
        if let Some(ref default) = default_branch {
            if source_branch == default {
                log_warning!(
                    "Source branch '{}' is the default branch, skipping cleanup for safety",
                    source_branch
                );
                return Ok(());
            }
        }

        // 查找源分支的 PR ID
        let source_pr_id = Self::find_source_branch_pr_id(source_branch)?;

        match source_pr_id {
            Some(pr_id) => {
                // 有 PR：询问是否关闭 PR
                log_info!("Source branch '{}' has PR #{}", source_branch, pr_id);
                let should_close = ConfirmDialog::new(format!(
                    "Close PR #{} and delete source branch '{}' (local and remote)?",
                    pr_id, source_branch
                ))
                .with_default(true)
                .with_cancel_message("PR and branch cleanup cancelled by user")
                .prompt()?;

                if should_close {
                    // 关闭 PR
                    let provider = create_provider()?;
                    match provider.close_pull_request(&pr_id) {
                        Ok(()) => {
                            log_success!("PR #{} closed successfully", pr_id);
                        }
                        Err(e) => {
                            log_warning!("Failed to close PR #{}: {}", pr_id, e);
                            log_info!("Continuing with branch deletion...");
                        }
                    }

                    // 删除分支
                    Self::delete_merged_branch(source_branch)?;
                } else {
                    log_info!("Skipping PR closure and branch deletion as requested by user");
                }
            }
            None => {
                // 没有 PR：询问是否删除分支
                let should_delete = ConfirmDialog::new(format!(
                    "Delete source branch '{}' (local and remote)?",
                    source_branch
                ))
                .with_default(true)
                .with_cancel_message("Branch deletion cancelled by user")
                .prompt()?;

                if should_delete {
                    Self::delete_merged_branch(source_branch)?;
                } else {
                    log_info!("Skipping branch deletion as requested by user");
                }
            }
        }

        Ok(())
    }

    /// 删除被合并的源分支
    ///
    /// 在合并成功后，删除源分支（本地和远程）。
    fn delete_merged_branch(source_branch: &str) -> Result<()> {
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
