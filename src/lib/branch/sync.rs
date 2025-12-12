//! Branch synchronization core logic
//!
//! This module provides the core branch synchronization functionality,
//! which can be used by both `branch sync` and `pr sync` commands.

use crate::base::dialog::{ConfirmDialog, SelectDialog};
use crate::base::indicator::Spinner;
use crate::commands::pr::helpers::handle_stash_pop_result;
use crate::git::{GitBranch, GitCommit, GitRepo, GitStash};
use crate::{log_break, log_error, log_info, log_success, log_warning};
use anyhow::{Context, Result};

/// 源分支信息
#[derive(Debug, Clone)]
pub struct SourceBranchInfo {
    /// 分支类型：true 表示远程分支，false 表示本地分支
    pub is_remote: bool,
    /// 用于合并的分支引用（本地分支名或 origin/branch-name）
    pub merge_ref: String,
}

/// 同步策略
#[derive(Debug, Clone, Copy)]
pub enum SyncStrategy {
    /// Merge 合并
    Merge(crate::MergeStrategy),
    /// Rebase 变基
    Rebase,
}

/// 同步选项
#[derive(Debug, Clone)]
pub struct BranchSyncOptions {
    /// 源分支名称（要合并到当前分支的分支）
    pub source_branch: String,
    /// 是否使用 rebase
    pub rebase: bool,
    /// 是否只允许 fast-forward
    pub ff_only: bool,
    /// 是否使用 squash
    pub squash: bool,
}

/// 同步结果
#[derive(Debug, Clone)]
pub struct BranchSyncResult {
    /// 当前分支名称
    pub current_branch: String,
    /// 源分支信息
    pub source_branch_info: SourceBranchInfo,
    /// 使用的同步策略
    pub strategy: SyncStrategy,
    /// 是否执行了 stash
    pub has_stashed: bool,
    /// 同步是否成功
    pub success: bool,
}

/// 同步后回调
pub trait BranchSyncCallbacks {
    /// 同步成功后调用（在恢复 stash 之前）
    ///
    /// # 参数
    /// - `result`: 同步结果
    /// - `is_remote`: 源分支是否为远程分支
    ///
    /// # 返回
    /// - `Ok(true)`: 已处理（如更新了 PR），跳过 stash pop
    /// - `Ok(false)`: 未处理，继续恢复 stash
    fn on_sync_success(&self, result: &BranchSyncResult, is_remote: bool) -> Result<bool>;

    /// 同步后清理回调（在恢复 stash 之后）
    ///
    /// # 参数
    /// - `result`: 同步结果
    /// - `source_branch`: 源分支名称
    fn on_after_sync(&self, result: &BranchSyncResult, source_branch: &str) -> Result<()>;
}

/// 分支同步核心逻辑
pub struct BranchSync;

