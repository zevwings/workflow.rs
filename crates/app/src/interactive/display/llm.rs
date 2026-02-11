//! LLM 验证结果格式化实现

use domain::{LLMVerificationResult, LLMVerificationStatus};
use prompt::{br, error, info, success, TableBuilder, Tabled};

use crate::interactive::display::formatter::VerificationResultFormatter;

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
            "提供商".to_string(),
            "模型".to_string(),
            "密钥".to_string(),
            "语言".to_string(),
            "验证".to_string(),
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
                        LLMVerificationStatus::Success { .. } => ("✓ 有效".to_string(), None),
                        LLMVerificationStatus::Failed { reason, .. } => {
                            ("✗ 失败".to_string(), Some(reason.clone()))
                        }
                    }
                } else {
                    ("未验证".to_string(), None)
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
                error!("LLM 验证失败: {}", reason);
            }
        } else if !self.configured {
            // 如果没有配置，直接返回
            return;
        }

        // 如果验证成功，输出成功消息和测试详情
        if let Some(LLMVerificationStatus::Success { test_response }) = &self.verification {
            info!("  系统提示: You are a helpful assistant.");
            info!("  用户提示: Say hello");
            info!("  响应: {}", test_response);
            success!("LLM 验证成功！");
        }

        br!();
    }
}
