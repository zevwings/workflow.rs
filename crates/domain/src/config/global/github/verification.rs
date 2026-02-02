//! GitHub 验证结果类型

/// GitHub 账号信息
#[derive(Debug, Clone)]
pub struct GitHubAccountInfo {
    /// 账号名称
    pub name: String,
    /// 是否当前账号
    pub is_current: bool,
    /// 邮箱
    pub email: String,
    /// API Token（掩码显示）
    pub token: String,
    /// 验证状态
    pub verification_status: Option<String>,
    /// 验证错误信息（如果验证失败）
    pub verification_error: Option<String>,
}

/// GitHub 验证总结
#[derive(Debug, Clone)]
pub struct GitHubVerificationSummary {
    /// 总账号数
    pub total_count: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败账号列表
    pub failed_accounts: Vec<String>,
}

/// GitHub 验证结果
#[derive(Debug, Clone)]
pub struct GitHubVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 账号列表
    pub accounts: Vec<GitHubAccountInfo>,
    /// 验证总结
    pub summary: GitHubVerificationSummary,
}
