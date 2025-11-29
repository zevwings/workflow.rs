//! Git 提交相关操作
//!
//! 本模块提供了 Git 提交相关的核心功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）

use anyhow::{Context, Result};

use super::helpers::{check_success, cmd_read, cmd_run};
use super::pre_commit::GitPreCommit;
use crate::log_info;

/// Git 提交管理
///
/// 提供提交相关的操作功能，包括：
/// - 检查 Git 状态和工作区更改
/// - 暂存文件（add）
/// - 提交更改（commit）
pub struct GitCommit;

impl GitCommit {
    /// 检查 Git 状态
    ///
    /// 使用 `--porcelain` 选项获取简洁、稳定的输出格式。
    /// 该格式适合程序解析，不包含颜色和装饰性输出。
    ///
    /// # 返回
    ///
    /// 返回 Git 状态的简洁输出字符串。
    pub fn status() -> Result<String> {
        cmd_read(&["status", "--porcelain"]).context("Failed to run git status")
    }

    /// 检查工作区是否有未提交的更改
    ///
    /// 检查工作区和暂存区是否有未提交的更改。
    /// 使用 `git diff --quiet` 检查工作区，使用 `git diff --cached --quiet` 检查暂存区。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有未提交的更改（工作区或暂存区）
    /// - `Ok(false)` - 如果没有未提交的更改
    #[allow(dead_code)]
    pub fn has_commit() -> Result<bool> {
        // 检查工作区是否有未提交的更改
        // 如果有差异，返回非零退出码（is_err() 返回 true）
        let has_worktree_changes = !check_success(&["diff", "--quiet"]);

        // 检查暂存区是否有未提交的更改
        let has_staged_changes = Self::has_staged()?;

        Ok(has_worktree_changes || has_staged_changes)
    }

    /// 检查是否有暂存的文件
    ///
    /// 使用 `git diff --cached --quiet` 检查暂存区是否有文件。
    /// `--quiet` 选项会在有差异时返回非零退出码，无差异时返回 0。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有暂存的文件
    /// - `Ok(false)` - 如果没有暂存的文件
    pub(crate) fn has_staged() -> Result<bool> {
        // 使用 git diff --cached --quiet 检查是否有暂存的文件
        // --quiet 选项：如果有差异，返回非零退出码；如果没有差异，返回 0
        // 所以如果命令成功（返回 true），说明没有暂存的文件；如果失败（返回 false），说明有暂存的文件
        Ok(!check_success(&["diff", "--cached", "--quiet"]))
    }

    /// 添加所有文件到暂存区
    ///
    /// 使用 `git add --all` 将所有已修改、新增和删除的文件添加到暂存区。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn add_all() -> Result<()> {
        cmd_run(&["add", "--all"]).context("Failed to add all files")
    }

    /// 提交更改
    ///
    /// 自动暂存所有已修改的文件，然后提交。
    /// 如果存在 pre-commit hooks，会在提交前执行（除非 `no_verify` 为 `true`）。
    ///
    /// # 参数
    ///
    /// * `message` - 提交消息
    /// * `no_verify` - 是否跳过 pre-commit hooks 验证
    ///
    /// # 行为
    ///
    /// 1. 检查是否有未提交的更改，如果没有则直接返回
    /// 2. 暂存所有已修改的文件
    /// 3. 如果 `no_verify` 为 `false` 且存在 pre-commit hooks，则执行 hooks
    /// 4. 执行提交操作
    pub fn commit(message: &str, no_verify: bool) -> Result<()> {
        // 1. 使用 git diff --quiet 检查是否有更改（更高效）
        let has_changes = Self::has_commit()?;

        // 2. 如果没有更改，直接返回
        if !has_changes {
            log_info!("Nothing to commit, working tree clean");
            return Ok(());
        }

        // 3. 暂存所有已修改的文件
        // 注意：即使文件已经在暂存区，执行 add_all() 也是安全的，不会造成问题
        // 这样可以确保所有更改都被暂存，包括未暂存和已暂存的更改
        Self::add_all().context("Failed to stage changes")?;

        // 6. 如果不需要跳过验证，且存在 pre-commit，则先执行 pre-commit
        if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
        }

        // 7. 执行提交
        let mut args = vec!["commit", "-m", message];
        if no_verify {
            args.push("--no-verify");
        }

        cmd_run(&args).context("Failed to commit")
    }

    /// 获取 Git 修改内容（工作区和暂存区）
    ///
    /// 获取工作区和暂存区的所有修改内容，用于传递给 LLM 生成分支名和 PR 标题。
    ///
    /// # 返回
    ///
    /// 返回 `Option<String>`：
    /// - `Some(String)` - 如果有修改内容，包含暂存区和工作区的修改
    /// - `None` - 如果没有修改内容
    ///
    /// # 格式
    ///
    /// 返回的字符串格式：
    /// ```
    /// Staged changes:
    /// {staged diff content}
    ///
    /// Working tree changes:
    /// {worktree diff content}
    /// ```
    pub fn get_diff() -> Option<String> {
        let mut diff_parts = Vec::new();

        // 获取暂存区的修改
        if let Ok(staged_diff) = cmd_read(&["diff", "--cached"]) {
            if !staged_diff.trim().is_empty() {
                diff_parts.push(format!("Staged changes:\n{}", staged_diff));
            }
        }

        // 获取工作区的修改
        if let Ok(worktree_diff) = cmd_read(&["diff"]) {
            if !worktree_diff.trim().is_empty() {
                diff_parts.push(format!("Working tree changes:\n{}", worktree_diff));
            }
        }

        if diff_parts.is_empty() {
            None
        } else {
            Some(diff_parts.join("\n\n"))
        }
    }

    /// 重置工作区到指定提交
    ///
    /// 使用 `git reset --hard <target>` 将工作区和暂存区重置到指定提交。
    /// 这会丢弃所有未提交的更改。
    ///
    /// # 参数
    ///
    /// * `target` - 目标提交引用（如 "HEAD", "HEAD~1", 分支名等）
    ///   如果为 `None`，则重置到当前 HEAD
    ///
    /// # 警告
    ///
    /// 此操作会**永久丢弃**工作区和暂存区的所有未提交更改，请谨慎使用。
    ///
    /// # 错误
    ///
    /// 如果重置失败，返回相应的错误信息。
    pub fn reset_hard(target: Option<&str>) -> Result<()> {
        let target = target.unwrap_or("HEAD");
        cmd_run(&["reset", "--hard", target])
            .with_context(|| format!("Failed to reset working directory to {}", target))
    }
}
