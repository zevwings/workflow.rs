//! Git 仓库检测和类型识别
//!
//! 本模块提供了 Git 仓库相关的检测功能：
//! - 检测当前目录是否为 Git 仓库
//! - 检测远程仓库类型（GitHub、Codeup 等）
//! - 获取远程仓库 URL

use anyhow::{Context, Result};
use duct::cmd;

use super::commit::Git;
use super::types::RepoType;

impl Git {
    /// 检查是否在 Git 仓库中
    ///
    /// 使用 `git rev-parse --git-dir` 检查当前目录是否为 Git 仓库。
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果当前目录是 Git 仓库，否则返回 `false`。
    pub fn is_git_repo() -> bool {
        cmd("git", &["rev-parse", "--git-dir", "--quiet"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok()
    }

    /// 检测远程仓库类型（GitHub 或 Codeup）
    ///
    /// 通过解析远程仓库 URL 来识别仓库类型。
    ///
    /// # 返回
    ///
    /// 返回 `RepoType` 枚举值，表示仓库类型。
    ///
    /// # 错误
    ///
    /// 如果无法获取远程 URL，返回相应的错误信息。
    pub fn detect_repo_type() -> Result<RepoType> {
        let url = Self::get_remote_url()?;
        Ok(Self::parse_repo_type_from_url(&url))
    }

    /// 从 URL 解析仓库类型
    ///
    /// 通过检查 URL 中是否包含特定域名来识别仓库类型。
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 返回对应的 `RepoType`：
    /// - 包含 `github.com` → `RepoType::GitHub`
    /// - 包含 `codeup.aliyun.com` → `RepoType::Codeup`
    /// - 其他 → `RepoType::Unknown`
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
    ///
    /// 使用 `git remote get-url origin` 获取远程仓库的 URL。
    ///
    /// # 返回
    ///
    /// 返回远程仓库的 URL 字符串（去除首尾空白）。
    ///
    /// # 错误
    ///
    /// 如果无法获取远程 URL，返回相应的错误信息。
    #[allow(dead_code)]
    pub fn get_remote_url() -> Result<String> {
        let output = cmd("git", &["remote", "get-url", "origin"])
            .read()
            .context("Failed to get remote URL")?;

        Ok(output.trim().to_string())
    }

    /// 获取 Git 目录路径
    ///
    /// 使用 `git rev-parse --git-dir` 获取 `.git` 目录的路径。
    ///
    /// # 返回
    ///
    /// 返回 `.git` 目录的路径字符串（去除首尾空白）。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或命令执行失败，返回相应的错误信息。
    pub(crate) fn get_git_dir() -> Result<String> {
        let output = cmd("git", &["rev-parse", "--git-dir"])
            .read()
            .context("Failed to get git directory")?;

        Ok(output.trim().to_string())
    }
}

