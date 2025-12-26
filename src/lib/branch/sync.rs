//! Branch synchronization core logic
//!
//! This module provides the core branch synchronization functionality,
//! which can be used by both `branch sync` and `pr sync` commands.

use crate::base::dialog::{ConfirmDialog, SelectDialog};
use crate::base::indicator::Spinner;
use crate::commands::pr::helpers::handle_stash_pop_result;
use crate::git::{GitBranch, GitCommandPublic as GitCommand, MergeStrategy, StashPopResult};
use crate::{log_break, log_error, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

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
        Self::sync_in(std::env::current_dir().wrap_err("Failed to get current directory")?, options, callbacks)
    }

    /// 执行分支同步（指定仓库路径）
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    /// * `options` - 同步选项
    /// * `callbacks` - 同步后回调
    pub fn sync_in(
        repo_path: impl AsRef<Path>,
        options: BranchSyncOptions,
        callbacks: Option<Box<dyn BranchSyncCallbacks>>,
    ) -> Result<BranchSyncResult> {
        let repo_path = repo_path.as_ref();

        let current_branch = GitBranch::current_branch_in(repo_path)?;
        log_success!("Current branch: {}", current_branch);

        let has_stashed = Self::check_working_directory_in(repo_path)?;

        let source_branch_info = Self::prepare_source_branch_in(repo_path, &options.source_branch)?;

        let strategy =
            Self::determine_sync_strategy(options.rebase, options.ff_only, options.squash);

        let success = Self::execute_sync_in(
            repo_path,
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

        let skip_stash_pop = if let Some(ref callbacks) = callbacks {
            callbacks.on_sync_success(&result, source_branch_info.is_remote)?
        } else {
            false
        };

        if !skip_stash_pop && has_stashed {
            log_info!("Restoring stashed changes...");
            handle_stash_pop_result(Self::stash_pop_in(repo_path, None));
        }

        // 如果回调已经推送了（skip_stash_pop == true），可能已经推送过了
        // 但为了确保所有情况都覆盖，我们仍然检查并询问用户
        let current_branch_exists_remote = Self::has_remote_branch_in(repo_path, &result.current_branch)
            .wrap_err("Failed to check if current branch exists on remote")?;

        if current_branch_exists_remote {
            // 如果回调已经推送了（通常发生在 PR Sync 且当前分支有 PR 的情况），
            // 再次推送应该不会有问题（Git 会检测到没有新提交）
            // 但为了避免重复推送，我们可以检查一下
            // 不过为了简化逻辑，我们总是询问用户，让用户决定
            Self::prompt_and_push_after_sync_in(repo_path, &result)?;
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

        if let Some(ref callbacks) = callbacks {
            callbacks.on_after_sync(&result, &options.source_branch)?;
        }

        Ok(result)
    }

    /// 检查工作区状态（指定仓库路径）
    ///
    /// 检查是否有未提交的更改，如果有则提示用户处理。
    /// 返回是否执行了 stash 操作。
    fn check_working_directory_in(repo_path: impl AsRef<Path>) -> Result<bool> {
        let repo_path = repo_path.as_ref();
        let has_uncommitted = Self::has_commit_in(repo_path)
            .wrap_err("Failed to check working directory status")?;

        if has_uncommitted {
            log_warning!("Working directory has uncommitted changes");
            let options = vec![
                "Stash changes and continue".to_string(),
                "Abort operation".to_string(),
            ];
            let selected = SelectDialog::new("How would you like to proceed?", options)
                .with_default(0)
                .prompt()
                .wrap_err("Failed to get user choice")?;
            let choice = if selected == "Stash changes and continue" {
                0
            } else {
                1
            };

            match choice {
                0 => {
                    log_info!("Stashing uncommitted changes...");
                    Self::stash_push_in(repo_path, Some("Auto-stash before branch sync"))?;
                    log_success!("Changes stashed successfully");
                    Ok(true)
                }
                1 => {
                    color_eyre::eyre::bail!("Operation cancelled by user");
                }
                _ => unreachable!(),
            }
        } else {
            Ok(false)
        }
    }

    /// 验证并准备源分支（指定仓库路径）
    ///
    /// 检查要合并到当前分支的源分支是否存在（本地或远程），如果只在远程则先 fetch。
    /// 返回源分支信息（类型、合并引用）。
    fn prepare_source_branch_in(repo_path: impl AsRef<Path>, source_branch: &str) -> Result<SourceBranchInfo> {
        let repo_path = repo_path.as_ref();
        let (exists_local, exists_remote) = Self::is_branch_exists_in(repo_path, source_branch)
            .wrap_err("Failed to check if source branch exists")?;

        if !exists_local && !exists_remote {
            color_eyre::eyre::bail!(
                "Source branch '{}' does not exist locally or remotely",
                source_branch
            );
        }

        if !exists_local && exists_remote {
            // 分支只在远程，需要先 fetch 以确保有最新的引用
            log_info!("Source branch '{}' only exists on remote", source_branch);
            Spinner::with("Fetching from remote...", || {
                Self::fetch_in(repo_path).wrap_err("Failed to fetch from remote")
            })?;
            log_success!("Fetched latest changes from remote");
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

    /// 执行同步操作（指定仓库路径）
    fn execute_sync_in(
        repo_path: impl AsRef<Path>,
        source_branch: &str,
        current_branch: &str,
        source_branch_info: &SourceBranchInfo,
        strategy: SyncStrategy,
        has_stashed: bool,
    ) -> Result<bool> {
        let repo_path = repo_path.as_ref();
        match strategy {
            SyncStrategy::Merge(merge_strategy) => {
                let merge_result = Spinner::with(
                    format!("Merging '{}' into '{}'...", source_branch, current_branch),
                    || Self::merge_branch_in(repo_path, &source_branch_info.merge_ref, merge_strategy),
                );

                // 如果合并失败，恢复 stash（如果有）
                if merge_result.is_err() && has_stashed {
                    log_info!("Merge failed, attempting to restore stashed changes...");
                    handle_stash_pop_result(Self::stash_pop_in(repo_path, None));
                }

                match merge_result {
                    Ok(()) => {
                        log_success!("Merge completed successfully");
                        Ok(true)
                    }
                    Err(e) => {
                        // 检查是否是合并冲突
                        if let Ok(has_conflicts) = Self::has_merge_conflicts_in(repo_path) {
                            if has_conflicts {
                                log_error!("Merge conflicts detected!");
                                log_info!("Please resolve the conflicts manually:");
                                log_info!("  1. Review conflicted files");
                                log_info!("  2. Resolve conflicts");
                                log_info!("  3. Stage resolved files: git add <files>");
                                log_info!("  4. Complete the merge: git commit");
                                log_info!("  5. Push when ready: git push");
                                color_eyre::eyre::bail!(
                                    "Merge conflicts detected. Please resolve manually."
                                );
                            }
                        }
                        Err(e)
                    }
                }
            }
            SyncStrategy::Rebase => {
                let rebase_result = Spinner::with(
                    format!("Rebasing '{}' onto '{}'...", current_branch, source_branch),
                    || Self::rebase_onto_in(repo_path, &source_branch_info.merge_ref),
                );

                // 如果 rebase 失败，恢复 stash（如果有）
                if rebase_result.is_err() && has_stashed {
                    log_info!("Rebase failed, attempting to restore stashed changes...");
                    handle_stash_pop_result(Self::stash_pop_in(repo_path, None));
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
                            color_eyre::eyre::bail!(
                                "Rebase conflicts detected. Please resolve manually."
                            );
                        }
                        Err(e)
                    }
                }
            }
        }
    }

    /// 询问用户并推送同步后的更改（指定仓库路径）
    fn prompt_and_push_after_sync_in(repo_path: impl AsRef<Path>, result: &BranchSyncResult) -> Result<()> {
        let repo_path = repo_path.as_ref();
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
            Self::push_after_sync_in(repo_path, result)?;
        } else {
            log_info!("Skipping push as requested by user");
            log_info!("You can push manually with: git push");
            if use_force {
                log_info!("Or with force: git push --force-with-lease");
            }
        }

        Ok(())
    }

    /// 推送同步后的更改（指定仓库路径）
    fn push_after_sync_in(repo_path: impl AsRef<Path>, result: &BranchSyncResult) -> Result<()> {
        let repo_path = repo_path.as_ref();
        let exists_remote = Self::has_remote_branch_in(repo_path, &result.current_branch)
            .wrap_err("Failed to check if branch exists on remote")?;

        // 如果是 rebase，使用 force-with-lease
        let use_force = matches!(result.strategy, SyncStrategy::Rebase);
        log_break!();
        log_info!("Pushing to remote...");
        log_break!();
        if use_force {
            Self::push_force_with_lease_in(repo_path, &result.current_branch)
                .wrap_err("Failed to push to remote (force-with-lease)")?;
        } else {
            Self::push_in(repo_path, &result.current_branch, !exists_remote)
                .wrap_err("Failed to push to remote")?;
        }
        log_break!();
        log_success!("Pushed to remote successfully");
        Ok(())
    }

    // ==================== 临时包装函数（使用 GitCommand::with_cwd()） ====================

    /// 检查工作区是否有未提交的更改（指定仓库路径）
    fn has_commit_in(repo_path: impl AsRef<Path>) -> Result<bool> {
        let repo_path = repo_path.as_ref();
        // 检查工作区是否有未提交的更改
        let has_worktree_changes = !GitCommand::new(["diff", "--quiet"])
            .with_cwd(repo_path)
            .quiet_success();

        // 检查暂存区是否有未提交的更改
        let has_staged_changes = !GitCommand::new(["diff", "--cached", "--quiet"])
            .with_cwd(repo_path)
            .quiet_success();

        Ok(has_worktree_changes || has_staged_changes)
    }

    /// 保存未提交的修改到 stash（指定仓库路径）
    fn stash_push_in(repo_path: impl AsRef<Path>, message: Option<&str>) -> Result<()> {
        let repo_path = repo_path.as_ref();
        let mut args = vec!["stash", "push"];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }
        GitCommand::new(args)
            .with_cwd(repo_path)
            .run()
            .wrap_err("Failed to stash changes")
    }

    /// 应用并删除指定的 stash（指定仓库路径）
    fn stash_pop_in(repo_path: impl AsRef<Path>, stash_ref: Option<&str>) -> Result<StashPopResult> {
        let repo_path = repo_path.as_ref();
        let stash_ref = stash_ref.unwrap_or("stash@{0}");
        let result = GitCommand::new(["stash", "pop", stash_ref])
            .with_cwd(repo_path)
            .run();

        match result {
            Ok(_) => Ok(StashPopResult {
                restored: true,
                message: Some(format!("Stash {} applied and removed", stash_ref)),
                warnings: vec![],
            }),
            Err(e) => {
                // 检查是否有未合并的路径（冲突文件）
                let has_unmerged = GitCommand::new(["ls-files", "-u"])
                    .with_cwd(repo_path)
                    .read()
                    .map(|output| !output.trim().is_empty())
                    .unwrap_or(false);

                if has_unmerged {
                    let warnings = vec![
                        format!(
                            "Merge conflicts detected when applying stash {}.",
                            stash_ref
                        ),
                        "The stash entry is kept in case you need it again.".to_string(),
                        "Please resolve the conflicts manually and then:".to_string(),
                        "  1. Resolve conflicts in the affected files".to_string(),
                        "  2. Stage the resolved files with: git add <file>".to_string(),
                        "  3. Continue with your workflow".to_string(),
                    ];
                    Ok(StashPopResult {
                        restored: false,
                        message: None,
                        warnings,
                    })
                } else {
                    let warnings = vec![
                        format!("Failed to apply stash {}: {}", stash_ref, e),
                        "The stash entry is kept. You can try again later.".to_string(),
                    ];
                    Ok(StashPopResult {
                        restored: false,
                        message: None,
                        warnings,
                    })
                }
            }
        }
    }

    /// 检查分支是否存在（指定仓库路径）
    fn is_branch_exists_in(repo_path: impl AsRef<Path>, branch_name: &str) -> Result<(bool, bool)> {
        let repo_path = repo_path.as_ref();
        // 使用 git rev-parse --verify 检查本地分支（更高效）
        let exists_local = GitCommand::new([
            "rev-parse",
            "--verify",
            &format!("refs/heads/{}", branch_name),
        ])
        .with_cwd(repo_path)
        .quiet_success();

        // 使用 git rev-parse --verify 检查远程分支（更高效）
        let exists_remote = GitCommand::new([
            "rev-parse",
            "--verify",
            &format!("refs/remotes/origin/{}", branch_name),
        ])
        .with_cwd(repo_path)
        .quiet_success();

        Ok((exists_local, exists_remote))
    }

    /// 从远程获取更新（指定仓库路径）
    fn fetch_in(repo_path: impl AsRef<Path>) -> Result<()> {
        let repo_path = repo_path.as_ref();
        GitCommand::new(["fetch", "origin"])
            .with_cwd(repo_path)
            .run()
            .wrap_err("Failed to fetch from origin")
    }

    /// 合并分支（指定仓库路径）
    fn merge_branch_in(repo_path: impl AsRef<Path>, source_branch: &str, strategy: MergeStrategy) -> Result<()> {
        let repo_path = repo_path.as_ref();
        // 保存合并前的 HEAD，用于验证合并是否完成
        let head_before = GitCommand::new(["rev-parse", "HEAD"])
            .with_cwd(repo_path)
            .read()
            .ok();

        let mut args = vec!["merge"];

        match strategy {
            MergeStrategy::Merge => {
                // 普通合并，不需要额外参数
            }
            MergeStrategy::Squash => {
                args.push("--squash");
            }
            MergeStrategy::FastForwardOnly => {
                args.push("--ff-only");
            }
        }

        args.push(source_branch);

        let merge_result = GitCommand::new(args)
            .with_cwd(repo_path)
            .run();

        // 检查合并是否实际上已经完成
        if merge_result.is_err() {
            // 检查是否有 MERGE_HEAD（表示合并正在进行或刚完成）
            let has_merge_head = GitCommand::new(["rev-parse", "--verify", "MERGE_HEAD"])
                .with_cwd(repo_path)
                .quiet_success();

            // 检查 HEAD 是否已经改变
            let head_after = GitCommand::new(["rev-parse", "HEAD"])
                .with_cwd(repo_path)
                .read()
                .ok();
            let head_changed = if let (Some(before), Some(after)) = (head_before, head_after) {
                before.trim() != after.trim()
            } else {
                false
            };

            if head_changed || has_merge_head {
                // 合并实际上已经完成，忽略引用更新错误
                return Ok(());
            }

            // 否则返回原始错误
            return merge_result.wrap_err_with(|| {
                format!(
                    "Failed to merge branch '{}' into current branch",
                    source_branch
                )
            });
        }

        Ok(())
    }

    /// 检查是否有合并冲突（指定仓库路径）
    fn has_merge_conflicts_in(repo_path: impl AsRef<Path>) -> Result<bool> {
        let repo_path = repo_path.as_ref();
        // 检查是否有未合并的文件
        if !GitCommand::new(["diff", "--check"])
            .with_cwd(repo_path)
            .quiet_success()
        {
            return Ok(true);
        }

        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        if GitCommand::new(["rev-parse", "--verify", "MERGE_HEAD"])
            .with_cwd(repo_path)
            .quiet_success()
        {
            // 检查是否有未解决的冲突文件
            if let Ok(files) = GitCommand::new(["diff", "--name-only", "--diff-filter=U"])
                .with_cwd(repo_path)
                .read()
            {
                return Ok(!files.trim().is_empty());
            }
        }

        Ok(false)
    }

    /// 将当前分支 rebase 到目标分支（指定仓库路径）
    fn rebase_onto_in(repo_path: impl AsRef<Path>, target_branch: &str) -> Result<()> {
        let repo_path = repo_path.as_ref();
        GitCommand::new(["rebase", target_branch])
            .with_cwd(repo_path)
            .run()
            .wrap_err_with(|| format!("Failed to rebase onto branch: {}", target_branch))
    }

    /// 检查分支是否在远程存在（指定仓库路径）
    fn has_remote_branch_in(repo_path: impl AsRef<Path>, branch_name: &str) -> Result<bool> {
        let (_, exists_remote) = Self::is_branch_exists_in(repo_path, branch_name)?;
        Ok(exists_remote)
    }

    /// 使用 force-with-lease 强制推送到远程仓库（指定仓库路径）
    fn push_force_with_lease_in(repo_path: impl AsRef<Path>, branch_name: &str) -> Result<()> {
        let repo_path = repo_path.as_ref();
        GitCommand::new(["push", "--force-with-lease", "origin", branch_name])
            .with_cwd(repo_path)
            .run()
            .wrap_err_with(|| format!("Failed to force push branch: {}", branch_name))
    }

    /// 推送到远程仓库（指定仓库路径）
    fn push_in(repo_path: impl AsRef<Path>, branch_name: &str, set_upstream: bool) -> Result<()> {
        let repo_path = repo_path.as_ref();
        let mut args = vec!["push"];
        if set_upstream {
            args.push("-u");
        }
        args.push("origin");
        args.push(branch_name);

        GitCommand::new(args)
            .with_cwd(repo_path)
            .run()
            .wrap_err_with(|| format!("Failed to push branch: {}", branch_name))
    }
}
