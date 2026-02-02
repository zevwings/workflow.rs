//! Jira 验证结果类型

/// Jira 配置信息
#[derive(Debug, Clone)]
pub struct JiraConfigInfo {
    /// 邮箱
    pub email: String,
    /// 服务地址
    pub service_address: String,
    /// API Token（掩码显示）
    pub api_token: String,
}

/// Jira 验证状态
#[derive(Debug, Clone)]
pub enum JiraVerificationStatus {
    /// 验证成功
    Success { email: String, account_id: String },
    /// 验证失败
    Failed {
        reason: String,
        details: Vec<String>,
    },
}

/// Jira 验证结果
#[derive(Debug, Clone)]
pub struct JiraVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<JiraConfigInfo>,
    /// 验证结果
    pub verification: Option<JiraVerificationStatus>,
}
