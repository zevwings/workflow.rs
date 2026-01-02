//! Git Cherry-pick 操作命令封装
//!
//! 提供 cherry-pick 相关的所有 Git 命令操作，包括：
//! - Cherry-pick 提交（cherry_pick）
//! - 继续 cherry-pick（continue）
//! - 中止 cherry-pick（abort）
//! - 检查 cherry-pick 状态（check_status）

use crate::git::commands::command::GitCommand;
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git Cherry-pick 命令操作
pub struct GitCherryPickCommand;

impl GitCherryPickCommand {
    /// Cherry-pick 提交
    ///
    /// 使用 `git cherry-pick <commit>` 命令
    pub fn cherry_pick(commit_sha: &str, no_commit: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["cherry-pick"];

        if no_commit {
            args.push("--no-commit");
        }

        args.push(commit_sha);

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::handle_cherry_pick_error)
            .wrap_err_with(|| format!("Failed to cherry-pick commit: {}", commit_sha))
    }

    /// 继续 cherry-pick
    ///
    /// 使用 `git cherry-pick --continue` 命令
    pub fn continue_cherry_pick(cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["cherry-pick", "--continue"], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to continue cherry-pick")
    }

    /// 中止 cherry-pick
    ///
    /// 使用 `git cherry-pick --abort` 命令
    pub fn abort_cherry_pick(cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["cherry-pick", "--abort"], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to abort cherry-pick")
    }

    /// 检查 cherry-pick 状态
    ///
    /// 检查 `.git/CHERRY_PICK_HEAD` 文件是否存在
    pub fn check_status(cwd: Option<&Path>) -> Result<bool> {
        let cwd = cwd.unwrap_or_else(|| Path::new("."));

        // 如果 .git 是文件（worktree），需要解析它
        let git_dir = if cwd.join(".git").is_file() {
            // 读取 .git 文件内容获取实际 git 目录路径
            if let Ok(content) = std::fs::read_to_string(cwd.join(".git")) {
                if let Some(path) = content.strip_prefix("gitdir: ") {
                    Path::new(path.trim()).to_path_buf()
                } else {
                    cwd.join(".git")
                }
            } else {
                cwd.join(".git")
            }
        } else {
            cwd.join(".git")
        };

        let cherry_pick_head = git_dir.join("CHERRY_PICK_HEAD");
        Ok(cherry_pick_head.exists())
    }
}
