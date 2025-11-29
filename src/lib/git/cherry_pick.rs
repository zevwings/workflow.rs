//! Git Cherry-pick 操作
//!
//! 本模块提供了 Git cherry-pick 相关的完整功能，包括：
//! - 应用提交到当前分支
//! - 应用提交但不提交（保留在工作区）
//! - 继续或中止 cherry-pick 操作
//! - 检查 cherry-pick 操作状态

use anyhow::{Context, Result};

use super::helpers::cmd_run;

/// Git Cherry-pick 管理
///
/// 提供 cherry-pick 相关的操作功能，包括：
/// - 应用提交到当前分支
/// - 应用提交但不提交（保留在工作区）
/// - 继续或中止 cherry-pick 操作
/// - 检查 cherry-pick 操作状态
pub struct GitCherryPick;

impl GitCherryPick {
    /// Cherry-pick 提交到当前分支
    ///
    /// 使用 `git cherry-pick` 将指定的提交应用到当前分支。
    ///
    /// # 参数
    ///
    /// * `commit` - 要 cherry-pick 的提交哈希
    ///
    /// # 错误
    ///
    /// 如果 cherry-pick 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 如果遇到冲突，cherry-pick 会暂停，需要用户手动解决冲突后继续。
    pub fn cherry_pick(commit: &str) -> Result<()> {
        cmd_run(&["cherry-pick", commit])
            .with_context(|| format!("Failed to cherry-pick commit: {}", commit))
    }

    /// Cherry-pick 提交到当前分支（不提交）
    ///
    /// 使用 `git cherry-pick --no-commit` 将指定的提交应用到当前分支的工作区，
    /// 但不创建提交。修改会保留在工作区（未暂存状态）。
    ///
    /// # 参数
    ///
    /// * `commit` - 要 cherry-pick 的提交哈希
    ///
    /// # 错误
    ///
    /// 如果 cherry-pick 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// - 如果遇到冲突，cherry-pick 会暂停，需要用户手动解决冲突后继续
    /// - 修改会保留在工作区，需要手动提交
    pub fn cherry_pick_no_commit(commit: &str) -> Result<()> {
        cmd_run(&["cherry-pick", "--no-commit", commit])
            .with_context(|| format!("Failed to cherry-pick commit (no-commit): {}", commit))
    }

    /// 继续 cherry-pick 操作
    ///
    /// 在解决冲突后，使用 `git cherry-pick --continue` 继续 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果继续操作失败，返回相应的错误信息。
    pub fn cherry_pick_continue() -> Result<()> {
        cmd_run(&["cherry-pick", "--continue"]).context("Failed to continue cherry-pick")
    }

    /// 中止 cherry-pick 操作
    ///
    /// 使用 `git cherry-pick --abort` 中止当前的 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果中止操作失败，返回相应的错误信息。
    pub fn cherry_pick_abort() -> Result<()> {
        cmd_run(&["cherry-pick", "--abort"]).context("Failed to abort cherry-pick")
    }

    /// 检查是否正在进行 cherry-pick 操作
    ///
    /// 通过检查 `.git/CHERRY_PICK_HEAD` 文件是否存在来判断。
    ///
    /// # 返回
    ///
    /// 如果正在进行 cherry-pick 操作，返回 `true`，否则返回 `false`。
    pub fn is_cherry_pick_in_progress() -> bool {
        std::path::Path::new(".git/CHERRY_PICK_HEAD").exists()
    }
}
