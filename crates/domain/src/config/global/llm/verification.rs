//! LLM 验证结果类型

/// LLM 配置信息
#[derive(Debug, Clone)]
pub struct LLMConfig {
    /// Provider
    pub provider: String,
    /// Model（包含 URL 信息，如果适用）
    pub model: String,
    /// Key（掩码显示）
    pub key: String,
    /// Output Language
    pub language: String,
}

/// LLM 验证状态
#[derive(Debug, Clone)]
pub enum LLMVerificationStatus {
    /// 验证成功
    Success {
        /// 测试响应内容
        test_response: String,
    },
    /// 验证失败
    Failed {
        /// 失败原因
        reason: String,
        /// 详细错误信息
        details: Vec<String>,
    },
}

/// LLM 验证结果
#[derive(Debug, Clone)]
pub struct LLMVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<LLMConfig>,
    /// 验证结果
    pub verification: Option<LLMVerificationStatus>,
}