impl BranchSync {
    /// 执行分支同步（核心逻辑）
    pub fn sync(
        options: BranchSyncOptions,
        callbacks: Option<Box<dyn BranchSyncCallbacks>>,
    ) -> Result<BranchSyncResult> {
        // 1. 获取当前分支
        let current_branch = GitBranch::current_branch()?;
        log_success!("Current branch: {}", current_branch);

        // 2. 检查工作区并 stash
        let has_stashed = Self::check_working_directory()?;

        // 3. 验证并准备源分支（要合并到当前分支的分支）
        let source_branch_info = Self::prepare_source_branch(&options.source_branch)?;

        // 4. 确定同步策略
        let strategy =
            Self::determine_sync_strategy(options.rebase, options.ff_only, options.squash);

        // 5. 执行同步
        let success = Self::execute_sync(
            &options.source_branch,
            &current_branch,
            &source_branch_info,
            strategy,
            has_stashed,
        )?;

        let result = BranchSyncResult {
            current_branch,
            source_branch_info: source_branch_info.clone(),
            strategy,
            has_stashed,
            success,
        };

        // 6. 调用回调处理同步后的操作
        let skip_stash_pop = if let Some(ref callbacks) = callbacks {
            callbacks.on_sync_success(&result, source_branch_info.is_remote)?
        } else {
            false
        };

        // 7. 恢复 stash（如果需要）
        if !skip_stash_pop && has_stashed {
            log_info!("Restoring stashed changes...");
            handle_stash_pop_result(GitStash::stash_pop(None));
        }

        // 8. 推送（如果当前分支在远程存在，使用 ConfirmDialog 确认）
        // 注意：如果回调已经推送了（skip_stash_pop == true），可能已经推送过了
        // 但为了确保所有情况都覆盖，我们仍然检查并询问用户
        let current_branch_exists_remote = GitBranch::has_remote_branch(&result.current_branch)
            .context("Failed to check if current branch exists on remote")?;

        if current_branch_exists_remote {
            // 如果回调已经推送了（通常发生在 PR Sync 且当前分支有 PR 的情况），
            // 再次推送应该不会有问题（Git 会检测到没有新提交）
            // 但为了避免重复推送，我们可以检查一下
            // 不过为了简化逻辑，我们总是询问用户，让用户决定
            Self::prompt_and_push_after_sync(&result)?;
        } else {
            log_info!(
                "Current branch '{}' does not exist on remote",
                result.current_branch
            );
            log_info!("Skipping push. You can push manually when ready: git push");
            if matches!(result.strategy, SyncStrategy::Rebase) {
                log_info!("Or with force: git push --force-with-lease");
            }
        }

        // 9. 调用清理回调
        if let Some(ref callbacks) = callbacks {
            callbacks.on_after_sync(&result, &options.source_branch)?;
        }

        Ok(result)
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
    /// 检查要合并到当前分支的源分支是否存在（本地或远程），如果只在远程则先 fetch。
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
            Spinner::with("Fetching from remote...", || {
                GitRepo::fetch().context("Failed to fetch from remote")
            })?;
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

    /// 执行同步操作
    fn execute_sync(
        source_branch: &str,
        current_branch: &str,
        source_branch_info: &SourceBranchInfo,
        strategy: SyncStrategy,
        has_stashed: bool,
    ) -> Result<bool> {
        match strategy {
            SyncStrategy::Merge(merge_strategy) => {
                let merge_result = Spinner::with(
                    format!("Merging '{}' into '{}'...", source_branch, current_branch),
                    || GitBranch::merge_branch(&source_branch_info.merge_ref, merge_strategy),
                );

                // 如果合并失败，恢复 stash（如果有）
                if merge_result.is_err() && has_stashed {
                    log_info!("Merge failed, attempting to restore stashed changes...");
                    handle_stash_pop_result(GitStash::stash_pop(None));
                }

                match merge_result {
                    Ok(()) => {
                        log_success!("Merge completed successfully");
                        Ok(true)
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
                        Err(e)
                    }
                }
            }
            SyncStrategy::Rebase => {
                // 执行 rebase
                let rebase_result = Spinner::with(
                    format!("Rebasing '{}' onto '{}'...", current_branch, source_branch),
                    || GitBranch::rebase_onto(&source_branch_info.merge_ref),
                );

                // 如果 rebase 失败，恢复 stash（如果有）
                if rebase_result.is_err() && has_stashed {
                    log_info!("Rebase failed, attempting to restore stashed changes...");
                    handle_stash_pop_result(GitStash::stash_pop(None));
                }

                match rebase_result {
                    Ok(()) => {
                        log_success!("Rebase completed successfully");
                        Ok(true)
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
                        Err(e)
                    }
                }
            }
        }
    }

    /// 询问用户并推送同步后的更改
    fn prompt_and_push_after_sync(result: &BranchSyncResult) -> Result<()> {
        let use_force = matches!(result.strategy, SyncStrategy::Rebase);
        let push_message = if use_force {
            format!(
                "Push '{}' to remote (force-with-lease)?",
                result.current_branch
            )
        } else {
            format!("Push '{}' to remote?", result.current_branch)
        };

        let should_push = ConfirmDialog::new(&push_message)
            .with_default(true)
            .with_cancel_message("Push cancelled by user")
            .prompt()?;

        if should_push {
            Self::push_after_sync(result)?;
        } else {
            log_info!("Skipping push as requested by user");
            log_info!("You can push manually with: git push");
            if use_force {
                log_info!("Or with force: git push --force-with-lease");
            }
        }

        Ok(())
    }

    /// 推送同步后的更改
    fn push_after_sync(result: &BranchSyncResult) -> Result<()> {
        let exists_remote = GitBranch::has_remote_branch(&result.current_branch)
            .context("Failed to check if branch exists on remote")?;

        // 如果是 rebase，使用 force-with-lease
        let use_force = matches!(result.strategy, SyncStrategy::Rebase);
        log_break!();
        log_info!("Pushing to remote...");
        log_break!();
        if use_force {
            GitBranch::push_force_with_lease(&result.current_branch)
                .context("Failed to push to remote (force-with-lease)")?;
        } else {
            GitBranch::push(&result.current_branch, !exists_remote)
                .context("Failed to push to remote")?;
        }
        log_break!();
        log_success!("Pushed to remote successfully");
        Ok(())
    }
}
