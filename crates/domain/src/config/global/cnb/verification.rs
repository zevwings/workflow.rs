//! CNB 配置验证结果

/// CNB 账号信息（用于验证结果显示）
#[derive(Debug, Clone)]
pub struct CNBAccountInfo {
    /// 账号名称
    pub name: String,
    /// 用户登录名
    pub login: String,
    /// 账号邮箱
    pub email: String,
    /// Token 是否有效
    pub is_token_valid: bool,
}

/// CNB 验证结果
#[derive(Debug, Clone)]
pub struct CNBVerificationResult {
    /// 是否配置了 CNB
    pub is_configured: bool,
    /// 当前账号信息
    pub current_account: Option<CNBAccountInfo>,
    /// 所有账号列表
    pub accounts: Vec<CNBAccountInfo>,
    /// 验证错误信息
    pub error: Option<String>,
}

impl CNBVerificationResult {
    /// 创建未配置的验证结果
    pub fn not_configured() -> Self {
        Self {
            is_configured: false,
            current_account: None,
            accounts: Vec::new(),
            error: Some("CNB not configured".to_string()),
        }
    }

    /// 创建验证失败的结果
    pub fn failed(error: String) -> Self {
        Self {
            is_configured: true,
            current_account: None,
            accounts: Vec::new(),
            error: Some(error),
        }
    }

    /// 创建验证成功的结果
    pub fn success(current_account: CNBAccountInfo, accounts: Vec<CNBAccountInfo>) -> Self {
        Self {
            is_configured: true,
            current_account: Some(current_account),
            accounts,
            error: None,
        }
    }

    /// 检查验证是否成功
    pub fn is_success(&self) -> bool {
        self.is_configured && self.error.is_none()
    }
}

/// CNB 验证摘要（用于批量验证）
#[derive(Debug, Clone)]
pub struct CNBVerificationSummary {
    /// 是否配置
    pub configured: bool,
    /// 当前账号名称
    pub current_account: Option<String>,
    /// 验证状态
    pub status: String,
}
