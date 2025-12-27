//! Git 远程操作认证回调
//!
//! 提供统一的认证回调机制，支持 SSH 和 HTTPS 两种认证方式。
//! 使用智能检测和缓存机制，自动选择最合适的认证方式。

use git2::{Cred, CredentialType, RemoteCallbacks};
use std::path::PathBuf;
use std::sync::OnceLock;

/// 缓存的认证信息
///
/// 在程序运行期间缓存认证相关的信息，避免重复查找。
struct CachedAuthInfo {
    /// SSH 密钥文件路径（如果找到）
    ssh_key_path: Option<PathBuf>,
    /// HTTPS token（从环境变量读取）
    https_token: Option<String>,
    /// HTTPS 用户名（从环境变量读取）
    https_username: Option<String>,
}

/// Git 远程操作认证管理
///
/// 提供统一的认证回调机制，支持 SSH 和 HTTPS 两种认证方式。
/// 使用智能检测和缓存机制，自动选择最合适的认证方式。
pub struct GitAuth;

impl GitAuth {
    /// 创建远程操作认证回调
    ///
    /// 每次调用都会创建一个新的 `RemoteCallbacks` 对象，但使用缓存的认证信息。
    /// 这样可以避免重复查找 SSH 密钥和环境变量。
    ///
    /// 根据远程 URL 类型（SSH/HTTPS）智能选择认证方式：
    ///
    /// **SSH 认证优先级：**
    /// 1. SSH Agent（如果可用）
    /// 2. 缓存的 SSH 密钥文件（通过 SSH config 或 GitHub 账号匹配找到）
    ///
    /// **HTTPS 认证优先级：**
    /// 1. 环境变量 `GITHUB_TOKEN` 或 `GIT_TOKEN`
    /// 2. 环境变量 `GIT_USERNAME`（如果设置了）
    ///
    /// # 返回
    ///
    /// 返回新创建的 `RemoteCallbacks` 对象，可以用于 Push、Fetch 等操作。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitAuth;
    /// use git2::PushOptions;
    ///
    /// let mut callbacks = GitAuth::get_remote_callbacks();
    /// let mut push_options = PushOptions::new();
    /// push_options.remote_callbacks(callbacks);
    /// ```
    pub fn get_remote_callbacks() -> RemoteCallbacks<'static> {
        Self::create_remote_callbacks()
    }

    /// 创建远程操作认证回调
    ///
    /// 根据远程 URL 类型（SSH/HTTPS）智能选择认证方式。
    ///
    /// # 返回
    ///
    /// 返回配置好的 `RemoteCallbacks` 对象，可以用于 Push、Fetch 等操作。
    ///
    /// # 错误处理
    ///
    /// 如果所有认证方式都失败，返回详细的错误信息，指导用户如何配置认证。
    fn create_remote_callbacks() -> RemoteCallbacks<'static> {
        let mut callbacks = RemoteCallbacks::new();
        let auth_info = Self::get_cached_auth_info();

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            // 1. 检测 URL 类型
            let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");
            let is_https = url.starts_with("https://");

            // 2. SSH 认证
            if is_ssh && allowed_types.contains(CredentialType::SSH_KEY) {
                let username = username_from_url.unwrap_or("git");

                // 优先尝试 SSH agent（最方便，适合开发环境）
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }

                // 回退到缓存的 SSH 密钥文件
                if let Some(ref key_path) = auth_info.ssh_key_path {
                    match Cred::ssh_key(username, None, key_path, None) {
                        Ok(cred) => return Ok(cred),
                        Err(e) => {
                            // 从传入的 URL 提取 host
                            let host = Self::extract_host_from_url(url)
                                .unwrap_or_else(|| "github.com".to_string());

                            return Err(git2::Error::from_str(&format!(
                                "SSH authentication failed: {}\n\
                                \n\
                                Remote URL: {}\n\
                                Host: {}\n\
                                \n\
                                Configuration Options:\n\
                                \n\
                                Option 1: Configure SSH config (Recommended)\n\
                                Add to ~/.ssh/config:\n\
                                \n\
                                  Host {}\n\
                                    HostName {}\n\
                                    User git\n\
                                    IdentityFile {:?}\n\
                                    IdentitiesOnly yes\n\
                                \n\
                                Then update remote URL:\n\
                                  git remote set-url origin git@{}:owner/repo.git\n\
                                \n\
                                Option 2: Use SSH agent\n\
                                  1. Add SSH key to agent:\n\
                                     ssh-add {:?}\n\
                                  2. Verify key is added:\n\
                                     ssh-add -l\n\
                                  3. Test connection:\n\
                                     ssh -T git@{}\n\
                                \n\
                                Option 3: Use HTTPS with token\n\
                                  1. Switch to HTTPS URL:\n\
                                     git remote set-url origin https://github.com/owner/repo.git\n\
                                  2. Set GitHub token:\n\
                                     export GITHUB_TOKEN=your_token_here\n\
                                \n\
                                Troubleshooting:\n\
                                  - Check key permissions: chmod 600 {:?}\n\
                                  - Verify SSH config: cat ~/.ssh/config\n\
                                  - Test SSH connection: ssh -T git@{}\n\
                                  - Check Git config: git config user.email",
                                e,
                                url,
                                host,
                                host,
                                host,
                                key_path,
                                host,
                                key_path,
                                host,
                                key_path,
                                host
                            )));
                        }
                    }
                }

                // 所有 SSH 认证方式都失败
                // 从传入的 URL 提取 host
                let host = Self::extract_host_from_url(url)
                    .unwrap_or_else(|| "github.com".to_string());

                return Err(git2::Error::from_str(&format!(
                    "No SSH key found for remote: {}\n\
                    \n\
                    Configuration Options:\n\
                    \n\
                    Option 1: Configure SSH config (Recommended)\n\
                    Add to ~/.ssh/config:\n\
                    \n\
                      Host {}\n\
                        HostName {}\n\
                        User git\n\
                        IdentityFile ~/.ssh/id_ed25519\n\
                        IdentitiesOnly yes\n\
                    \n\
                    Then update remote URL:\n\
                      git remote set-url origin git@{}:owner/repo.git\n\
                    \n\
                    Option 2: Use SSH agent\n\
                      1. Generate SSH key (if not exists):\n\
                         ssh-keygen -t ed25519 -C \"your_email@example.com\"\n\
                      2. Add SSH key to agent:\n\
                         ssh-add ~/.ssh/id_ed25519\n\
                      3. Verify key is added:\n\
                         ssh-add -l\n\
                      4. Test connection:\n\
                         ssh -T git@{}\n\
                    \n\
                    Option 3: Use HTTPS with token\n\
                      1. Switch to HTTPS URL:\n\
                         git remote set-url origin https://github.com/owner/repo.git\n\
                      2. Set GitHub token:\n\
                         export GITHUB_TOKEN=your_token_here\n\
                    \n\
                    Troubleshooting:\n\
                      - Check if SSH key exists: ls -la ~/.ssh/id_*\n\
                      - Verify SSH config: cat ~/.ssh/config\n\
                      - Test SSH connection: ssh -T git@{}\n\
                      - Check Git config: git config user.email",
                    url,
                    host,
                    host,
                    host,
                    host,
                    host
                )));
            }

            // 3. HTTPS 认证
            if is_https && allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                let username = auth_info
                    .https_username
                    .as_ref()
                    .map(|s| s.as_str())
                    .or(username_from_url)
                    .or_else(|| Self::extract_username_from_url(url))
                    .unwrap_or("git")
                    .to_string();

                // 使用缓存的 HTTPS token
                if let Some(ref token) = auth_info.https_token {
                    return Cred::userpass_plaintext(&username, token);
                }

                // HTTPS 认证失败
                return Err(git2::Error::from_str(
                    "No HTTPS credentials found. Please set one of:\n\
                    1. GITHUB_TOKEN environment variable\n\
                    2. GIT_TOKEN environment variable"
                ));
            }

            // 不支持的认证类型
            Err(git2::Error::from_str(&format!(
                "Unsupported credential type for URL: {}. \
                Supported types: SSH_KEY, USER_PASS_PLAINTEXT",
                url
            )))
        });

        callbacks
    }

    /// 获取缓存的认证信息
    ///
    /// 使用 `OnceLock` 实现单例模式，在程序运行期间只初始化一次。
    ///
    /// # 返回
    ///
    /// 返回缓存的认证信息。
    fn get_cached_auth_info() -> &'static CachedAuthInfo {
        static AUTH_INFO: OnceLock<CachedAuthInfo> = OnceLock::new();

        AUTH_INFO.get_or_init(|| {
            // 初始化时读取一次认证信息
            let ssh_key_path = Self::find_ssh_key();
            let https_token = std::env::var("GITHUB_TOKEN")
                .or_else(|_| std::env::var("GIT_TOKEN"))
                .ok();
            let https_username = std::env::var("GIT_USERNAME").ok();

            CachedAuthInfo {
                ssh_key_path,
                https_token,
                https_username,
            }
        })
    }

    /// 查找 SSH 密钥文件（混合方案）
    ///
    /// 按优先级顺序查找 SSH 密钥：
    /// 1. SSH config 匹配（如果远程 URL 匹配 SSH config 中的 Host）
    /// 2. GitHub 账号匹配（通过 Git 配置的 user.email 匹配 GitHub 账号，然后查找对应的 SSH config Host）
    /// 3. SSH agent 中的密钥（在认证回调中处理）
    ///
    /// 如果以上方式都失败，将返回 `None`，并在认证回调中提供详细的配置指导。
    ///
    /// # 返回
    ///
    /// 返回找到的第一个存在的密钥文件路径，如果都不存在则返回 `None`。
    fn find_ssh_key() -> Option<PathBuf> {
        // 优先级 1: 尝试从 SSH config 获取
        if let Ok(remote_url) = crate::git::GitRepo::get_remote_url() {
            if let Some(key_path) = Self::get_ssh_key_from_config(&remote_url) {
                return Some(key_path);
            }
        }

        // 优先级 2: 通过 Git 配置 + GitHub API 匹配
        if let Some(key_path) = Self::get_ssh_key_from_github_account() {
            return Some(key_path);
        }

        // 优先级 3: SSH agent（在认证回调中处理，这里跳过）

        // 如果前三个优先级都失败，返回 None，让认证回调提供详细的配置指导
        None
    }

    /// 从 SSH config 获取密钥路径
    ///
    /// 解析 `~/.ssh/config` 文件，根据远程 URL 匹配对应的 Host 配置。
    ///
    /// 匹配规则：
    /// 1. 精确匹配：从 URL 提取 host，查找 `Host <host>` 配置
    /// 2. HostName 匹配：查找 `HostName` 与 URL host 匹配的配置
    ///
    /// # 参数
    ///
    /// * `remote_url` - 远程仓库 URL（如 `git@github.com:user/repo.git`）
    ///
    /// # 返回
    ///
    /// 返回匹配的 `IdentityFile` 路径，如果未找到则返回 `None`。
    fn get_ssh_key_from_config(remote_url: &str) -> Option<PathBuf> {
        // 从 URL 提取 host
        let host = Self::extract_host_from_url(remote_url)?;

        // 解析 SSH config
        let config_path = Self::get_ssh_config_path()?;
        let config_content = std::fs::read_to_string(&config_path).ok()?;

        // 解析配置并查找匹配的 Host
        Self::parse_ssh_config_and_match(&config_content, &host)
    }

    /// 从 URL 提取 host
    ///
    /// 支持多种 URL 格式：
    /// - `git@github.com:user/repo.git` → `github.com`
    /// - `git@github:user/repo.git` → `github`
    /// - `ssh://git@github.com/user/repo.git` → `github.com`
    /// - `https://github.com/user/repo.git` → `github.com`
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 返回提取的 host，如果无法提取则返回 `None`。
    fn extract_host_from_url(url: &str) -> Option<String> {
        // SSH 格式: git@host:path 或 ssh://git@host/path
        if let Some(start) = url.find('@') {
            let after_at = &url[start + 1..];
            if let Some(end) = after_at.find(':') {
                return Some(after_at[..end].to_string());
            }
            if let Some(end) = after_at.find('/') {
                return Some(after_at[..end].to_string());
            }
        }

        // HTTPS 格式: https://host/path
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                return Some(after_protocol[..end].to_string());
            }
        }

        None
    }

    /// 获取 SSH config 文件路径
    ///
    /// 返回 `~/.ssh/config` 的完整路径。
    fn get_ssh_config_path() -> Option<PathBuf> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;

        let config_path = PathBuf::from(&home_dir).join(".ssh").join("config");
        if config_path.exists() {
            Some(config_path)
        } else {
            None
        }
    }

    /// 解析 SSH config 并匹配 Host
    ///
    /// 解析 SSH config 文件内容，查找匹配指定 host 的配置。
    ///
    /// 匹配规则：
    /// 1. 精确匹配：`Host <host>` 与目标 host 完全匹配
    /// 2. HostName 匹配：`HostName <hostname>` 与目标 host 匹配
    ///
    /// # 参数
    ///
    /// * `config_content` - SSH config 文件内容
    /// * `target_host` - 要匹配的目标 host
    ///
    /// # 返回
    ///
    /// 返回匹配的 `IdentityFile` 路径，如果未找到则返回 `None`。
    fn parse_ssh_config_and_match(config_content: &str, target_host: &str) -> Option<PathBuf> {
        let mut current_hosts: Vec<String> = Vec::new();
        let mut current_hostname: Option<String> = None;
        let mut current_identity_file: Option<PathBuf> = None;
        let mut matched_identity_file: Option<PathBuf> = None;

        for line in config_content.lines() {
            let line = line.trim();

            // 跳过注释和空行
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 解析指令
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "Host" => {
                    // 保存之前的配置（如果有匹配）
                    if Self::is_host_match(&current_hosts, current_hostname.as_deref(), target_host)
                    {
                        if let Some(ref identity_file) = current_identity_file {
                            matched_identity_file = Some(identity_file.clone());
                        }
                    }

                    // 开始新的 Host 块
                    current_hosts = parts[1..].iter().map(|s| s.to_string()).collect();
                    current_hostname = None;
                    current_identity_file = None;
                }
                "HostName" => {
                    if parts.len() > 1 {
                        current_hostname = Some(parts[1].to_string());
                    }
                }
                "IdentityFile" => {
                    if parts.len() > 1 {
                        let identity_file = parts[1];
                        // 展开 ~ 为 home 目录
                        let expanded_path = if identity_file.starts_with('~') {
                            if let Some(home_dir) = std::env::var("HOME")
                                .or_else(|_| std::env::var("USERPROFILE"))
                                .ok()
                            {
                                PathBuf::from(&home_dir)
                                    .join(&identity_file[2..]) // 跳过 "~/"
                            } else {
                                PathBuf::from(identity_file)
                            }
                        } else {
                            PathBuf::from(identity_file)
                        };

                        if expanded_path.exists() {
                            current_identity_file = Some(expanded_path);
                        }
                    }
                }
                _ => {
                    // 其他指令，忽略
                }
            }
        }

        // 检查最后一个 Host 块
        if Self::is_host_match(&current_hosts, current_hostname.as_deref(), target_host) {
            if let Some(ref identity_file) = current_identity_file {
                matched_identity_file = Some(identity_file.clone());
            }
        }

        matched_identity_file
    }

    /// 检查 Host 是否匹配目标 host
    ///
    /// 匹配规则：
    /// 1. 精确匹配：hosts 中包含目标 host
    /// 2. HostName 匹配：hostname 与目标 host 匹配
    ///
    /// # 参数
    ///
    /// * `hosts` - Host 指令中的 host 列表
    /// * `hostname` - HostName 指令的值
    /// * `target_host` - 要匹配的目标 host
    ///
    /// # 返回
    ///
    /// 如果匹配则返回 `true`，否则返回 `false`。
    fn is_host_match(hosts: &[String], hostname: Option<&str>, target_host: &str) -> bool {
        // 精确匹配：检查 hosts 中是否包含目标 host
        if hosts.iter().any(|h| h == target_host) {
            return true;
        }

        // HostName 匹配：检查 hostname 是否与目标 host 匹配
        if let Some(hn) = hostname {
            if hn == target_host {
                return true;
            }
        }

        false
    }

    /// 通过 Git 配置和 GitHub API 匹配密钥
    ///
    /// 根据 Git 配置的 user.email 匹配 GitHub 账号，然后尝试通过 SSH config
    /// 找到对应的密钥。匹配策略：
    /// 1. 查找 Host 为 "github-<account-name>" 的配置（如 `github-brainim`）
    /// 2. 查找 Host 为 "github_<account-name>" 的配置
    /// 3. 如果账号是当前激活的账号，也可以尝试匹配默认的 `github` Host
    ///
    /// # 返回
    ///
    /// 返回匹配的 SSH 密钥路径，如果未找到则返回 `None`。
    fn get_ssh_key_from_github_account() -> Option<PathBuf> {
        // 1. 获取 Git 配置的 user.email（优先使用仓库级别配置）
        let git_email = crate::git::helpers::open_repo()
            .ok()
            .and_then(|repo| {
                repo.config()
                    .ok()
                    .and_then(|config| config.get_string("user.email").ok())
            })
            .filter(|s| !s.is_empty())
            .or_else(|| {
                // 如果没有仓库级别配置，使用全局配置
                crate::git::GitConfig::get_global_user()
                    .ok()
                    .and_then(|(email, _)| email)
            })?;

        // 2. 在 GitHubSettings 中查找匹配的账号
        let settings = crate::base::settings::Settings::load();
        let matched_account = settings
            .github
            .accounts
            .iter()
            .find(|acc| acc.email == git_email)?;

        // 3. 检查是否是当前激活的账号
        let is_current = settings
            .github
            .current
            .as_ref()
            .map(|current| current == &matched_account.name)
            .unwrap_or(false);

        // 4. 尝试通过 SSH config 匹配账号对应的密钥
        let account_name = &matched_account.name;
        let mut possible_hosts: Vec<String> = vec![
            format!("github-{}", account_name),
            format!("github_{}", account_name),
        ];

        // 如果是当前账号，也可以尝试匹配默认的 github Host
        if is_current {
            possible_hosts.push("github".to_string());
        }

        // 解析 SSH config
        let config_path = Self::get_ssh_config_path()?;
        let config_content = std::fs::read_to_string(&config_path).ok()?;

        // 尝试匹配每个可能的 host
        for host in possible_hosts {
            if let Some(key_path) = Self::parse_ssh_config_and_match(&config_content, &host) {
                return Some(key_path);
            }
        }

        // 5. 如果没有找到匹配的 SSH config，返回 None
        // 让认证回调提供详细的配置指导
        None
    }


    /// 从 URL 中提取用户名
    ///
    /// 尝试从 HTTPS URL 中提取用户名。
    /// 例如：`https://username@github.com/owner/repo.git` → `username`
    ///
    /// # 参数
    ///
    /// * `url` - 远程仓库 URL
    ///
    /// # 返回
    ///
    /// 返回提取的用户名，如果无法提取则返回 `None`。
    fn extract_username_from_url(url: &str) -> Option<&str> {
        // 匹配 https://username@hostname/... 格式
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(at_pos) = after_protocol.find('@') {
                return Some(&after_protocol[..at_pos]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ssh_key() {
        // 测试 SSH 密钥查找
        // 注意：find_ssh_key 是私有方法，这里只测试 get_remote_callbacks 不会 panic
        let _callbacks = GitAuth::get_remote_callbacks();
        // 如果用户有 SSH 密钥，应该能找到
        // 如果没有，返回 None 也是正常的
    }

    #[test]
    fn test_extract_username_from_url() {
        assert_eq!(
            GitAuth::extract_username_from_url("https://user@github.com/owner/repo.git"),
            Some("user")
        );
        assert_eq!(
            GitAuth::extract_username_from_url("https://github.com/owner/repo.git"),
            None
        );
    }

    #[test]
    fn test_get_remote_callbacks() {
        // 测试获取认证回调（不应该 panic）
        let _callbacks = GitAuth::get_remote_callbacks();
        // RemoteCallbacks 没有公共方法可以验证，但创建成功就说明没问题
    }
}

