//! Git 远程操作认证回调
//!
//! 提供统一的认证回调机制，支持 SSH 和 HTTPS 两种认证方式。
//! 使用智能检测和缓存机制，自动选择最合适的认证方式。

use crate::trace_debug;
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
    /// 获取远程操作认证回调
    ///
    /// 每次调用都会创建新的 `RemoteCallbacks` 对象，但使用缓存的认证信息。
    /// 这样可以避免重复查找 SSH 密钥和环境变量，提高性能。
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
            trace_debug!("Git authentication requested for URL: {}", url);
            trace_debug!("Username from URL: {:?}", username_from_url);
            trace_debug!("Allowed credential types: {:?}", allowed_types);

            // 1. 检测 URL 类型
            let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");
            let is_https = url.starts_with("https://");

            trace_debug!("URL type detected - SSH: {}, HTTPS: {}", is_ssh, is_https);

            // 2. SSH 认证
            if is_ssh && allowed_types.contains(CredentialType::SSH_KEY) {
                trace_debug!("Attempting SSH authentication...");

                // 优先尝试 SSH agent（最方便，适合开发环境）
                let username = username_from_url.unwrap_or("git");
                trace_debug!("Trying SSH agent for user: {}", username);
                match Cred::ssh_key_from_agent(username) {
                    Ok(cred) => {
                        trace_debug!("SSH authentication succeeded via SSH agent");
                        return Ok(cred);
                    }
                    Err(e) => {
                        trace_debug!("SSH agent authentication failed: {} (code: {}, class: {})",
                            e.message(), e.code() as i32, e.class() as i32);
                    }
                }

                trace_debug!("SSH agent authentication failed, trying SSH key file");

                // 回退到缓存的 SSH 密钥文件
                if let Some(ref key_path) = auth_info.ssh_key_path {
                    trace_debug!("Using SSH key file: {:?}", key_path);
                    match Cred::ssh_key(username, None, key_path, None) {
                        Ok(cred) => {
                            trace_debug!("SSH authentication succeeded via key file");
                            return Ok(cred);
                        }
                        Err(e) => {
                            trace_debug!("SSH key file authentication failed: {} (code: {}, class: {})",
                                e.message(), e.code() as i32, e.class() as i32);

                            // 提供详细的错误信息
                            let error_msg = format!(
                                "SSH authentication failed: {}\n\n\
                                Possible solutions:\n\
                                1. Check if SSH agent is running: ssh-add -l\n\
                                2. Add your SSH key to agent: ssh-add ~/.ssh/id_ed25519\n\
                                3. Verify SSH key is added to GitHub/GitLab: ssh -T git@github.com\n\
                                4. Check SSH key file permissions: chmod 600 ~/.ssh/id_ed25519\n\
                                5. Consider using HTTPS with token: git remote set-url origin https://github.com/owner/repo.git",
                                e.message()
                            );
                            return Err(git2::Error::from_str(&error_msg));
                        }
                    }
                }

                // 所有 SSH 认证方式都失败
                trace_debug!("All SSH authentication methods failed");
                return Err(git2::Error::from_str(
                    "No SSH key found. Please ensure:\n\
                    1. SSH agent is running and has keys added (ssh-add ~/.ssh/id_ed25519)\n\
                    2. Or SSH key file exists at ~/.ssh/id_ed25519, ~/.ssh/id_rsa, or ~/.ssh/id_ecdsa\n\
                    3. Verify SSH key is added to your Git provider (GitHub/GitLab)\n\
                    4. Consider using HTTPS with token instead"
                ));
            }

            // 3. HTTPS 认证
            if is_https && allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                trace_debug!("Attempting HTTPS authentication...");

                let username = auth_info
                    .https_username
                    .as_deref()
                    .or(username_from_url)
                    .or_else(|| Self::extract_username_from_url(url))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "git".to_string());

                trace_debug!("Using username: {}", username);

                // 使用缓存的 HTTPS token
                if let Some(ref token) = auth_info.https_token {
                    trace_debug!("HTTPS token found, attempting authentication");
                    match Cred::userpass_plaintext(&username, token) {
                        Ok(cred) => {
                            trace_debug!("HTTPS authentication succeeded");
                            return Ok(cred);
                        }
                        Err(e) => {
                            trace_debug!("HTTPS authentication failed: {}", e);
                            return Err(e);
                        }
                    }
                }

                // HTTPS 认证失败
                trace_debug!("No HTTPS token found in environment variables");
                return Err(git2::Error::from_str(
                    "No HTTPS credentials found. Please set one of:\n\
                    1. GITHUB_TOKEN environment variable\n\
                    2. GIT_TOKEN environment variable",
                ));
            }

            // 不支持的认证类型
            trace_debug!("Unsupported credential type for URL: {}", url);
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
            trace_debug!("Initializing Git authentication cache...");

            let ssh_key_path = Self::find_ssh_key();
            if let Some(ref key_path) = ssh_key_path {
                trace_debug!("Found SSH key: {:?}", key_path);
            } else {
                trace_debug!("No SSH key found");
            }

            let https_token =
                std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GIT_TOKEN")).ok();
            if https_token.is_some() {
                trace_debug!("HTTPS token found in environment");
            } else {
                trace_debug!("No HTTPS token found in environment");
            }

            let https_username = std::env::var("GIT_USERNAME").ok();
            if let Some(ref username) = https_username {
                trace_debug!("HTTPS username found: {}", username);
            }

            CachedAuthInfo {
                ssh_key_path,
                https_token,
                https_username,
            }
        })
    }

    /// 查找 SSH 密钥文件
    ///
    /// 按优先级顺序查找常见的 SSH 密钥文件：
    /// 1. `~/.ssh/id_ed25519` (推荐，最安全)
    /// 2. `~/.ssh/id_rsa` (最常见)
    /// 3. `~/.ssh/id_ecdsa` (较少使用)
    ///
    /// # 返回
    ///
    /// 返回找到的第一个存在的密钥文件路径，如果都不存在则返回 `None`。
    fn find_ssh_key() -> Option<PathBuf> {
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
        let key = GitAuth::find_ssh_key();
        // 如果用户有 SSH 密钥，应该能找到
        // 如果没有，返回 None 也是正常的
        println!("Found SSH key: {:?}", key);
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
        assert!(true); // 占位符
    }
}
