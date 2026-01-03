//! Git 远程操作认证辅助
//!
//! 提供认证相关的辅助功能，如查找 SSH 密钥、从 URL 提取信息等。
//! 注意：Git 命令本身会通过系统 SSH agent 和 Git 凭据存储来处理认证。

use std::path::PathBuf;

/// Git 远程操作认证管理
///
/// 提供认证相关的辅助功能。
/// 注意：Git 命令本身会通过系统 SSH agent 和 Git 凭据存储来处理认证，
/// 所以不需要手动处理认证回调。
pub struct GitAuth;

impl GitAuth {
    /// 查找 SSH 密钥文件路径
    ///
    /// 按优先级顺序查找 SSH 密钥：
    /// 1. SSH config 匹配（如果远程 URL 匹配 SSH config 中的 Host）
    /// 2. 默认密钥顺序：`~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, `~/.ssh/id_ecdsa`
    ///
    /// # 返回
    ///
    /// 返回找到的第一个存在的密钥文件路径，如果都不存在则返回 `None`。
    pub fn find_ssh_key() -> Option<PathBuf> {
        // 优先级 1: 尝试从 SSH config 获取
        if let Ok(remote_url) = crate::git::GitRepo::get_remote_url() {
            if let Some(key_path) = Self::get_ssh_key_from_config(&remote_url) {
                return Some(key_path);
            }
        }

        // 优先级 2: 默认密钥顺序
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
    pub fn extract_host_from_url(url: &str) -> Option<String> {
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
    pub fn extract_username_from_url(url: &str) -> Option<&str> {
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

    /// 测试从 URL 中提取用户名（private 方法测试）
    ///
    /// ## 测试目的
    /// 验证 `extract_username_from_url()` 私有方法能够正确从 HTTPS URL 中提取用户名。
    ///
    /// ## 测试场景
    /// 1. 测试包含用户名的 HTTPS URL
    /// 2. 测试不包含用户名的 HTTPS URL
    ///
    /// ## 预期结果
    /// - 包含用户名的 URL 返回 Some("username")
    /// - 不包含用户名的 URL 返回 None
    #[test]
    fn test_extract_username_from_url() {
        // Test URL with username
        assert_eq!(
            GitAuth::extract_username_from_url("https://user@github.com/owner/repo.git"),
            Some("user")
        );

        // Test URL without username
        assert_eq!(
            GitAuth::extract_username_from_url("https://github.com/owner/repo.git"),
            None
        );
    }
}
