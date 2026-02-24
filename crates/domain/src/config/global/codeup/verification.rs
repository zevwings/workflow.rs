//! Codeup 验证结果类型

/// Codeup 配置信息
#[derive(Debug, Clone)]
pub struct CodeupConfigInfo {
    /// 项目 ID
    pub project_id: String,
    /// CSRF Token（掩码显示）
    pub csrf_token: String,
    /// Cookie（掩码显示）
    pub cookie: String,
}

/// Codeup 验证状态
#[derive(Debug, Clone)]
pub enum CodeupVerificationStatus {
    /// 验证成功
    Success { username: String },
    /// 验证失败
    Failed {
        reason: String,
        details: Vec<String>,
    },
}

/// Codeup 验证结果
#[derive(Debug, Clone)]
pub struct CodeupVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<CodeupConfigInfo>,
    /// 验证结果
    pub verification: Option<CodeupVerificationStatus>,
}
