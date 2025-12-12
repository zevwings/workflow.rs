use crate::base::dialog::ConfirmDialog;
use crate::branch::sync::{BranchSync, BranchSyncCallbacks, BranchSyncOptions, BranchSyncResult};
use crate::commands::check;
use crate::git::GitBranch;
use crate::pr::create_provider_auto;
use crate::pr::helpers::get_current_branch_pr_id;
use crate::{log_break, log_debug, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// PR 分支同步的命令（合并了 integrate 和 sync 的功能）
#[allow(dead_code)]
pub struct PullRequestSyncCommand;

/// PR 同步回调实现
struct PRSyncCallbacks;

impl BranchSyncCallbacks for PRSyncCallbacks {
    fn on_sync_success(&self, result: &BranchSyncResult, is_remote: bool) -> Result<bool> {
        if is_remote {
            // 远程分支：检查并更新当前分支的 PR
            let pr_updated =
                PullRequestSyncCommand::check_and_update_current_branch_pr(&result.current_branch)?;
            if pr_updated {
                // 已更新 PR，不需要 stash pop（代码已推送）
                log_info!("PR updated, skipping stash pop");
            }
            Ok(pr_updated) // 如果更新了 PR，跳过 stash pop
        } else {
            Ok(false) // 本地分支，不跳过 stash pop
        }
    }

    fn on_after_sync(&self, result: &BranchSyncResult, source_branch: &str) -> Result<()> {
        // 处理源分支的 PR 和分支清理
        PullRequestSyncCommand::handle_source_branch_cleanup(source_branch, &result.current_branch)
    }
}

impl PullRequestSyncCommand {
    /// 将指定分支同步到当前分支
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要合并到当前分支的源分支名称
    /// * `rebase` - 是否使用 rebase（默认使用 merge）
    /// * `ff_only` - 是否只允许 fast-forward 合并
    /// * `squash` - 是否使用 squash 合并
    #[allow(dead_code)]
    pub fn sync(source_branch: String, rebase: bool, ff_only: bool, squash: bool) -> Result<()> {
        // 1. 运行检查（可选，但建议运行）
        log_info!("Running pre-flight checks...");
        if let Err(e) = check::CheckCommand::run_all() {
            log_warning!("Pre-flight checks failed: {}", e);
            ConfirmDialog::new("Continue anyway?")
                .with_default(false)
                .with_cancel_message("Operation cancelled by user")
                .prompt()?;
        }

        let options = BranchSyncOptions {
            source_branch: source_branch.clone(),
            rebase,
            ff_only,
            squash,
        };

        // 使用 PR 回调
        let callbacks = Box::new(PRSyncCallbacks);

        BranchSync::sync(options, Some(callbacks))?;

        log_break!();
        log_success!("Sync completed successfully!");
        Ok(())
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

            // 检查当前分支是否在远程存在
            let exists_remote = GitBranch::has_remote_branch(current_branch)
                .wrap_err("Failed to check if current branch exists on remote")?;

            // 推送代码以更新 PR
            log_info!("Pushing changes to update PR...");
            GitBranch::push(current_branch, !exists_remote)
                .wrap_err("Failed to push to remote to update PR")?;
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
    ///
    /// 返回 PR ID（如果找到），否则返回 None。
    fn find_source_branch_pr_id(source_branch: &str) -> Result<Option<String>> {
        // 方法 1: 通过工作历史查找（主要方式，支持所有平台）
        let remote_url = crate::git::GitRepo::get_remote_url().ok();
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

        // Codeup 特定逻辑已移除（Codeup support has been removed）
        // 方法 2: 对于 Codeup，尝试通过 API 查找（如果工作历史中没有）
        // ... (removed)

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
            .wrap_err("Failed to check if source branch exists")?;

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
                    let provider = create_provider_auto()?;
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
            .wrap_err("Failed to check if source branch exists")?;

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
                .wrap_err_with(|| format!("Failed to delete local branch: {}", source_branch))?;
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
