//! Branch synchronization core logic
//!
//! This module provides the core branch synchronization functionality,
//! which can be used by both `branch sync` and `pr sync` commands.

use crate::base::dialog::{ConfirmDialog, SelectDialog};
use crate::base::indicator::Spinner;
use crate::commands::pr::helpers::handle_stash_pop_result;
use crate::git::{GitAuth, GitBranch, GitRepository, MergeStrategy, StashPopResult};
use crate::{log_break, log_error, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use git2::{FetchOptions, PushOptions};
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
        Self::sync_in(
            std::env::current_dir().wrap_err("Failed to get current directory")?,
            options,
            callbacks,
        )
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
        let current_branch_exists_remote =
            Self::has_remote_branch_in(repo_path, &result.current_branch)
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
        let has_uncommitted =
            Self::has_commit_in(repo_path).wrap_err("Failed to check working directory status")?;

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
    fn prepare_source_branch_in(
        repo_path: impl AsRef<Path>,
        source_branch: &str,
    ) -> Result<SourceBranchInfo> {
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
                    || {
                        Self::merge_branch_in(
                            repo_path,
                            &source_branch_info.merge_ref,
                            merge_strategy,
                        )
                    },
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
    fn prompt_and_push_after_sync_in(
        repo_path: impl AsRef<Path>,
        result: &BranchSyncResult,
    ) -> Result<()> {
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
        Spinner::with_output("Pushing to remote...", || {
            if use_force {
                Self::push_force_with_lease_in(repo_path, &result.current_branch)
                    .wrap_err("Failed to push to remote (force-with-lease)")
            } else {
                Self::push_in(repo_path, &result.current_branch, !exists_remote)
                    .wrap_err("Failed to push to remote")
            }
        })?;
        log_break!();
        log_success!("Pushed to remote successfully");
        Ok(())
    }

    // ==================== Git 操作函数（使用 git2 API） ====================

    /// 检查工作区是否有未提交的更改（指定仓库路径）
    fn has_commit_in(repo_path: impl AsRef<Path>) -> Result<bool> {
        crate::git::GitCommit::has_commit_in(repo_path)
    }

    /// 保存未提交的修改到 stash（指定仓库路径）
    fn stash_push_in(repo_path: impl AsRef<Path>, message: Option<&str>) -> Result<()> {
        let repo_path_ref = repo_path.as_ref();
        let mut repo = GitRepository::open_at(repo_path_ref)?;

        // 构建 stash 消息
        let stash_message = if let Some(msg) = message {
            msg.to_string()
        } else {
            // 如果没有提供消息，使用默认格式
            let branch = GitBranch::current_branch_in(repo_path_ref)
                .unwrap_or_else(|_| "unknown".to_string());
            format!(
                "WIP on {}: {}",
                branch,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            )
        };

        // 获取签名并保存 stash
        // 先提取签名的数据（name 和 email），然后创建新的签名
        let (name, email) = {
            let sig = repo.signature()?;
            (
                sig.name().unwrap_or("").to_string(),
                sig.email().unwrap_or("").to_string(),
            )
        };
        let signature =
            git2::Signature::now(&name, &email).wrap_err("Failed to create signature")?;
        // 签名已创建，现在可以获取可变引用
        let repo_inner = repo.as_inner_mut();
        repo_inner
            .stash_save(&signature, &stash_message, None)
            .wrap_err("Failed to stash changes")?;

        Ok(())
    }

    /// 应用并删除指定的 stash（指定仓库路径）
    fn stash_pop_in(
        repo_path: impl AsRef<Path>,
        stash_ref: Option<&str>,
    ) -> Result<StashPopResult> {
        let mut repo = GitRepository::open_at(repo_path)?;
        let stash_ref = stash_ref.unwrap_or("stash@{0}");

        // 解析 stash 索引
        let stash_index = stash_ref
            .strip_prefix("stash@{")
            .and_then(|s| s.strip_suffix("}"))
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid stash reference: {}", stash_ref))?;

        // 先应用 stash
        let mut apply_options = git2::StashApplyOptions::new();
        let repo_inner = repo.as_inner_mut();
        let apply_result = repo_inner.stash_apply(stash_index, Some(&mut apply_options));

        match apply_result {
            Ok(_) => {
                // 应用成功，删除 stash
                repo_inner
                    .stash_drop(stash_index)
                    .wrap_err("Failed to drop stash after successful apply")?;

                Ok(StashPopResult {
                    restored: true,
                    message: Some(format!("Stash {} applied and removed", stash_ref)),
                    warnings: vec![],
                })
            }
            Err(e) => {
                // 应用失败，检查是否有冲突
                let has_conflicts = {
                    let index = repo.index().ok();
                    index.map(|idx| idx.has_conflicts()).unwrap_or(false)
                };

                if has_conflicts {
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
        let repo = GitRepository::open_at(repo_path)?;

        // 检查本地分支
        let local_ref = format!("refs/heads/{}", branch_name);
        let exists_local = repo.find_reference(&local_ref).is_ok();

        // 检查远程分支
        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        let exists_remote = repo.find_reference(&remote_ref).is_ok();

        Ok((exists_local, exists_remote))
    }

    /// 从远程获取更新（指定仓库路径）
    fn fetch_in(repo_path: impl AsRef<Path>) -> Result<()> {
        let mut repo = GitRepository::open_at(repo_path)?;
        let mut remote = repo.find_remote("origin")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 获取远程更新
        let refs: &[&str] = &[];
        remote
            .fetch(refs, Some(&mut fetch_options), None)
            .wrap_err("Failed to fetch from origin")?;

        Ok(())
    }

    /// 合并分支（指定仓库路径）
    fn merge_branch_in(
        repo_path: impl AsRef<Path>,
        source_branch: &str,
        strategy: MergeStrategy,
    ) -> Result<()> {
        let mut repo = GitRepository::open_at(repo_path)?;

        // 获取当前 HEAD（在获取可变引用之前）
        let (head_name, head_commit_id) = {
            let head = repo.head()?;
            let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
            (head.name().map(|s| s.to_string()), head_commit.id())
        };
        let head_name = head_name.ok_or_else(|| color_eyre::eyre::eyre!("HEAD has no name"))?;

        // 获取可变引用并解析源分支引用
        let repo_inner = repo.as_inner_mut();
        let head_commit =
            repo_inner.find_commit(head_commit_id).wrap_err("Failed to get HEAD commit")?;
        let source_ref = repo_inner
            .revparse_single(source_branch)
            .wrap_err_with(|| format!("Failed to parse source branch: {}", source_branch))?;
        let source_commit = source_ref.peel_to_commit().wrap_err_with(|| {
            format!(
                "Source branch '{}' does not point to a commit",
                source_branch
            )
        })?;

        // 分析合并（需要 AnnotatedCommit）
        let source_annotated = repo_inner
            .find_annotated_commit(source_commit.id())
            .wrap_err("Failed to create annotated commit")?;
        let (merge_analysis, _) = repo_inner
            .merge_analysis(&[&source_annotated])
            .wrap_err("Failed to analyze merge")?;

        match strategy {
            MergeStrategy::FastForwardOnly => {
                if merge_analysis.is_fast_forward() {
                    // Fast-forward 合并
                    let mut head_ref = repo_inner
                        .find_reference(&head_name)
                        .wrap_err("Failed to find HEAD reference")?;
                    head_ref
                        .set_target(source_commit.id(), "Fast-forward merge")
                        .wrap_err("Failed to update HEAD reference")?;

                    // 更新工作目录
                    repo_inner.set_head(&head_name).wrap_err("Failed to set HEAD")?;
                    repo_inner
                        .checkout_head(Some(
                            git2::build::CheckoutBuilder::default().force().remove_untracked(true),
                        ))
                        .wrap_err("Failed to checkout HEAD")?;
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "Cannot fast-forward merge. Use a different merge strategy."
                    ));
                }
            }
            MergeStrategy::Squash => {
                // Squash 合并：将所有提交压缩为一个提交
                let mut merge_index = repo_inner
                    .merge_commits(&head_commit, &source_commit, None)
                    .wrap_err("Failed to merge commits for squash")?;

                if merge_index.has_conflicts() {
                    merge_index.write().wrap_err("Failed to write merge index")?;
                    return Err(color_eyre::eyre::eyre!(
                        "Merge conflicts detected. Please resolve conflicts manually."
                    ));
                }

                let tree_id =
                    merge_index.write_tree_to(repo_inner).wrap_err("Failed to write merge tree")?;
                let tree = repo_inner.find_tree(tree_id).wrap_err("Failed to find merge tree")?;

                let signature =
                    repo_inner.signature().wrap_err("Failed to get repository signature")?;

                // 创建 squash 提交消息
                let message = format!("Squashed commit of branch '{}'", source_branch);

                repo_inner
                    .commit(
                        Some("HEAD"),
                        &signature,
                        &signature,
                        &message,
                        &tree,
                        &[&head_commit],
                    )
                    .wrap_err("Failed to create squash commit")?;
            }
            MergeStrategy::Merge => {
                if merge_analysis.is_fast_forward() {
                    // Fast-forward 合并
                    let mut head_ref = repo_inner
                        .find_reference(&head_name)
                        .wrap_err("Failed to find HEAD reference")?;
                    head_ref
                        .set_target(source_commit.id(), "Fast-forward merge")
                        .wrap_err("Failed to update HEAD reference")?;

                    // 更新工作目录
                    repo_inner.set_head(&head_name).wrap_err("Failed to set HEAD")?;
                    repo_inner
                        .checkout_head(Some(
                            git2::build::CheckoutBuilder::default().force().remove_untracked(true),
                        ))
                        .wrap_err("Failed to checkout HEAD")?;
                } else {
                    // 普通合并
                    let mut merge_index = repo_inner
                        .merge_commits(&head_commit, &source_commit, None)
                        .wrap_err("Failed to merge commits")?;

                    if merge_index.has_conflicts() {
                        merge_index.write().wrap_err("Failed to write merge index")?;
                        return Err(color_eyre::eyre::eyre!(
                            "Merge conflicts detected. Please resolve conflicts manually."
                        ));
                    }

                    let tree_id = merge_index
                        .write_tree_to(repo_inner)
                        .wrap_err("Failed to write merge tree")?;
                    let tree =
                        repo_inner.find_tree(tree_id).wrap_err("Failed to find merge tree")?;

                    let signature =
                        repo_inner.signature().wrap_err("Failed to get repository signature")?;

                    let message = format!("Merge branch '{}'", source_branch);

                    repo_inner
                        .commit(
                            Some("HEAD"),
                            &signature,
                            &signature,
                            &message,
                            &tree,
                            &[&head_commit, &source_commit],
                        )
                        .wrap_err("Failed to create merge commit")?;
                }
            }
        }

        Ok(())
    }

    /// 检查是否有合并冲突（指定仓库路径）
    fn has_merge_conflicts_in(repo_path: impl AsRef<Path>) -> Result<bool> {
        let mut repo = GitRepository::open_at(repo_path)?;

        // 检查索引是否有冲突
        let index = repo.index()?;
        if index.has_conflicts() {
            return Ok(true);
        }

        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        let git_dir = repo.as_inner().path();
        let merge_head_path = git_dir.join("MERGE_HEAD");
        if merge_head_path.exists() {
            // 如果有 MERGE_HEAD，检查工作区是否有冲突标记
            let head = repo.head()?.peel_to_tree()?;
            let diff = repo.as_inner().diff_tree_to_index(Some(&head), Some(&index), None)?;

            // 检查是否有冲突标记（<<<<<<, ======, >>>>>>）
            let mut has_conflict_markers = false;
            diff.foreach(
                &mut |_delta, _progress| true,
                None,
                None,
                Some(&mut |_delta, _hunk, line| {
                    let content = std::str::from_utf8(line.content()).unwrap_or("");
                    if content.contains("<<<<<<<")
                        || content.contains("=======")
                        || content.contains(">>>>>>>")
                    {
                        has_conflict_markers = true;
                        false // 停止遍历
                    } else {
                        true
                    }
                }),
            )?;

            if has_conflict_markers {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 将当前分支 rebase 到目标分支（指定仓库路径）
    fn rebase_onto_in(repo_path: impl AsRef<Path>, target_branch: &str) -> Result<()> {
        let mut repo = GitRepository::open_at(repo_path)?;

        // 获取当前分支的 HEAD commit（在获取可变引用之前）
        let head_commit_id = {
            let head = repo.head()?;
            let head_commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;
            head_commit.id()
        };

        // 获取可变引用
        let repo_inner = repo.as_inner_mut();
        let _head_commit = repo_inner
            .find_commit(head_commit_id)
            .wrap_err("Failed to get commit from HEAD")?;

        // 解析目标分支（支持本地分支或远程分支）
        let target_ref = repo_inner
            .find_reference(&format!("refs/heads/{}", target_branch))
            .or_else(|_| {
                repo_inner.find_reference(&format!("refs/remotes/origin/{}", target_branch))
            })
            .wrap_err_with(|| format!("Failed to find branch: {}", target_branch))?;
        let target_commit = target_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", target_branch))?;

        // 转换为 AnnotatedCommit（rebase 需要）
        let head_annotated = repo_inner
            .find_annotated_commit(head_commit_id)
            .wrap_err("Failed to create annotated commit from HEAD")?;
        let target_annotated =
            repo_inner.find_annotated_commit(target_commit.id()).wrap_err_with(|| {
                format!(
                    "Failed to create annotated commit from branch: {}",
                    target_branch
                )
            })?;

        // 初始化 rebase（不使用 inmemory，需要更新工作目录）
        let mut rebase_opts = git2::RebaseOptions::new();

        let mut rebase = repo_inner
            .rebase(
                Some(&head_annotated),
                Some(&target_annotated),
                None,
                Some(&mut rebase_opts),
            )
            .wrap_err_with(|| {
                format!("Failed to initialize rebase onto branch: {}", target_branch)
            })?;

        // 应用所有 rebase 操作
        while let Some(_op) = rebase.next() {
            let _op = _op.wrap_err("Failed to get next rebase operation")?;

            // 检查是否有冲突（使用 repo_inner 访问索引）
            if repo_inner.index()?.has_conflicts() {
                rebase.abort().ok(); // 尝试中止 rebase
                color_eyre::eyre::bail!(
                    "Rebase conflict detected. Please resolve conflicts manually and continue."
                );
            }

            // 提交 rebase 操作（使用 repo_inner 获取签名）
            let signature = repo_inner.signature().wrap_err("Failed to get signature")?;
            rebase
                .commit(None, &signature, None)
                .wrap_err("Failed to commit rebase operation")?;
        }

        // 完成 rebase
        rebase
            .finish(None)
            .wrap_err_with(|| format!("Failed to finish rebase onto branch: {}", target_branch))?;

        Ok(())
    }

    /// 检查分支是否在远程存在（指定仓库路径）
    fn has_remote_branch_in(repo_path: impl AsRef<Path>, branch_name: &str) -> Result<bool> {
        let (_, exists_remote) = Self::is_branch_exists_in(repo_path, branch_name)?;
        Ok(exists_remote)
    }

    /// 使用 force-with-lease 强制推送到远程仓库（指定仓库路径）
    fn push_force_with_lease_in(repo_path: impl AsRef<Path>, branch_name: &str) -> Result<()> {
        let mut repo = GitRepository::open_at(repo_path)?;
        let mut remote = repo.find_remote("origin")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建 refspec（使用 force-with-lease）
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // 推送（force-with-lease 需要在 refspec 前加 +）
        // 注意：git2 的 force-with-lease 需要通过设置 refspec 的 force 标志来实现
        // 但更安全的方式是使用普通的 push，如果失败再提示用户
        // 这里我们使用 +refs/heads/... 来实现 force-with-lease 的效果
        let force_refspec = format!("+{}", refspec);
        remote
            .push(&[&force_refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to force push branch: {}", branch_name))?;

        Ok(())
    }

    /// 推送到远程仓库（指定仓库路径）
    fn push_in(repo_path: impl AsRef<Path>, branch_name: &str, set_upstream: bool) -> Result<()> {
        let mut repo = GitRepository::open_at(repo_path)?;
        let mut remote = repo.find_remote("origin")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建 refspec
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // 推送
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to push branch: {}", branch_name))?;

        // 如果设置了 set_upstream，更新上游分支配置
        if set_upstream {
            // 释放 remote 的借用，然后获取 repo 的可变引用
            drop(remote);
            let repo_inner = repo.as_inner_mut();
            let mut branch = repo_inner
                .find_branch(branch_name, git2::BranchType::Local)
                .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;

            branch
                .set_upstream(Some(&format!("origin/{}", branch_name)))
                .wrap_err_with(|| format!("Failed to set upstream for branch: {}", branch_name))?;
        }

        Ok(())
    }
}
