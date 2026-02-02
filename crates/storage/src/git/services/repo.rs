//! 仓库业务逻辑服务
//!
//! 提供仓库相关的业务逻辑实现，包括：
//! - URL 解析和提取
//! - 仓库类型识别

use domain::git::error::GitError;
use domain::git::{CodePlatform, RepoInfo};
use gix::{refs::FullName, Repository};
use once_cell::sync::Lazy;
use regex::Regex;

/// URL 解析器模式定义
struct UrlPattern {
    regex: Regex,
}

/// 预编译的正则表达式模式
static URL_PATTERNS: Lazy<Vec<UrlPattern>> = Lazy::new(|| {
    vec![
        // GitHub SSH 协议格式: ssh://git@github.com/owner/repo.git
        UrlPattern {
            regex: Regex::new(r"ssh://git@github[^/]*/(.+?)(?:\.git)?/?$")
                .expect("Invalid regex pattern for GitHub SSH protocol"),
        },
        // GitHub SSH 格式: git@github.com:owner/repo.git
        UrlPattern {
            regex: Regex::new(r"git@github[^:]*:(.+?)(?:\.git)?$")
                .expect("Invalid regex pattern for GitHub SSH"),
        },
        // GitHub HTTPS 格式: https://github.com/owner/repo.git
        UrlPattern {
            regex: Regex::new(r"https?://(?:www\.)?github\.com/(.+?)(?:\.git)?/?$")
                .expect("Invalid regex pattern for GitHub HTTPS"),
        },
        // Codeup SSH 格式: git@codeup.aliyun.com:owner/repo.git
        UrlPattern {
            regex: Regex::new(r"git@codeup\.aliyun\.com:(.+?)(?:\.git)?$")
                .expect("Invalid regex pattern for Codeup SSH"),
        },
        // Codeup HTTPS/HTTP 格式: https://codeup.aliyun.com/owner/repo.git
        UrlPattern {
            regex: Regex::new(r"https?://codeup\.aliyun\.com/(.+?)(?:\.git)?/?$")
                .expect("Invalid regex pattern for Codeup HTTPS"),
        },
        // 通用 HTTPS/HTTP 格式: https://domain.com/path/to/repo.git
        // 提取路径部分（去除开头的 / 和结尾的 .git）
        // 对于多段路径，提取最后两部分作为 owner/repo
        UrlPattern {
            regex: Regex::new(r"https?://[^/]+/(.+?)(?:\.git)?/?$")
                .expect("Invalid regex pattern for generic HTTPS"),
        },
    ]
});

/// 仓库服务接口
pub trait RepoService: Send + Sync {
    /// 打开仓库
    ///
    /// 根据当前目录打开 Git 仓库。
    ///
    /// # 返回
    ///
    /// 返回打开的 `Repository` 对象。
    ///
    /// # 错误
    ///
    /// 如果无法发现或打开仓库，返回相应的错误。
    fn open_repo(&self) -> Result<Repository, GitError>;

    /// 从 gix Repository 获取指定远程的 URL
    ///
    /// 内部辅助方法，从已打开的仓库中获取指定远程的 URL。
    ///
    /// # 参数
    ///
    /// * `repo` - 已打开的仓库
    /// * `remote_name` - 远程名称（如 "origin", "upstream"）
    ///
    /// # 返回
    ///
    /// 返回远程 URL 字符串。
    fn find_remote_url_from_repo(
        &self,
        repo: &Repository,
        remote_name: &str,
    ) -> Result<String, GitError>;

