//! Branch synchronization command
//!
//! Provides CLI interface for branch synchronization without PR-specific logic.

use crate::branch::sync::{BranchSync, BranchSyncCallbacks, BranchSyncOptions, BranchSyncResult};
use crate::commands::check;
use crate::{log_info, log_success};
use anyhow::Result;

/// 分支同步命令（无 PR 逻辑）
pub struct BranchSyncCommand;

/// 空回调实现（用于纯分支同步，不需要 PR 处理）
struct NoOpCallbacks;

impl BranchSyncCallbacks for NoOpCallbacks {
    fn on_sync_success(&self, _result: &BranchSyncResult, _is_remote: bool) -> Result<bool> {
        Ok(false) // 不跳过 stash pop
    }

    fn on_after_sync(&self, _result: &BranchSyncResult, _source_branch: &str) -> Result<()> {
        Ok(()) // 不执行任何清理
    }
}

impl BranchSyncCommand {
    /// 执行分支同步（纯分支操作，无 PR 逻辑）
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要合并到当前分支的源分支名称
    /// * `rebase` - 是否使用 rebase（默认使用 merge）
    /// * `ff_only` - 是否只允许 fast-forward 合并
    /// * `squash` - 是否使用 squash 合并
    pub fn sync(source_branch: String, rebase: bool, ff_only: bool, squash: bool) -> Result<()> {
        // 运行检查
        log_info!("Running pre-flight checks...");
        check::CheckCommand::run_all()?;

        let options = BranchSyncOptions {
            source_branch: source_branch.clone(),
            rebase,
            ff_only,
            squash,
        };

        // 使用空回调（不需要 PR 处理）
        let callbacks = Box::new(NoOpCallbacks);

        BranchSync::sync(options, Some(callbacks))?;

        log_success!("Branch sync completed successfully!");
        Ok(())
    }
}
