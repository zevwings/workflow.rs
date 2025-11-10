//! Git 提交相关操作
//!
//! 本模块提供了 Git 提交相关的核心功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）

use anyhow::{Context, Result};
use duct::cmd;

use crate::log_info;

/// Git 操作结构体
///
/// 提供 Git 仓库的各种操作功能，包括提交、推送、分支管理等。
pub struct Git;

impl Git {
    /// 检查 Git 状态
    ///
    /// 使用 `--porcelain` 选项获取简洁、稳定的输出格式。
    /// 该格式适合程序解析，不包含颜色和装饰性输出。
    ///
    /// # 返回
    ///
    /// 返回 Git 状态的简洁输出字符串。
    pub fn status() -> Result<String> {
        cmd("git", &["status", "--porcelain"])
            .read()
            .context("Failed to run git status")
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
        let has_worktree_changes = cmd("git", &["diff", "--quiet"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_err(); // 如果有差异，返回非零退出码（Err）

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
        // 所以如果命令成功（返回 0），说明没有暂存的文件；如果失败（返回非零），说明有暂存的文件
        let result = cmd("git", &["diff", "--cached", "--quiet"])
            .stdout_null()
            .stderr_null()
            .run();

        match result {
            Ok(_) => Ok(false), // 没有暂存的文件
            Err(_) => Ok(true), // 有暂存的文件
        }
    }

    /// 添加所有文件到暂存区
    ///
    /// 使用 `git add --all` 将所有已修改、新增和删除的文件添加到暂存区。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn add_all() -> Result<()> {
        cmd("git", &["add", "--all"])
            .run()
            .context("Failed to add all files")?;
        Ok(())
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
        if !no_verify && Self::has_pre_commit() {
            Self::run_pre_commit()?;
        }

        // 7. 执行提交
        let mut args = vec!["commit", "-m", message];
        if no_verify {
            args.push("--no-verify");
        }

        cmd("git", &args).run().context("Failed to commit")?;
        Ok(())
    }
}
