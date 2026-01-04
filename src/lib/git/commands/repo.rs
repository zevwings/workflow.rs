//! Git 仓库操作命令封装
//!
//! 提供仓库相关的所有 Git 命令操作，包括：
//! - 仓库检测（是否为 Git 仓库）
//! - 远程仓库操作（获取 URL、获取更新）

use crate::git::commands::command::GitCommand;
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
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        GitCommand::run(&["remote", "get-url", remote], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to get remote URL for: {}", remote))
    }

    /// 列出所有远程仓库
    ///
    /// 使用 `git remote` 命令
    pub fn list_remotes(cwd: Option<&Path>) -> Result<Vec<String>> {
        let output =
            GitCommand::run(&["remote"], cwd).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 添加远程仓库
    ///
    /// 使用 `git remote add <name> <url>` 命令
    pub fn add_remote(name: &str, url: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["remote", "add", name, url], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to add remote: {} -> {}", name, url))
    }

    /// 删除远程仓库
    ///
    /// 使用 `git remote remove <name>` 命令
    pub fn remove_remote(name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["remote", "remove", name], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to remove remote: {}", name))
    }

    /// 从远程获取更新
    ///
    /// 使用 `git fetch <remote>` 命令
    pub fn fetch(remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        GitCommand::execute(&["fetch", remote], cwd)
            .map_err(GitCommand::handle_auth_error)
            .wrap_err_with(|| format!("Failed to fetch from remote: {}", remote))
    }

    /// 获取所有远程更新
    ///
    /// 使用 `git fetch --all` 命令
    pub fn fetch_all(cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["fetch", "--all"], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to fetch from all remotes")
    }

    /// 清理已删除的远程分支引用
    ///
    /// 使用 `git remote prune <remote>` 命令
    pub fn prune_remote(remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        GitCommand::execute(&["remote", "prune", remote], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to prune remote: {}", remote))
    }

    /// 获取 Git 目录路径
    ///
    /// 使用 `git rev-parse --git-dir` 命令
    pub fn get_git_dir(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "--git-dir"], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get Git directory")
    }

    /// 获取工作目录根路径
    ///
    /// 使用 `git rev-parse --show-toplevel` 命令
    pub fn get_workdir(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "--show-toplevel"], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get work directory")
    }

    /// 列出所有引用
    ///
    /// 使用 `git for-each-ref --format=%(refname) <pattern>` 命令
    pub fn for_each_ref(pattern: &str, cwd: Option<&Path>) -> Result<Vec<String>> {
        let output = GitCommand::run(&["for-each-ref", "--format=%(refname)", pattern], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 删除引用
    ///
    /// 使用 `git update-ref -d <ref>` 命令
    pub fn delete_ref(reference: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["update-ref", "-d", reference], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to delete ref: {}", reference))
    }

    /// 检查引用是否存在
    ///
    /// 使用 `git rev-parse --verify <ref>` 命令
    pub fn ref_exists(reference: &str, cwd: Option<&Path>) -> bool {
        GitCommand::check(&["rev-parse", "--verify", reference], cwd)
    }

    /// 初始化仓库
    ///
    /// 使用 `git init [-b <branch>]` 命令
    pub fn init(initial_branch: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["init"];
        if let Some(branch) = initial_branch {
            args.push("-b");
            args.push(branch);
        }
        GitCommand::execute(&args, cwd)
            .map_err(|e| {
                // 将 GitError 转换为详细的错误消息，确保包含 Git 命令的 stderr
                let error_msg = format!("{}", e);
                color_eyre::eyre::eyre!("Failed to initialize repository: {}", error_msg)
            })
    }

    /// 设置远程仓库 URL
    ///
    /// 使用 `git remote set-url <name> <url>` 命令
    pub fn set_remote_url(name: &str, url: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["remote", "set-url", name, url], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to set remote URL: {} -> {}", name, url))
    }

    /// 列出远程引用
    ///
    /// 使用 `git ls-remote <remote>` 命令
    pub fn ls_remote(remote: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["ls-remote", remote], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to list remote refs: {}", remote))
    }
}
