//! Git Tag 操作命令封装
//!
//! 提供 tag 相关的所有 Git 命令操作，包括：
//! - Tag 查询（列出本地/远程 tag、检查存在性）
//! - Tag 操作（创建、删除、推送）

use crate::git::commands::{GitCommand, GitError};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git Tag 命令操作
pub struct GitTagCommand;

impl GitTagCommand {
    /// 列出所有本地 tag
    ///
    /// 使用 `git tag` 命令
    pub fn list_local_tags(cwd: Option<&Path>) -> Result<Vec<String>> {
        let output =
            GitCommand::run(&["tag"], cwd).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        let mut tags: Vec<String> =
            output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        tags.sort();
        Ok(tags)
    }

    /// 列出所有远程 tag
    ///
    /// 使用 `git ls-remote --tags` 命令
    pub fn list_remote_tags(remote: Option<&str>, cwd: Option<&Path>) -> Result<Vec<String>> {
        let remote = remote.unwrap_or("origin");
        let output = GitCommand::run(&["ls-remote", "--tags", remote], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        let mut tags = Vec::new();
        for line in output.lines() {
            // 格式: <commit_hash>	refs/tags/<tag_name>
            // 或者: <commit_hash>	refs/tags/<tag_name>^{} (peeled tag)
            if let Some(ref_part) = line.split_whitespace().nth(1) {
                if let Some(tag_ref) = ref_part.strip_prefix("refs/tags/") {
                    // 移除 ^{} 后缀（peeled tag）
                    let tag_name = tag_ref.strip_suffix("^{}").unwrap_or(tag_ref);
                    if !tags.contains(&tag_name.to_string()) {
                        tags.push(tag_name.to_string());
                    }
                }
            }
        }

        tags.sort();
        Ok(tags)
    }

    /// 检查 tag 是否存在（本地）
    ///
    /// 使用 `git show-ref --verify` 命令
    pub fn tag_exists_local(tag_name: &str, cwd: Option<&Path>) -> Result<bool> {
        Ok(GitCommand::check(
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/tags/{}", tag_name),
            ],
            cwd,
        ))
    }

    /// 检查 tag 是否存在（远程）
    ///
    /// 使用 `git ls-remote` 命令
    pub fn tag_exists_remote(
        tag_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<bool> {
        let remote = remote.unwrap_or("origin");
        let output = GitCommand::run(
            &[
                "ls-remote",
                "--tags",
                remote,
                &format!("refs/tags/{}", tag_name),
            ],
            cwd,
        )
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        Ok(!output.trim().is_empty())
    }

    /// 检查 tag 是否存在（本地和远程）
    ///
    /// 返回 `(本地存在, 远程存在)`
    pub fn tag_exists(
        tag_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<(bool, bool)> {
        let exists_local = Self::tag_exists_local(tag_name, cwd)?;
        let exists_remote = Self::tag_exists_remote(tag_name, remote, cwd)?;
        Ok((exists_local, exists_remote))
    }

    /// 创建 tag
    ///
    /// 使用 `git tag <name>` 命令（创建 lightweight tag）
    pub fn create_tag(tag_name: &str, commit_sha: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["tag"];

        if let Some(sha) = commit_sha {
            args.push(sha);
        }

        args.push(tag_name);

        GitCommand::execute(&args, cwd)
            .map_err(|e| match e {
                GitError::BranchAlreadyExists { branch } => {
                    color_eyre::eyre::eyre!("Tag '{}' already exists", branch)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err_with(|| format!("Failed to create tag: {}", tag_name))
    }

    /// 创建带注释的 tag
    ///
    /// 使用 `git tag -a -m <message> <name>` 命令
    pub fn create_annotated_tag(
        tag_name: &str,
        message: &str,
        commit_sha: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let mut args = vec!["tag", "-a", "-m", message];

        if let Some(sha) = commit_sha {
            args.push(sha);
        }

        args.push(tag_name);

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to create annotated tag: {}", tag_name))
    }

    /// 删除本地 tag
    ///
    /// 使用 `git tag -d <name>` 命令
    pub fn delete_local(tag_name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["tag", "-d", tag_name], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to delete local tag: {}", tag_name))
    }

    /// 删除远程 tag
    ///
    /// 使用 `git push <remote> :refs/tags/<name>` 命令
    pub fn delete_remote(tag_name: &str, remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        GitCommand::execute(&["push", remote, &format!(":refs/tags/{}", tag_name)], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to delete remote tag: {}", tag_name))
    }

    /// 推送 tag 到远程
    ///
    /// 使用 `git push <remote> <tag>` 命令
    pub fn push_tag(tag_name: &str, remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        GitCommand::execute(&["push", remote, tag_name], cwd)
            .map_err(|e| match e {
                GitError::AuthenticationFailed { reason } => {
                    color_eyre::eyre::eyre!("Authentication failed: {}", reason)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .wrap_err_with(|| format!("Failed to push tag {} to {}", tag_name, remote))
    }

    /// 推送所有 tag 到远程
    ///
    /// 使用 `git push <remote> --tags` 命令
    pub fn push_all_tags(remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or("origin");
        GitCommand::execute(&["push", remote, "--tags"], cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to push all tags to {}", remote))
    }

    /// 获取 tag 指向的 commit hash
    ///
    /// 使用 `git rev-parse` 命令
    pub fn get_tag_commit(tag_name: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", tag_name], cwd)
            .map_err(|e| match e {
                GitError::CommitNotFound { .. } => {
                    color_eyre::eyre::eyre!("Tag '{}' does not exist", tag_name)
                }
                _ => color_eyre::eyre::eyre!("{}", e),
            })
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to get commit hash for tag: {}", tag_name))
    }
}
