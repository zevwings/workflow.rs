//! Git 仓库检测和类型识别
//!
//! 本模块提供了 Git 仓库相关的检测功能：
//! - 检测当前目录是否为 Git 仓库
//! - 检测远程仓库类型（GitHub 等）
//! - 获取远程仓库 URL

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use regex::Regex;
use std::path::Path;

use super::helpers::open_repo;
use super::types::RepoType;

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
    /// 使用 git2 库检查当前目录是否为 Git 仓库。
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果当前目录是 Git 仓库，否则返回 `false`。
    pub fn is_git_repo() -> bool {
        git2::Repository::open(".").is_ok()
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
    /// 使用 git2 库获取远程仓库的 URL。
    ///
    /// # 返回
    ///
    /// 返回远程仓库的 URL 字符串。
    ///
    /// # 错误
    ///
    /// 如果无法获取远程 URL，返回相应的错误信息。
    pub fn get_remote_url() -> Result<String> {
        let repo = open_repo()?;
        let remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        remote
            .url()
            .ok_or_else(|| color_eyre::eyre::eyre!("Remote 'origin' has no URL"))
            .map(|url| url.to_string())
    }

    /// 获取远程仓库 URL（指定仓库路径）
    ///
    /// 使用 git2 库获取指定仓库的远程 URL。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// 返回远程仓库的 URL 字符串。
    ///
    /// # 错误
    ///
    /// 如果无法获取远程 URL，返回相应的错误信息。
    pub fn get_remote_url_in(repo_path: impl AsRef<std::path::Path>) -> Result<String> {
        let repo = git2::Repository::open(repo_path.as_ref())
            .wrap_err_with(|| format!("Failed to open repository at: {:?}", repo_path.as_ref()))?;
        let remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        remote
            .url()
            .ok_or_else(|| color_eyre::eyre::eyre!("Remote 'origin' has no URL"))
            .map(|url| url.to_string())
    }

    /// 获取 Git 目录路径
    ///
    /// 使用 git2 库获取 `.git` 目录的路径。
    ///
    /// # 返回
    ///
    /// 返回 `.git` 目录的路径字符串。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或操作失败，返回相应的错误信息。
    pub(crate) fn get_git_dir() -> Result<String> {
        let repo = open_repo()?;
        let git_dir = repo.path();
        git_dir
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Git directory path is not valid UTF-8"))
            .map(|s| s.to_string())
    }

    /// 从远程仓库获取更新
    ///
    /// 使用 git2 库从远程仓库获取最新的分支和提交信息。
    /// 支持 SSH 和 HTTPS 认证，适用于私有仓库。
    ///
    /// # 错误
    ///
    /// 如果获取失败，返回相应的错误信息。
    pub fn fetch() -> Result<()> {
        use super::helpers::open_repo;
        use super::GitAuth;
        use git2::FetchOptions;

        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 获取远程更新
        // 使用空数组表示获取所有默认的 refspecs
        let refs: &[&str] = &[];
        remote
            .fetch(refs, Some(&mut fetch_options), None)
            .wrap_err("Failed to fetch from origin")?;

        Ok(())
    }

    /// 清理远程分支引用
    ///
    /// 使用 git2 库移除已删除的远程分支引用。
    /// 通过获取远程引用列表，然后删除本地不存在的远程引用。
    ///
    /// # 错误
    ///
    /// 如果清理失败，返回相应的错误信息。
    pub fn prune_remote() -> Result<()> {
        let repo = open_repo()?;

        // 先获取远程更新，确保远程引用是最新的
        Self::fetch()?;

        // 获取远程引用列表
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 连接远程并获取引用列表
        let callbacks = super::GitAuth::get_remote_callbacks();
        remote
            .connect_auth(git2::Direction::Fetch, Some(callbacks), None)
            .wrap_err("Failed to connect to remote")?;

        // 获取远程引用列表
        let remote_refs = remote.list().wrap_err("Failed to list remote references")?;

        // 构建远程引用名称集合
        let mut remote_ref_names = std::collections::HashSet::new();
        for remote_ref in remote_refs {
            remote_ref_names.insert(remote_ref.name().to_string());
        }

        // 遍历本地所有远程引用（refs/remotes/origin/*）
        let local_remote_refs: Vec<String> = repo
            .references()?
            .filter_map(|reference| {
                reference.ok().and_then(|ref_| {
                    ref_.name()
                        .and_then(|name| name.strip_prefix("refs/remotes/origin/"))
                        .map(|name| name.to_string())
                })
            })
            .collect();

        // 删除本地存在但远程不存在的引用
        let mut deleted_count = 0;
        for local_ref_name in local_remote_refs {
            let remote_ref_name = format!("refs/heads/{}", local_ref_name);
            if !remote_ref_names.contains(&remote_ref_name) {
                // 远程引用不存在，删除本地引用
                let ref_name = format!("refs/remotes/origin/{}", local_ref_name);
                if let Ok(mut reference) = repo.find_reference(&ref_name) {
                    reference
                        .delete()
                        .wrap_err_with(|| format!("Failed to delete reference: {}", ref_name))?;
                    deleted_count += 1;
                }
            }
        }

        if deleted_count > 0 {
            crate::log_info!("Pruned {} stale remote reference(s)", deleted_count);
        }

        Ok(())
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
        Self::extract_repo_name_in(
            std::env::current_dir().wrap_err("Failed to get current directory")?,
        )
    }

    /// 从 Git remote URL 提取仓库名（指定仓库路径）
    ///
    /// 支持 GitHub 平台：
    /// - GitHub: git@github.com:owner/repo.git → owner/repo
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// 返回 `owner/repo` 格式的仓库名。
    ///
    /// # 错误
    ///
    /// 如果无法从 URL 中提取仓库名，返回相应的错误信息。
    pub fn extract_repo_name_in(repo_path: impl AsRef<Path>) -> Result<String> {
        let url = Self::get_remote_url_in(repo_path)?;
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