    /// 查找引用
    ///
    /// 从已打开的仓库中查找指定名称的引用。
    ///
    /// # 参数
    ///
    /// * `repo` - 已打开的仓库
    /// * `ref_name` - 引用名称（如 "refs/heads/main", "refs/remotes/origin/main"）
    ///
    /// # 返回
    ///
    /// 返回找到的引用对象。
    ///
    /// # 错误
    ///
    /// 如果找不到指定引用，返回相应的错误。
    fn find_reference<'a>(
        &self,
        repo: &'a Repository,
        ref_name: &str,
    ) -> Result<gix::Reference<'a>, GitError>;

    /// 从 URL 中提取仓库名
    ///
    /// 支持多种 Git 托管平台的 URL 格式：
    /// - GitHub SSH 协议: ssh://git@github.com/owner/repo.git
    /// - GitHub SSH: git@github.com:owner/repo.git
    /// - GitHub SSH (别名): git@github-brainim:owner/repo.git
    /// - GitHub HTTPS: https://github.com/owner/repo.git
    /// - Codeup SSH: git@codeup.aliyun.com:owner/repo.git
    /// - Codeup HTTPS: https://codeup.aliyun.com/owner/repo.git
    /// - 通用 HTTPS: https://domain.com/path/to/repo.git (提取最后两部分作为 owner/repo)
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 如果成功提取，返回 `owner/repo` 格式的仓库名，否则返回 `None`。
    fn extract_repo_name_from_url(&self, url: &str) -> Option<String>;

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
    /// - 包含 `codeup.aliyun.com` → `RepoType::Codeup`
    /// - 其他 → `RepoType::Unknown`
    fn parse_repo_type_from_url(&self, url: &str) -> CodePlatform;

    /// 规范化 Git 目录路径
    ///
    /// 处理相对路径、符号链接解析等，返回规范化的绝对路径。
    fn normalize_git_dir_path(&self, repo: &Repository) -> Result<String, GitError>;

    /// 解析仓库字符串为 owner 和 repo_name
    ///
    /// # 参数
    ///
    /// * `repo` - `owner/repo` 格式的仓库字符串
    ///
    /// # 返回
    ///
    /// 返回 `(owner, repo_name)` 元组
    fn parse_repo_name(&self, repo: &str) -> Result<(String, String), GitError>;

    /// 获取仓库信息
    ///
    /// 一次性获取仓库的所有基本信息，包括：
    /// - 是否为 Git 仓库
    /// - 仓库类型（GitHub、Codeup、Unknown）
    /// - origin 远程仓库 URL
    /// - Git 目录路径
    /// - 仓库名称（owner/repo 格式）
    /// - 仓库所有者（owner）
    ///
    /// # 参数
    ///
    /// * `repo` - 已打开的仓库
    /// * `is_git_repo` - 是否为 Git 仓库
    ///
    /// # 返回
    ///
    /// 返回包含所有仓库信息的 `RepoInfo` 结构体。
    fn get_repo_info(&self, repo: &Repository, is_git_repo: bool) -> domain::git::RepoInfo;
}

/// 仓库业务逻辑服务实现
pub struct RepoServiceImpl;

impl RepoServiceImpl {
    /// 创建新的仓库服务实例
    pub fn new() -> Self {
        Self
    }

    /// 尝试使用正则表达式从 URL 中提取仓库名
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    /// * `pattern` - URL 模式
    ///
    /// # 返回
    ///
    /// 如果匹配成功，返回 `owner/repo` 格式的仓库名，否则返回 `None`。
    fn try_extract_with_pattern(url: &str, pattern: &UrlPattern) -> Option<String> {
        pattern
            .regex
            .captures(url)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
    }
}

impl RepoService for RepoServiceImpl {
    fn open_repo(&self) -> Result<Repository, GitError> {
        gix::discover(".").map_err(|e| {
            GitError::OperationFailed(format!("Failed to discover Git repository: {}", e))
        })
    }

    fn find_remote_url_from_repo(
        &self,
        repo: &Repository,
        remote_name: &str,
    ) -> Result<String, GitError> {
        let remote = repo.find_remote(remote_name).map_err(|e| {
            GitError::OperationFailed(format!("Failed to find remote '{}': {}", remote_name, e))
        })?;

        let url = remote.url(gix::remote::Direction::Fetch).ok_or_else(|| {
            GitError::OperationFailed(format!("Failed to get URL from remote '{}'", remote_name))
        })?;

        // 使用 Display trait 转换为字符串（会处理非 UTF-8 字符）
        Ok(url.to_string())
    }

