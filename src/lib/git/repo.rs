use anyhow::{Context, Result};
use duct::cmd;

use super::commit::Git;
use super::types::RepoType;

impl Git {
    /// 检查是否在 Git 仓库中
    pub fn is_git_repo() -> bool {
        cmd("git", &["rev-parse", "--git-dir", "--quiet"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok()
    }

    /// 检测远程仓库类型（GitHub 或 Codeup）
    pub fn detect_repo_type() -> Result<RepoType> {
        let url = Self::get_remote_url()?;
        Ok(Self::parse_repo_type_from_url(&url))
    }

    /// 从 URL 解析仓库类型
    fn parse_repo_type_from_url(url: &str) -> RepoType {
        if url.contains("github.com") {
            RepoType::GitHub
        } else if url.contains("codeup.aliyun.com") {
            RepoType::Codeup
        } else {
            RepoType::Unknown
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

    /// 获取 Git 目录路径
    ///
    /// 返回 `.git` 目录的路径
    pub(crate) fn get_git_dir() -> Result<String> {
        let output = cmd("git", &["rev-parse", "--git-dir"])
            .read()
            .context("Failed to get git directory")?;

        Ok(output.trim().to_string())
    }
}

