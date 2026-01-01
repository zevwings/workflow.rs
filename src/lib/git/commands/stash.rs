//! Git Stash 操作命令封装
//!
//! 提供 stash 相关的所有 Git 命令操作，包括：
//! - Stash 保存（stash_push）
//! - Stash 恢复（stash_pop、stash_apply）
//! - Stash 列表（list_stash）
//! - Stash 删除（drop_stash）

use crate::git::commands::{GitCommand, GitError};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Stash 条目
#[derive(Debug, Clone)]
pub struct StashEntry {
    pub index: usize,
    pub branch: String,
    pub message: String,
    pub commit_hash: String,
}

/// Git Stash 命令操作
pub struct GitStashCommand;

impl GitStashCommand {
    /// 保存未提交的修改到 stash
    ///
    /// 使用 `git stash push -m <message>` 命令
    pub fn stash_push(message: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["stash", "push"];

        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to stash changes")
    }

    /// 恢复 stash（会删除 stash）
    ///
    /// 使用 `git stash pop` 命令
    pub fn stash_pop(stash_index: Option<usize>, cwd: Option<&Path>) -> Result<()> {
        let mut args: Vec<&str> = vec!["stash", "pop"];
        let stash_ref;

        if let Some(index) = stash_index {
            stash_ref = format!("stash@{{{}}}", index);
            args.push(&stash_ref);
        }

        GitCommand::execute(&args, cwd)
            .map_err(|e| match e {
                GitError::StashConflict { details } => {
                    color_eyre::eyre::eyre!("Stash pop conflict:\n{}", details)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err("Failed to pop stash")
    }

    /// 应用 stash（不删除 stash）
    ///
    /// 使用 `git stash apply` 命令
    pub fn stash_apply(stash_index: Option<usize>, cwd: Option<&Path>) -> Result<()> {
        let mut args: Vec<&str> = vec!["stash", "apply"];
        let stash_ref;

        if let Some(index) = stash_index {
            stash_ref = format!("stash@{{{}}}", index);
            args.push(&stash_ref);
        }

        GitCommand::execute(&args, cwd)
            .map_err(|e| match e {
                GitError::StashConflict { details } => {
                    color_eyre::eyre::eyre!("Stash apply conflict:\n{}", details)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err("Failed to apply stash")
    }

    /// 列出所有 stash
    ///
    /// 使用 `git stash list` 命令
    pub fn list_stash(cwd: Option<&Path>) -> Result<Vec<StashEntry>> {
        let output = GitCommand::run(&["stash", "list"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        let mut entries = Vec::new();
        for (index, line) in output.lines().enumerate() {
            // 解析格式: stash@{0}: WIP on branch: message
            // 或: stash@{0}: On branch: message
            if let Some(entry) = Self::parse_stash_line(line, index, cwd) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// 解析 stash 列表行
    fn parse_stash_line(line: &str, index: usize, cwd: Option<&Path>) -> Option<StashEntry> {
        // 格式: stash@{0}: WIP on branch: message
        // 提取分支和消息
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() < 2 {
            return None;
        }

        let message_part = parts[1].trim();
        let (branch, message) = if let Some(rest) = message_part.strip_prefix("WIP on ") {
            if let Some(colon_pos) = rest.find(':') {
                (rest[..colon_pos].trim(), rest[colon_pos + 1..].trim())
            } else {
                (rest, "")
            }
        } else if let Some(rest) = message_part.strip_prefix("On ") {
            if let Some(colon_pos) = rest.find(':') {
                (rest[..colon_pos].trim(), rest[colon_pos + 1..].trim())
            } else {
                (rest, "")
            }
        } else {
            ("", message_part)
        };

        // 获取 commit hash
        let commit_hash = GitCommand::run(&["rev-parse", &format!("stash@{{{}}}", index)], cwd)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        Some(StashEntry {
            index,
            branch: branch.to_string(),
            message: message.to_string(),
            commit_hash,
        })
    }

    /// 删除 stash
    ///
    /// 使用 `git stash drop` 命令
    pub fn drop_stash(stash_index: Option<usize>, cwd: Option<&Path>) -> Result<()> {
        let mut args: Vec<&str> = vec!["stash", "drop"];
        let stash_ref;

        if let Some(index) = stash_index {
            stash_ref = format!("stash@{{{}}}", index);
            args.push(&stash_ref);
        }

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to drop stash")
    }

    /// 检查是否有冲突
    ///
    /// 使用 `git diff --check` 命令
    pub fn check_conflicts(cwd: Option<&Path>) -> Result<bool> {
        let output = GitCommand::run(&["diff", "--check"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok(!output.trim().is_empty())
    }
}
