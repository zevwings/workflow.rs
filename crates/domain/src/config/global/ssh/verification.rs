//! SSH 验证结果类型

use crate::ssh::entity::SshKeyInfo;

/// SSH 验证结果
#[derive(Debug, Clone)]
pub struct SshVerificationResult {
    /// ssh-agent 是否可用
    pub agent_available: bool,
    /// 已加载的密钥列表
    pub loaded_keys: Vec<SshKeyInfo>,
    /// 错误信息（如果有）
    pub error: Option<String>,
}
