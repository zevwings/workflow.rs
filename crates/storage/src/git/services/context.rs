//! Git 仓库上下文
//!
//! 提供 git2::Repository 的管理和访问。

use domain::git::{CodePlatform, GitError, RepoInfo};
use git2::{CertificateCheckStatus, Repository};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// URL 解析器模式
static URL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // GitHub SSH over 443: git@ssh.github.com:443/owner/repo.git
        Regex::new(r"git@ssh\.github\.com:\d+/(.+?)(?:\.git)?/?$").unwrap(),
        // GitHub SSH 协议格式（含 ssh.github.com）: ssh://git@ssh.github.com:443/owner/repo.git
        Regex::new(r"ssh://git@ssh\.github\.com(?::\d+)?/(.+?)(?:\.git)?/?$").unwrap(),
        // GitHub SSH 协议格式: ssh://git@github.com/owner/repo.git
        Regex::new(r"ssh://git@github[^/]*/(.+?)(?:\.git)?/?$").unwrap(),
        // GitHub SSH 格式: git@github.com:owner/repo.git (需在 ssh.github.com 之后，避免误匹配)
        Regex::new(r"git@github[^:]*:(.+?)(?:\.git)?$").unwrap(),
        // 兼容错误写法：git@github/owner/repo.git（缺少 .com 或误用 / 代替 :）
        Regex::new(r"git@[^/]+/(.+?)(?:\.git)?$").unwrap(),
        // GitHub HTTPS 格式: https://github.com/owner/repo.git
        Regex::new(r"https?://(?:www\.)?github\.com/(.+?)(?:\.git)?/?$").unwrap(),
        // Codeup SSH 格式: git@codeup.aliyun.com:owner/repo.git
        Regex::new(r"git@codeup\.aliyun\.com:(.+?)(?:\.git)?$").unwrap(),
        // Codeup HTTPS/HTTP 格式: https://codeup.aliyun.com/owner/repo.git
        Regex::new(r"https?://codeup\.aliyun\.com/(.+?)(?:\.git)?/?$").unwrap(),
        // 通用 HTTPS/HTTP 格式
        Regex::new(r"https?://[^/]+/(.+?)(?:\.git)?/?$").unwrap(),
    ]
});

/// 供容器注入的 Git 上下文持有者（返回 discover 得到的上下文）
pub trait GitContextHolder: Send + Sync {
    fn context(&self) -> GitContext;
}

/// 持有已发现的 Git 上下文，用于注册到容器
#[derive(Clone)]
pub struct DiscoveredContext(pub GitContext);

impl GitContextHolder for DiscoveredContext {
    fn context(&self) -> GitContext {
        self.0.clone()
    }
}

/// Git 仓库上下文
///
/// 管理 git2::Repository 实例，提供线程安全的仓库访问。
#[derive(Clone)]
pub struct GitContext {
    inner: Arc<GitContextInner>,
}

struct GitContextInner {
    repo: Mutex<Repository>,
    path: PathBuf,
}

// SAFETY: git2::Repository 内部使用 libgit2，
// 我们通过 Arc 确保只有一个所有者，通过不可变引用访问
unsafe impl Send for GitContextInner {}
unsafe impl Sync for GitContextInner {}

impl GitContext {
    /// 创建新的 Git 上下文
    pub fn new(repo: Repository) -> Self {
        let path = repo.workdir().map(Path::to_path_buf).unwrap_or_default();
        Self {
            inner: Arc::new(GitContextInner {
                repo: Mutex::new(repo),
                path,
            }),
        }
    }

    /// 从当前目录向上查找 Git 仓库
    pub fn discover() -> Result<Self, GitError> {
        let repo = Repository::discover(".").map_err(|_| GitError::NotGitRepo)?;
        Ok(Self::new(repo))
    }

