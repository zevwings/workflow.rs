//! Git 分支操作命令封装
//!
//! 提供分支相关的所有 Git 命令操作，包括：
//! - 分支查询（当前分支、分支列表、分支存在性检查）
//! - 分支操作（创建、切换、删除）
//! - 分支合并（merge、rebase）
//! - 分支推送（push、pull）

use crate::git::commands::{GitCommand, GitError};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git 分支命令操作
pub struct GitBranchCommand;

impl GitBranchCommand {
    /// 获取当前分支名
    ///
    /// 使用 `git branch --show-current` 命令
    pub fn current_branch(cwd: Option<&Path>) -> Result<String> {
        let output = GitCommand::run(&["branch", "--show-current"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        let branch = output.trim();
        if branch.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Not on any branch (detached HEAD state)"
            ));
        }

        Ok(branch.to_string())
    }

    /// 检查分支是否存在（本地）
    ///
    /// 使用 `git show-ref --verify` 命令
    pub fn branch_exists_local(branch_name: &str, cwd: Option<&Path>) -> Result<bool> {
        Ok(GitCommand::check(
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{}", branch_name),
            ],
            cwd,
        ))
    }

    /// 检查分支是否存在（远程）
    ///
    /// 使用 `git show-ref --verify` 命令
    pub fn branch_exists_remote(
        branch_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<bool> {
        let remote = remote.unwrap_or("origin");
        Ok(GitCommand::check(
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/remotes/{}/{}", remote, branch_name),
            ],
            cwd,
        ))
    }

    /// 检查分支是否存在（本地和远程）
    ///
    /// 返回 `(本地存在, 远程存在)`
    pub fn branch_exists(
        branch_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<(bool, bool)> {
        let exists_local = Self::branch_exists_local(branch_name, cwd)?;
        let exists_remote = Self::branch_exists_remote(branch_name, remote, cwd)?;
        Ok((exists_local, exists_remote))
    }

    /// 创建分支
    ///
    /// 使用 `git branch <name>` 命令
    pub fn create_branch(branch_name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["branch", branch_name], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))
    }

    /// 创建或切换分支
    ///
    /// 优先使用 `git switch`（Git 2.23+），失败时回退到 `git checkout`
    pub fn checkout_branch(branch_name: &str, create: bool, cwd: Option<&Path>) -> Result<()> {
        if create {
            // 优先使用 git switch -c
            if GitCommand::execute(&["switch", "-c", branch_name], cwd).is_ok() {
                return Ok(());
            }
            // 回退到 git checkout -b
            GitCommand::execute(&["checkout", "-b", branch_name], cwd)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))
        } else {
            // 优先使用 git switch
            if GitCommand::execute(&["switch", branch_name], cwd).is_ok() {
                return Ok(());
            }
            // 回退到 git checkout
            GitCommand::execute(&["checkout", branch_name], cwd)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))
        }
        .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))
    }

    /// 删除分支
    ///
    /// 使用 `git branch -d` 或 `git branch -D` 命令
    pub fn delete_branch(branch_name: &str, force: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["branch"];
        if force {
            args.push("-D");
        } else {
            args.push("-d");
        }
        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to delete branch: {}", branch_name))
    }

    /// 获取所有本地分支
    ///
    /// 使用 `git branch` 命令
    pub fn list_branches(cwd: Option<&Path>) -> Result<Vec<String>> {
        let output =
            GitCommand::run(&["branch"], cwd).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok(output
            .lines()
            .map(|s| s.trim().trim_start_matches('*').trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// 检查分支是否已合并
    ///
    /// 使用 `git branch --merged` 命令
    pub fn is_merged(branch_name: &str, target: Option<&str>, cwd: Option<&Path>) -> Result<bool> {
        let target = target.unwrap_or("HEAD");
        let output = GitCommand::run(&["branch", "--merged", target], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        // 检查输出中是否包含该分支名
        Ok(output.lines().any(|line| {
            let line = line.trim().trim_start_matches('*').trim();
            line == branch_name || line.ends_with(&format!("/{}", branch_name))
        }))
    }

    /// 合并分支
    ///
    /// 使用 `git merge` 命令
    pub fn merge_branch(
        branch_name: &str,
        strategy: Option<&str>,
        no_ff: bool,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let mut args = vec!["merge"];

        if let Some(strategy) = strategy {
            args.push("--strategy");
            args.push(strategy);
        }

        if no_ff {
            args.push("--no-ff");
        }

        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(|e| match e {
                GitError::MergeConflict { details } => {
                    color_eyre::eyre::eyre!("Merge conflict detected:\n{}", details)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err_with(|| format!("Failed to merge branch: {}", branch_name))
    }

    /// 推送分支
    ///
    /// 使用 `git push` 命令
    pub fn push(
        branch_name: &str,
        force: bool,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        let mut args = vec!["push"];

        if force {
            args.push("--force-with-lease");
        }

        args.push(remote);
        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(|e| match e {
                GitError::AuthenticationFailed { reason } => {
                    color_eyre::eyre::eyre!("Authentication failed: {}", reason)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err_with(|| format!("Failed to push branch {} to {}", branch_name, remote))
    }

    /// 拉取分支
    ///
    /// 使用 `git pull` 命令
    pub fn pull(branch_name: &str, remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        GitCommand::execute(&["pull", remote, branch_name], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to pull branch {} from {}", branch_name, remote))
    }
}
