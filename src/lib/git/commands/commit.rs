//! Git 提交操作命令封装
//!
//! 提供提交相关的所有 Git 命令操作，包括：
//! - 状态检查（status、has_changes）
//! - 暂存操作（add、add_all）
//! - 提交操作（commit、amend）
//! - 提交信息（get_commit_info、get_diff）

use crate::git::commands::GitCommand;
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git 提交命令操作
pub struct GitCommitCommand;

impl GitCommitCommand {
    /// 检查 Git 状态
    ///
    /// 使用 `git status --porcelain` 命令
    pub fn status(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["status", "--porcelain"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get git status")
    }

    /// 检查是否有未提交的更改
    ///
    /// 使用 `git status --porcelain` 命令
    pub fn has_changes(cwd: Option<&Path>) -> Result<bool> {
        let output = Self::status(cwd)?;
        Ok(!output.trim().is_empty())
    }

    /// 暂存文件
    ///
    /// 使用 `git add <file>` 命令
    pub fn add(file: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["add", file], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to stage file: {}", file))
    }

    /// 暂存所有文件
    ///
    /// 使用 `git add .` 命令
    pub fn add_all(cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["add", "."], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to stage all files")
    }

    /// 创建提交
    ///
    /// 使用 `git commit -m <message>` 命令
    pub fn commit(message: &str, no_verify: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["commit", "-m", message];

        if no_verify {
            args.push("--no-verify");
        }

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to commit: {}", message))
    }

    /// 修改最后一次提交
    ///
    /// 使用 `git commit --amend` 命令
    pub fn amend(message: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["commit", "--amend"];

        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        } else {
            args.push("--no-edit");
        }

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to amend commit")
    }

    /// 获取当前 HEAD 的 SHA
    ///
    /// 使用 `git rev-parse HEAD` 命令
    pub fn get_head_sha(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "HEAD"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get HEAD SHA")
    }

    /// 获取提交信息
    ///
    /// 使用 `git log --format` 命令
    ///
    /// # 返回
    ///
    /// 返回 `(消息, 作者, 日期)` 元组
    pub fn get_commit_info(
        commit_sha: &str,
        cwd: Option<&Path>,
    ) -> Result<(String, String, String)> {
        // 获取提交消息
        let message = GitCommand::run(&["log", "-1", "--format=%s", commit_sha], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        // 获取作者
        let author = GitCommand::run(&["log", "-1", "--format=%an <%ae>", commit_sha], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        // 获取日期
        let date = GitCommand::run(&["log", "-1", "--format=%ai", commit_sha], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok((
            message.trim().to_string(),
            author.trim().to_string(),
            date.trim().to_string(),
        ))
    }

    /// 获取差异内容
    ///
    /// 使用 `git diff` 命令
    pub fn get_diff(staged: bool, cwd: Option<&Path>) -> Result<String> {
        let mut args = vec!["diff"];

        if staged {
            args.push("--cached");
        }

        GitCommand::run(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get diff")
    }
}