    /// 初始化新的 Git 仓库
    #[cfg(any(test, feature = "testing"))]
    #[allow(dead_code)]
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, GitError> {
        let repo = Repository::init(&path).map_err(|e| {
            GitError::OperationFailed(format!(
                "Failed to initialize repository at {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;
        Ok(Self::new(repo))
    }

    /// 获取底层的 git2::Repository 引用
    pub fn repository(&self) -> std::sync::MutexGuard<'_, Repository> {
        self.inner.repo.lock().expect("Failed to lock repository")
    }

    /// 获取底层的 git2::Repository 可变引用
    ///
    /// 用于需要 `&mut Repository` 的操作（如 stash 操作）。
    pub fn repository_mut(&self) -> std::sync::MutexGuard<'_, Repository> {
        self.inner.repo.lock().expect("Failed to lock repository")
    }

    /// 获取仓库的工作目录路径
    pub fn workdir(&self) -> &Path {
        &self.inner.path
    }

    /// 检查是否为 bare 仓库
    pub fn is_bare(&self) -> bool {
        self.inner.repo.lock().expect("Failed to lock repository").is_bare()
    }

    /// 获取仓库信息
    pub fn info(&self) -> RepoInfo {
        let repo = self.inner.repo.lock().expect("Failed to lock repository");
        // 尝试获取 origin URL
        let origin_url = repo
            .find_remote("origin")
            .ok()
            .and_then(|remote| remote.url().map(String::from));

        // 根据 URL 检测仓库类型
        let kind = origin_url.as_ref().map(|url| Self::parse_repo_kind(url));

        // 获取 Git 目录
        let directory = repo.path().canonicalize().ok().and_then(|p| p.to_str().map(String::from));

        // 提取仓库名称
        let name = origin_url.as_ref().and_then(|url| Self::extract_repo_name(url));

        // 提取 owner
        let owner = name.as_ref().and_then(|n| {
            let parts: Vec<&str> = n.split('/').collect();
            if parts.len() >= 2 {
                Some(parts[0].to_string())
            } else {
                None
            }
        });

        RepoInfo {
            is_valid: true,
            kind,
            origin_url,
            directory,
            name,
            owner,
        }
    }

    /// 从 URL 提取仓库名称
    pub fn extract_repo_name(url: &str) -> Option<String> {
        for (idx, pattern) in URL_PATTERNS.iter().enumerate() {
            if let Some(caps) = pattern.captures(url) {
                if let Some(m) = caps.get(1) {
                    let repo_name = m.as_str().to_string();
                    // 对于通用 HTTPS URL（最后一个模式），提取最后两部分
                    if idx == URL_PATTERNS.len() - 1 {
                        let parts: Vec<&str> = repo_name.split('/').collect();
                        if parts.len() >= 2 {
                            let owner = parts[parts.len() - 2];
                            let repo = parts[parts.len() - 1];
                            return Some(format!("{}/{}", owner, repo));
                        }
                    }
                    return Some(repo_name);
                }
            }
        }
        None
    }

    /// 从 URL 解析仓库类型
    pub fn parse_repo_kind(url: &str) -> CodePlatform {
        if url.contains("github.com")
            || url.contains("ssh.github.com")
            || url.starts_with("git@github")
            || url.starts_with("git@ssh.github")
            || url.starts_with("ssh://git@github")
            || url.starts_with("ssh://git@ssh.github")
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

    // ========== 辅助方法 ==========

    /// 创建认证回调
    ///
    /// 用于远程操作（push/pull/fetch）的认证。
    /// 优先使用 SSH agent，然后尝试默认凭据。
    pub fn create_callbacks<'a>() -> git2::RemoteCallbacks<'a> {
        let mut callbacks = git2::RemoteCallbacks::new();
        // libgit2 使用 libssh2，不读 OpenSSH 的 known_hosts，必须显式接受主机密钥
        callbacks.certificate_check(|_cert, host| {
            toolkit::log_info!(
                "create_callbacks: certificate_check invoked, host = {}",
                host
            );
            Ok(CertificateCheckStatus::CertificateOk)
        });

        // 使用 Arc<Mutex> 来跟踪重试次数，避免无限循环
        let retry_count = Arc::new(Mutex::new(0u32));

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            // 检查重试次数，防止无限循环
            let mut count = retry_count.lock().expect("retry_count lock poisoned");
            *count += 1;

            const MAX_RETRIES: u32 = 3;
            if *count > MAX_RETRIES {
                toolkit::log_info!(
                    "create_callbacks: max retries ({}) exceeded, failing authentication",
                    MAX_RETRIES
                );
                return Err(git2::Error::from_str(
                    "Authentication failed after maximum retry attempts"
                ));
            }

            toolkit::log_info!(
                "create_callbacks: credentials invoked (attempt {}), url = {}, allowed_types = {:?}",
                *count,
                url,
                allowed_types
            );

            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                let username = username_from_url.unwrap_or("git");
                toolkit::log_info!("create_callbacks: trying ssh_key_from_agent(username = {})", username);
                match git2::Cred::ssh_key_from_agent(username) {
                    Ok(cred) => {
                        toolkit::log_info!("create_callbacks: ssh_key_from_agent ok");
                        return Ok(cred);
                    }
                    Err(e) => {
                        toolkit::log_info!("create_callbacks: ssh_key_from_agent failed: {}", e);
                    }
                }
            }
            if allowed_types.contains(git2::CredentialType::DEFAULT) {
                toolkit::log_info!("create_callbacks: trying Cred::default()");
                return git2::Cred::default();
            }
            toolkit::log_info!("create_callbacks: no authentication available");
            Err(git2::Error::from_str("no authentication available"))
        });
        callbacks.push_transfer_progress(|current, total, bytes| {
            toolkit::log_info!(
                "create_callbacks: push_transfer_progress current={} total={} bytes={}",
                current,
                total,
                bytes
            );
        });
        callbacks
    }