    fn find_reference<'a>(
        &self,
        repo: &'a Repository,
        ref_name: &str,
    ) -> Result<gix::Reference<'a>, GitError> {
        let full_name = FullName::try_from(ref_name).map_err(|e| {
            GitError::OperationFailed(format!("Invalid reference name '{}': {}", ref_name, e))
        })?;

        repo.find_reference(&full_name).map_err(|e| {
            GitError::OperationFailed(format!("Failed to find reference '{}': {}", ref_name, e))
        })
    }

    fn extract_repo_name_from_url(&self, url: &str) -> Option<String> {
        // 尝试使用预编译的正则表达式模式匹配
        for (idx, pattern) in URL_PATTERNS.iter().enumerate() {
            if let Some(repo_name) = Self::try_extract_with_pattern(url, pattern) {
                // 对于通用 HTTPS URL（最后一个模式），如果路径有多段，提取最后两部分
                if idx == URL_PATTERNS.len() - 1 {
                    // 这是通用 HTTPS 模式，提取的可能是多段路径
                    let parts: Vec<&str> = repo_name.split('/').collect();
                    if parts.len() >= 2 {
                        // 提取最后两部分作为 owner/repo
                        let owner = parts[parts.len() - 2];
                        let repo = parts[parts.len() - 1];
                        return Some(format!("{}/{}", owner, repo));
                    } else if parts.len() == 1 {
                        // 只有一段，使用默认 owner 或直接返回
                        return Some(repo_name);
                    }
                }
                return Some(repo_name);
            }
        }
        None
    }

    fn parse_repo_type_from_url(&self, url: &str) -> CodePlatform {
        // 检查 GitHub：包含 github.com 或 SSH host 以 github 开头（处理 SSH Host 别名，如 git@github-brainim:user/repo.git）
        if url.contains("github.com")
            || url.starts_with("git@github")
            || url.starts_with("ssh://git@github")
        {
            CodePlatform::GitHub
        } else if url.contains("cnb.cool") || url.starts_with("git@cnb.cool:") {
            CodePlatform::CNB
        } else if url.contains("codeup.aliyun.com") {
            CodePlatform::Codeup
        } else {
            CodePlatform::Unknown
        }
    }

    fn normalize_git_dir_path(&self, repo: &Repository) -> Result<String, GitError> {
        let git_dir = repo.git_dir();
        let absolute_path = if git_dir.is_absolute() {
            git_dir.to_path_buf()
        } else {
            // 如果是相对路径，基于当前工作目录转换为绝对路径
            std::env::current_dir()
                .map_err(|e| {
                    GitError::OperationFailed(format!("Failed to get current directory: {}", e))
                })?
                .join(git_dir)
        };

        // 规范化路径（解析符号链接等）
        let canonical_path = absolute_path.canonicalize().unwrap_or(absolute_path);

        canonical_path
            .to_str()
            .ok_or_else(|| {
                GitError::OperationFailed("Git directory path is not valid UTF-8".to_string())
            })
            .map(|s| s.to_string())
    }

    fn parse_repo_name(&self, repo: &str) -> Result<(String, String), GitError> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            return Err(GitError::OperationFailed(format!(
                "Invalid repo format: {}",
                repo
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    fn get_repo_info(&self, repo: &Repository, is_git_repo: bool) -> RepoInfo {
        // 如果不是 Git 仓库，直接返回基本信息
        if !is_git_repo {
            return RepoInfo {
                is_valid: false,
                kind: None,
                origin_url: None,
                directory: None,
                name: None,
                owner: None,
            };
        }

        // 尝试获取 origin URL
        let origin_url = self.find_remote_url_from_repo(repo, "origin").ok();

        // 根据 URL 检测仓库类型
        let repo_type = origin_url
            .as_ref()
            .map(|url| self.parse_repo_type_from_url(url));

        // 尝试获取 Git 目录
        let git_dir = self.normalize_git_dir_path(repo).ok();

        // 尝试提取仓库名
        let repo_name = origin_url
            .as_ref()
            .and_then(|url| self.extract_repo_name_from_url(url));

        // 从仓库名中提取 owner
        let owner = repo_name
            .as_ref()
            .and_then(|name| self.parse_repo_name(name).ok())
            .map(|(owner, _)| owner);

        RepoInfo {
            is_valid: true,
            kind: repo_type,
            origin_url,
            directory: git_dir,
            name: repo_name,
            owner,
        }
    }
}
