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
    /// 2. 缓存的 SSH 密钥文件
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
                            return Err(git2::Error::from_str(&format!(
                                "SSH authentication failed: {}\n\
                                \n\
                                Troubleshooting:\n\
                                1. Add SSH key to agent: ssh-add {:?}\n\
                                2. Check key permissions: chmod 600 {:?}\n\
                                3. Test SSH connection: ssh -T git@github.com\n\
                                4. Or use HTTPS URL with GITHUB_TOKEN environment variable",
                                e, key_path, key_path
                            )));
                        }
                    }
                }

                // 所有 SSH 认证方式都失败
                return Err(git2::Error::from_str(
                    "No SSH key found. Please ensure:\n\
                    1. SSH agent is running and has keys added: ssh-add ~/.ssh/id_ed25519\n\
                    2. Or SSH key file exists at ~/.ssh/id_ed25519, ~/.ssh/id_rsa, or ~/.ssh/id_ecdsa\n\
                    3. Or switch to HTTPS URL and set GITHUB_TOKEN environment variable"
                ));
            }

            // 3. HTTPS 认证
            if is_https && allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                let username = auth_info
                    .https_username
                    .as_deref()
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
            let https_token =
                std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GIT_TOKEN")).ok();
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
    /// 2. SSH agent 中的密钥（在认证回调中处理）
    /// 3. 默认密钥顺序：`~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, `~/.ssh/id_ecdsa`
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

        // 优先级 2: SSH agent（在认证回调中处理，这里跳过）

        // 优先级 3: 默认密钥顺序
        Self::find_ssh_key_default()
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
        let home_dir = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;

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
                            if let Ok(home_dir) =
                                std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
                            {
                                PathBuf::from(&home_dir).join(&identity_file[2..])
                            // 跳过 "~/"
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

    /// 查找默认 SSH 密钥文件
    ///
    /// 按优先级顺序查找常见的 SSH 密钥文件：
    /// 1. `~/.ssh/id_ed25519` (推荐，最安全)
    /// 2. `~/.ssh/id_rsa` (最常见)
    /// 3. `~/.ssh/id_ecdsa` (较少使用)
    ///
    /// # 返回
    ///
    /// 返回找到的第一个存在的密钥文件路径，如果都不存在则返回 `None`。
    fn find_ssh_key_default() -> Option<PathBuf> {
        let home_dir = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;

        let key_paths = vec![
            PathBuf::from(&home_dir).join(".ssh").join("id_ed25519"),
            PathBuf::from(&home_dir).join(".ssh").join("id_rsa"),
            PathBuf::from(&home_dir).join(".ssh").join("id_ecdsa"),
        ];

        key_paths.into_iter().find(|p| p.exists())
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
