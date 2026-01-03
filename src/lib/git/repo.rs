//! Git 仓库检测和类型识别
//!
//! 本模块提供了 Git 仓库相关的检测功能：
//! - 检测当前目录是否为 Git 仓库
//! - 检测远程仓库类型（GitHub 等）
//! - 获取远程仓库 URL

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use regex::Regex;
use std::path::Path;

use super::types::RepoType;
use super::GitRepository;
use crate::base::resilience::{
    default_download_timeout, execute_with_timeout_and_retry, RetryConfig, TimeoutConfig,
};

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
    /// 使用 GitRepository 封装层检查当前目录是否为 Git 仓库。
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果当前目录是 Git 仓库，否则返回 `false`。
    pub fn is_git_repo() -> bool {
        GitRepository::open().is_ok()
    }

    /// 检查指定路径是否为 Git 仓库
    ///
    /// 使用 GitRepository 封装层检查指定路径是否为 Git 仓库。
    ///
    /// # 参数
    ///
    /// * `path` - 要检查的路径
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果指定路径是 Git 仓库，否则返回 `false`。
    pub fn is_git_repo_at(path: impl AsRef<Path>) -> bool {
        GitRepository::open_at(path).is_ok()
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
    /// 使用 GitCommand 获取远程仓库的 URL。
    ///
    /// # 返回
    ///
    /// 返回远程仓库的 URL 字符串。
    ///
    /// # 错误
    ///
    /// 如果无法获取远程 URL，返回相应的错误信息。
    pub fn get_remote_url() -> Result<String> {
        let mut repo = GitRepository::open()?;
        let remote = repo.find_origin_remote()?;

        remote.url()
    }

    /// 获取远程仓库 URL（指定仓库路径）
    ///
    /// 使用 GitCommand 获取指定仓库的远程 URL。
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
        let mut repo = GitRepository::open_at(repo_path)?;
        let remote = repo.find_origin_remote()?;

        remote.url()
    }

    /// 获取 Git 目录路径
    ///
    /// 使用 `git rev-parse --git-dir` 命令获取 `.git` 目录的路径。
    /// 对于标准仓库，返回 `.git` 目录的绝对路径。
    /// 对于 worktree 和 submodule，git 命令会自动处理正确的路径。
    ///
    /// # 返回
    ///
    /// 返回 `.git` 目录的路径字符串（绝对路径）。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或操作失败，返回相应的错误信息。
    pub(crate) fn get_git_dir() -> Result<String> {
        use super::commands::repo::GitRepoCommand;

        // 使用 git rev-parse --git-dir 获取 .git 目录路径
        let git_dir = GitRepoCommand::get_git_dir(None)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get Git directory")?;

        // 转换为绝对路径
        let git_dir_path = Path::new(&git_dir);
        let absolute_path = if git_dir_path.is_absolute() {
            git_dir_path.to_path_buf()
        } else {
            // 如果是相对路径，基于当前工作目录转换为绝对路径
            std::env::current_dir()
                .wrap_err("Failed to get current directory")?
                .join(git_dir_path)
        };

        // 规范化路径（解析符号链接等）
        let canonical_path = absolute_path.canonicalize().unwrap_or(absolute_path);

        canonical_path
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Git directory path is not valid UTF-8"))
            .map(|s| s.to_string())
    }

    /// 从远程仓库获取更新
    ///
    /// 使用 GitCommand 从远程仓库获取最新的分支和提交信息。
    /// 支持 SSH 和 HTTPS 认证，适用于私有仓库。
    /// 包含超时和重试机制，提高网络操作的可靠性。
    ///
    /// # 错误
    ///
    /// 如果获取失败，返回相应的错误信息。
    pub fn fetch() -> Result<()> {
        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();
        let retry_config = RetryConfig::platform_default();

        execute_with_timeout_and_retry(
            timeout_config,
            retry_config,
            || -> Result<()> {
                let mut repo = GitRepository::open()?;
                let mut remote = repo.find_origin_remote()?;

                // 获取远程更新
                // 使用空数组表示获取所有默认的 refspecs
                remote.fetch(&[]).wrap_err("Failed to fetch from origin")?;

                Ok(())
            },
            "Fetching from remote",
        )?;
        Ok(())
    }

    /// 清理远程分支引用
    ///
    /// 使用 GitCommand 移除已删除的远程分支引用。
    /// 通过获取远程引用列表，然后删除本地不存在的远程引用。
    /// 包含超时保护，防止网络操作卡住。
    ///
    /// # 错误
    ///
    /// 如果清理失败，返回相应的错误信息。
    pub fn prune_remote() -> Result<()> {
        // 先获取远程更新，确保远程引用是最新的（已有超时/重试保护）
        Self::fetch()?;

        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();

        // 保护远程操作
        execute_with_timeout_and_retry(
            timeout_config,
            RetryConfig::platform_default(),
            || -> Result<()> {
                let mut repo = GitRepository::open()?;
                let remote = repo.find_origin_remote()?;

                // 获取远程引用列表
                let remote_refs = remote.list().wrap_err("Failed to list remote references")?;

                // 构建远程引用名称集合（只包含分支引用）
                let mut remote_ref_names = std::collections::HashSet::new();
                for (ref_name, _sha) in remote_refs {
                    if ref_name.starts_with("refs/heads/") {
                        remote_ref_names.insert(ref_name);
                    }
                }

                // 获取本地所有远程引用（refs/remotes/origin/*）
                use crate::git::commands::GitRepoCommand;
                let output =
                    GitRepoCommand::for_each_ref("refs/remotes/origin/", Some(repo.path()))
                        .wrap_err("Failed to list local remote references")?;

                let local_remote_refs: Vec<String> = output
                    .iter()
                    .filter_map(|line| {
                        line.strip_prefix("refs/remotes/origin/").map(|name| name.to_string())
                    })
                    .collect();

                // 删除本地存在但远程不存在的引用
                let mut deleted_count = 0;
                for local_ref_name in local_remote_refs {
                    let remote_ref_name = format!("refs/heads/{}", local_ref_name);
                    if !remote_ref_names.contains(&remote_ref_name) {
                        // 远程引用不存在，删除本地引用
                        let ref_name = format!("refs/remotes/origin/{}", local_ref_name);
                        if GitRepoCommand::ref_exists(&ref_name, Some(repo.path())) {
                            GitRepoCommand::delete_ref(&ref_name, Some(repo.path()))
                                .wrap_err_with(|| {
                                    format!("Failed to delete reference: {}", ref_name)
                                })?;
                            deleted_count += 1;
                        }
                    }
                }

                if deleted_count > 0 {
                    crate::log_info!("Pruned {} stale remote reference(s)", deleted_count);
                }

                Ok(())
            },
            "Pruning remote references",
        )?;
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
    /// - GitHub SSH 协议: ssh://git@github.com/owner/repo.git
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
        // GitHub SSH 协议格式: ssh://git@github.com/owner/repo.git 或 ssh://git@github-xxx/owner/repo.git
        // 支持 SSH host 别名（如 ssh://git@github-brainim/owner/repo.git）
        let github_ssh_proto_re = Regex::new(r"ssh://git@github[^/]*/(.+?)(?:\.git)?/?$")
            .wrap_err("Invalid regex pattern")?;
        if let Some(caps) = github_ssh_proto_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| {
                    eyre!(
                        "Failed to extract repo name from GitHub SSH protocol URL: {}",
                        url
                    )
                })?
                .as_str()
                .to_string());
        }

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