    /// 获取签名
    ///
    /// 优先使用仓库配置，如果失败则使用默认值。
    pub fn get_signature(&self) -> Result<git2::Signature<'static>, GitError> {
        self.inner
            .repo
            .lock()
            .expect("Failed to lock repository")
            .signature()
            .or_else(|_| git2::Signature::now("User", "user@example.com"))
            .map_err(|e| GitError::SignatureError(e.to_string()))
    }

    /// 解析引用到 commit
    ///
    /// 支持分支名、tag 名、SHA 等各种引用格式。
    pub fn resolve_commit(&self, reference: &str) -> Result<git2::Oid, GitError> {
        let repo = self.inner.repo.lock().expect("Failed to lock repository");
        let obj = repo
            .revparse_single(reference)
            .map_err(|_| GitError::InvalidReference(reference.to_string()))?;
        let commit = obj.peel_to_commit().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        Ok(commit.id())
    }

    /// 获取 HEAD 指向的 commit
    pub fn head_commit(&self) -> Result<git2::Oid, GitError> {
        let repo = self.inner.repo.lock().expect("Failed to lock repository");
        let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let commit = head.peel_to_commit().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        Ok(commit.id())
    }

    /// 获取分支的 commit
    ///
    /// # 参数
    /// - `name`: 分支名称
    /// - `branch_type`: 分支类型（本地或远程）
    ///
    /// # 返回
    /// 分支指向的 commit ID
    #[allow(dead_code)]
    pub fn get_branch_commit(
        &self,
        name: &str,
        branch_type: git2::BranchType,
    ) -> Result<git2::Oid, GitError> {
        let repo = self.inner.repo.lock().expect("Failed to lock repository");
        let branch = repo
            .find_branch(name, branch_type)
            .map_err(|_| GitError::BranchNotFound(name.to_string()))?;
        let commit = branch
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        Ok(commit.id())
    }

    /// 确保 HEAD 指向分支（非 detached 状态）
    ///
    /// # 返回
    /// 当前分支名称
    ///
    /// # 错误
    /// 如果 HEAD 处于 detached 状态，返回错误
    #[allow(dead_code)]
    pub fn ensure_head_is_branch(&self) -> Result<String, GitError> {
        let repo = self.inner.repo.lock().expect("Failed to lock repository");
        let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        if !head.is_branch() {
            return Err(GitError::OperationFailed(
                "HEAD is in detached state".into(),
            ));
        }
        head.shorthand()
            .map(String::from)
            .ok_or_else(|| GitError::OperationFailed("Invalid branch name".into()))
    }

    /// 读取 .gitignore 文件并提取目录模式
    ///
    /// 解析 .gitignore 文件，提取其中的目录模式（以 / 结尾或常见的构建/缓存目录）。
    /// 这些模式可用于在 git 操作中提前过滤，避免扫描大型目录。
    ///
    /// # 返回
    /// 返回目录模式列表，例如 ["target", "node_modules", "dist"]
    ///
    /// # 注意
    /// - 如果 .gitignore 不存在，返回空列表
    /// - 只提取目录模式，不包括文件模式
    /// - 自动添加一些常见的大型目录（如果 .gitignore 中没有）
    pub fn get_ignore_directory_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // 读取 .gitignore 文件
        let gitignore_path = self.workdir().join(".gitignore");
        if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
            for line in content.lines() {
                let line = line.trim();

                // 跳过空行和注释
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // 移除前导的 / 和尾部的 /
                let pattern = line.trim_start_matches('/').trim_end_matches('/');

                // 跳过包含通配符的复杂模式（如 *.log, **/*.pyc）
                if pattern.contains('*') || pattern.contains('?') {
                    continue;
                }

                // 跳过否定模式（以 ! 开头）
                if pattern.starts_with('!') {
                    continue;
                }

                // 只保留目录模式（原始行以 / 结尾，或不包含文件扩展名）
                if line.ends_with('/') || !pattern.contains('.') {
                    patterns.push(pattern.to_string());
                }
            }
        }

        // 添加一些常见的大型目录（如果 .gitignore 中没有）
        // 这些目录通常会导致性能问题
        let common_large_dirs = [
            "target",       // Rust
            "node_modules", // Node.js
            "dist",         // Build output
            "build",        // Build output
            ".next",        // Next.js
            ".nuxt",        // Nuxt.js
            "coverage",     // Test coverage
            ".cache",       // Cache
            "tmp",          // Temporary files
            "vendor",       // Go/PHP dependencies
            ".git",         // Git internal
        ];

        for dir in &common_large_dirs {
            if !patterns.contains(&dir.to_string()) {
                patterns.push(dir.to_string());
            }
        }

        patterns
    }
}

impl std::fmt::Debug for GitContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitContext")
            .field("path", &self.inner.path)
            .field("is_bare", &self.is_bare())
            .finish()
    }
}
