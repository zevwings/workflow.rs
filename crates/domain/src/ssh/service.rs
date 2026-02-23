//! SSH 服务接口

use std::path::{Path, PathBuf};

use super::entity::SshKeyInfo;
use super::error::SshError;

/// SSH 密钥管理服务
pub trait SshService: Send + Sync {
    /// 检查 ssh-agent 是否可用
    fn is_agent_available(&self) -> bool;

    /// 列出 ssh-agent 中已加载的密钥
    fn list_loaded_keys(&self) -> Result<Vec<SshKeyInfo>, SshError>;

    /// 扫描 ~/.ssh/ 下的常见私钥文件
    fn scan_keys(&self) -> Vec<PathBuf>;

    /// 获取给定算法的默认密钥路径
    fn default_key_path(&self, algorithm: &str) -> PathBuf;

    /// 生成 SSH 密钥
    fn generate_key(
        &self,
        output_path: &Path,
        algorithm: &str,
        comment: Option<&str>,
        passphrase: Option<&str>,
        force: bool,
    ) -> Result<(), SshError>;

    /// 添加密钥到 ssh-agent
    fn add_key(&self, key_path: &Path, lifetime: Option<u64>) -> Result<(), SshError>;

    /// 通过指纹查找 ~/.ssh/ 下对应的私钥路径
    ///
    /// 扫描 ~/.ssh/*.pub，用 ssh-keygen -lf 获取指纹进行匹配。
    fn find_key_path_by_fingerprint(&self, fingerprint: &str) -> Option<PathBuf>;

    /// 从 ssh-agent 移除密钥
    fn remove_key_by_path(&self, key_path: &Path) -> Result<(), SshError>;

    /// 清空 ssh-agent 中所有密钥
    fn remove_all_keys(&self) -> Result<(), SshError>;
}
