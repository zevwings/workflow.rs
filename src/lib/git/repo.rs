use anyhow::{Context, Result};
use duct::cmd;

use super::types::RepoType;

/// 检查是否在 Git 仓库中
pub fn is_git_repo() -> bool {
    cmd("git", &["rev-parse", "--git-dir"]).read().is_ok()
}

/// 检测远程仓库类型（GitHub 或 Codeup）
pub fn detect_repo_type() -> Result<RepoType> {
    let output = cmd("git", &["remote", "get-url", "origin"])
        .read()
        .context("Failed to get remote URL")?;

    let url = output.trim();
    if url.contains("github.com") {
        Ok(RepoType::GitHub)
    } else if url.contains("codeup.aliyun.com") {
        Ok(RepoType::Codeup)
    } else {
        Ok(RepoType::Unknown)
    }
}

/// 获取远程仓库 URL
#[allow(dead_code)]
pub fn get_remote_url() -> Result<String> {
    let output = cmd("git", &["remote", "get-url", "origin"])
        .read()
        .context("Failed to get remote URL")?;

    Ok(output.trim().to_string())
}

/// 检查工作区是否有未提交的更改
#[allow(dead_code)]
pub fn has_uncommitted_changes() -> Result<bool> {
    let output = cmd("git", &["status", "--porcelain"])
        .read()
        .context("Failed to check git status")?;

    Ok(!output.trim().is_empty())
}
