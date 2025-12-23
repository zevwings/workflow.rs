//! Git 仓库检测和类型识别
//!
//! 本模块提供了 Git 仓库相关的检测功能：
//! - 检测当前目录是否为 Git 仓库
//! - 检测远程仓库类型（GitHub 等）
//! - 获取远程仓库 URL

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use regex::Regex;

use super::types::RepoType;
use super::GitCommand;

/// Git 仓库管理
///
/// 提供仓库相关的操作功能，包括：
/// - 检测当前目录是否为 Git 仓库
/// - 检测远程仓库类型（GitHub 等）
/// - 获取远程仓库 URL
/// - 从远程获取更新
pub struct GitRepo;

impl GitRepo {
    /// 检查是否在 Git 仓库中
    ///
    /// 使用 `git rev-parse --git-dir` 检查当前目录是否为 Git 仓库。
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果当前目录是 Git 仓库，否则返回 `false`。
    pub fn is_git_repo() -> bool {
        GitCommand::new(["rev-parse", "--git-dir", "--quiet"]).quiet_success()
    }

    /// 检测远程仓库类型（GitHub）
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
    /// 支持识别 SSH Host 别名（如 `github-brainim`）。
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 返回对应的 `RepoType`：
    /// - 包含 `github.com` 或 host 以 `github` 开头 → `RepoType::GitHub`
    /// - 包含 `codeup.aliyun.com` → `RepoType::Codeup`（检测支持，但 PR 功能不支持）
    /// - 其他 → `RepoType::Unknown`
    fn parse_repo_type_from_url(url: &str) -> RepoType {
        // 检查 GitHub：包含 github.com 或 SSH host 以 github 开头（处理 SSH Host 别名，如 git@github-brainim:user/repo.git）
        if url.contains("github.com")
            || url.starts_with("git@github")
            || url.starts_with("ssh://git@github")
        {
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
    pub fn get_remote_url() -> Result<String> {
        GitCommand::new(["remote", "get-url", "origin"])
            .read()
            .wrap_err("Failed to get remote URL")
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
        GitCommand::new(["rev-parse", "--git-dir"])
            .read()
            .wrap_err("Failed to get git directory")
    }

    /// 从远程仓库获取更新
    ///
    /// 使用 `git fetch origin` 从远程仓库获取最新的分支和提交信息。
    ///
    /// # 错误
    ///
    /// 如果获取失败，返回相应的错误信息。
    pub fn fetch() -> Result<()> {
        GitCommand::new(["fetch", "origin"])
            .run()
            .wrap_err("Failed to fetch from origin")
    }

    /// 清理远程分支引用
    ///
    /// 使用 `git remote prune origin` 移除已删除的远程分支引用。
    ///
    /// # 错误
    ///
    /// 如果清理失败，返回相应的错误信息。
    pub fn prune_remote() -> Result<()> {
        GitCommand::new(["remote", "prune", "origin"])
            .run()
            .wrap_err("Failed to prune remote references")
    }

    /// 从 Git remote URL 提取仓库名（owner/repo 格式）
    ///
    /// 支持 GitHub 平台：
    /// - GitHub: git@github.com:owner/repo.git → owner/repo
    ///
    /// # 返回
    ///
    /// 返回 `owner/repo` 格式的仓库名。
    ///
    /// # 错误
    ///
    /// 如果无法从 URL 中提取仓库名，返回相应的错误信息。
    pub fn extract_repo_name() -> Result<String> {
        let url = Self::get_remote_url()?;
        Self::extract_repo_name_from_url(&url)
    }

    /// 从 URL 字符串提取仓库名
    ///
    /// 支持多种 URL 格式：
    /// - GitHub SSH: git@github.com:owner/repo.git
    /// - GitHub SSH (别名): git@github-brainim:owner/repo.git
    /// - GitHub HTTPS: https://github.com/owner/repo.git
    /// - Codeup SSH: git@codeup.aliyun.com:owner/repo.git
    /// - Codeup HTTPS: https://codeup.aliyun.com/owner/repo.git
    /// - Codeup HTTP: http://codeup.aliyun.com/owner/repo
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 返回 `owner/repo` 格式的仓库名。
    ///
    /// # 错误
    ///
    /// 如果无法从 URL 中提取仓库名，返回相应的错误信息。
    pub fn extract_repo_name_from_url(url: &str) -> Result<String> {
        // GitHub SSH 格式: git@github.com:owner/repo.git 或 git@github-xxx:owner/repo.git
        let github_ssh_re =
            Regex::new(r"git@github[^:]*:(.+?)(?:\.git)?$").wrap_err("Invalid regex pattern")?;
        if let Some(caps) = github_ssh_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| eyre!("Failed to extract repo name from GitHub SSH URL: {}", url))?
                .as_str()
                .to_string());
        }

        // GitHub HTTPS 格式: https://github.com/owner/repo.git
        let github_https_re = Regex::new(r"https?://(?:www\.)?github\.com/(.+?)(?:\.git)?/?$")
            .wrap_err("Invalid regex pattern")?;
        if let Some(caps) = github_https_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| eyre!("Failed to extract repo name from GitHub HTTPS URL: {}", url))?
                .as_str()
                .to_string());
        }

        // Codeup SSH 格式: git@codeup.aliyun.com:owner/repo.git
        let codeup_ssh_re = Regex::new(r"git@codeup\.aliyun\.com:(.+?)(?:\.git)?$")
            .wrap_err("Invalid regex pattern")?;
        if let Some(caps) = codeup_ssh_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| eyre!("Failed to extract repo name from Codeup SSH URL: {}", url))?
                .as_str()
                .to_string());
        }

        // Codeup HTTPS/HTTP 格式: https://codeup.aliyun.com/owner/repo.git 或 http://codeup.aliyun.com/owner/repo
        let codeup_https_re = Regex::new(r"https?://codeup\.aliyun\.com/(.+?)(?:\.git)?/?$")
            .wrap_err("Invalid regex pattern")?;
        if let Some(caps) = codeup_https_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| eyre!("Failed to extract repo name from Codeup HTTPS URL: {}", url))?
                .as_str()
                .to_string());
        }

        color_eyre::eyre::bail!("Failed to extract repo name from URL: {}", url)
    }
}
