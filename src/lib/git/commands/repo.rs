//! Git 仓库操作命令封装
//!
//! 提供仓库相关的所有 Git 命令操作，包括：
//! - 仓库检测（是否为 Git 仓库）
//! - 远程仓库操作（获取 URL、获取更新）

use crate::git::commands::{GitCommand, GitError};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git 仓库命令操作
pub struct GitRepoCommand;

impl GitRepoCommand {
    /// 检查是否在 Git 仓库中
    ///
    /// 使用 `git rev-parse --git-dir` 命令
    pub fn is_git_repo(cwd: Option<&Path>) -> bool {
        GitCommand::check(&["rev-parse", "--git-dir"], cwd)
    }

    /// 获取远程仓库 URL
    ///
    /// 使用 `git remote get-url <remote>` 命令
    pub fn get_remote_url(remote: Option<&str>, cwd: Option<&Path>) -> Result<String> {
        let remote = remote.unwrap_or("origin");
        GitCommand::run(&["remote", "get-url", remote], cwd)
            .map_err(|e| match e {
                GitError::NotGitRepo => {
                    color_eyre::eyre::eyre!("Not in a Git repository")
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to get remote URL for: {}", remote))
    }

    /// 列出所有远程仓库
    ///
    /// 使用 `git remote` 命令
    pub fn list_remotes(cwd: Option<&Path>) -> Result<Vec<String>> {
        let output =
            GitCommand::run(&["remote"], cwd).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok(output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
    }

    /// 添加远程仓库
    ///
    /// 使用 `git remote add <name> <url>` 命令
    pub fn add_remote(name: &str, url: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["remote", "add", name, url], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to add remote: {} -> {}", name, url))
    }

    /// 删除远程仓库
    ///
    /// 使用 `git remote remove <name>` 命令
    pub fn remove_remote(name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["remote", "remove", name], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to remove remote: {}", name))
    }

    /// 从远程获取更新
    ///
    /// 使用 `git fetch <remote>` 命令
    pub fn fetch(remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        GitCommand::execute(&["fetch", remote], cwd)
            .map_err(|e| match e {
                GitError::AuthenticationFailed { reason } => {
                    color_eyre::eyre::eyre!("Authentication failed: {}", reason)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err_with(|| format!("Failed to fetch from remote: {}", remote))
    }

    /// 获取所有远程更新
    ///
    /// 使用 `git fetch --all` 命令
    pub fn fetch_all(cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["fetch", "--all"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to fetch from all remotes")
    }

    /// 清理已删除的远程分支引用
    ///
    /// 使用 `git remote prune <remote>` 命令
    pub fn prune_remote(remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        GitCommand::execute(&["remote", "prune", remote], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to prune remote: {}", remote))
    }

    /// 获取 Git 目录路径
    ///
    /// 使用 `git rev-parse --git-dir` 命令
    pub fn get_git_dir(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "--git-dir"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get Git directory")
    }

    /// 获取工作目录根路径
    ///
    /// 使用 `git rev-parse --show-toplevel` 命令
    pub fn get_workdir(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "--show-toplevel"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get work directory")
    }
}
