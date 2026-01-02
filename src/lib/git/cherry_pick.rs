//! Git Cherry-pick 操作
//!
//! 本模块提供了 Git cherry-pick 相关的完整功能，包括：
//! - 应用提交到当前分支
//! - 应用提交但不提交（保留在工作区）
//! - 继续或中止 cherry-pick 操作
//! - 检查 cherry-pick 操作状态

use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

use super::GitRepository;
use crate::git::commands::cherry_pick::GitCherryPickCommand;
use crate::git::commands::GitCommitCommand;

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
    /// 使用 Git 命令将指定的提交应用到当前分支。
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
        let repo = GitRepository::open()?;
        GitCherryPickCommand::cherry_pick(commit, false, Some(repo.path()))
    }

    /// Cherry-pick 提交到当前分支（不提交）
    ///
    /// 使用 Git 命令将指定的提交应用到当前分支的工作区，
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
        let repo = GitRepository::open()?;
        GitCherryPickCommand::cherry_pick(commit, true, Some(repo.path()))
    }

    /// 继续 cherry-pick 操作
    ///
    /// 在解决冲突后，使用 Git 命令继续 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果继续操作失败，返回相应的错误信息。
    pub fn cherry_pick_continue() -> Result<()> {
        let repo = GitRepository::open()?;

        // 检查是否正在进行 cherry-pick
        if !GitCherryPickCommand::check_status(Some(repo.path()))
            .wrap_err("Failed to check cherry-pick status")?
        {
            color_eyre::eyre::bail!("No cherry-pick in progress");
        }

        // 检查是否有未解决的冲突
        if Self::has_conflicts(Some(repo.path()))? {
            color_eyre::eyre::bail!(
                "Cherry-pick conflicts not resolved. Please resolve conflicts before continuing."
            );
        }

        GitCherryPickCommand::continue_cherry_pick(Some(repo.path()))
    }

    /// 中止 cherry-pick 操作
    ///
    /// 使用 Git 命令中止当前的 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果中止操作失败，返回相应的错误信息。
    pub fn cherry_pick_abort() -> Result<()> {
        let repo = GitRepository::open()?;

        // 检查是否正在进行 cherry-pick
        if !GitCherryPickCommand::check_status(Some(repo.path()))
            .wrap_err("Failed to check cherry-pick status")?
        {
            color_eyre::eyre::bail!("No cherry-pick in progress");
        }

        GitCherryPickCommand::abort_cherry_pick(Some(repo.path()))
    }

    /// 检查是否正在进行 cherry-pick 操作
    ///
    /// 通过检查 `.git/CHERRY_PICK_HEAD` 文件是否存在来判断。
    ///
    /// # 返回
    ///
    /// 如果正在进行 cherry-pick 操作，返回 `true`，否则返回 `false`。
    pub fn is_cherry_pick_in_progress() -> bool {
        GitRepository::open()
            .ok()
            .and_then(|repo| GitCherryPickCommand::check_status(Some(repo.path())).ok())
            .unwrap_or(false)
    }

    /// 检查是否有未解决的冲突
    ///
    /// 使用 `git diff --check` 检查工作区是否有冲突标记。
    ///
    /// # 参数
    ///
    /// * `cwd` - 工作目录路径
    ///
    /// # 返回
    ///
    /// 如果有冲突，返回 `true`，否则返回 `false`。
    fn has_conflicts(cwd: Option<&Path>) -> Result<bool> {
        // 使用 git diff --check 检查冲突标记
        // 如果有冲突，命令会返回非零退出码
        match GitCommitCommand::check_conflicts(cwd) {
            Ok(false) => Ok(false),
            Ok(true) => Ok(true),
            Err(_) => {
                // 进一步检查是否有冲突标记
                let status_output = GitCommitCommand::status(cwd).unwrap_or_default();
                // 检查是否有冲突标记（UU 表示未合并）
                Ok(status_output.lines().any(|line| {
                    line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD")
                }))
            }
        }
    }
}
