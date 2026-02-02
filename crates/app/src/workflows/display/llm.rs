//! LLM 验证结果格式化实现

use crate::workflows::display::formatter::VerificationResultFormatter;
use domain::{LLMVerificationResult, LLMVerificationStatus};
use prompt::{br, error, info, success, TableBuilder, Tabled};

/// LLM 配置表格行
///
/// 用于在表格中显示 LLM 配置信息（包含验证状态）。
pub struct LLMConfigRow {
    pub provider: String,
    pub model: String,
    pub key: String,
    pub language: String,
    pub verification: String,
}

impl Tabled for LLMConfigRow {
    fn headers() -> Vec<String> {
        vec![
            "Provider".to_string(),
            "Model".to_string(),
            "Key".to_string(),
            "Language".to_string(),
            "Verification".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.provider.clone(),
            self.model.clone(),
            self.key.clone(),
            self.language.clone(),
            self.verification.clone(),
        ]
    }
}

impl VerificationResultFormatter for LLMVerificationResult {
    fn format(&self) {
        // 只要有配置信息就显示表格，不管 configured 是否为 true
        if let Some(ref config) = self.config {
            let (verification_status, failure_reason) =
                if let Some(ref verification) = self.verification {
                    match verification {
                        LLMVerificationStatus::Success { .. } => ("✓ Valid".to_string(), None),
                        LLMVerificationStatus::Failed { reason, .. } => {
                            ("✗ Failed".to_string(), Some(reason.clone()))
                        }
                    }
                } else {
                    ("Not verified".to_string(), None)
                };

            let row = LLMConfigRow {
                provider: config.provider.clone(),
                model: config.model.clone(),
                key: config.key.clone(),
                language: config.language.clone(),
                verification: verification_status,
            };

            let table_builder = TableBuilder::from_tabled(vec![row]);
            let _ = table_builder.display();

            // 如果验证失败，显示错误信息
            if let Some(reason) = failure_reason {
                error!("LLM Verify failed: {}", reason);
            }
        } else if !self.configured {
            // 如果没有配置，直接返回
            return;
        }

        // 如果验证成功，输出成功消息和测试详情
        if let Some(LLMVerificationStatus::Success { test_response }) = &self.verification {
            info!("  System prompt: You are a helpful assistant.");
            info!("  User prompt: Say hello");
            info!("  Response: {}", test_response);
            success!("LLM verified successfully!");
        }

        br!();
    }
}
