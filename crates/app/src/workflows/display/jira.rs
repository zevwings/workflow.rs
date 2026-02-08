//! Jira 验证结果格式化实现

use crate::workflows::display::formatter::VerificationResultFormatter;
use domain::{JiraVerificationResult, JiraVerificationStatus};
use prompt::{br, success, warning, TableBuilder, Tabled};

/// Jira 配置表格行
///
/// 用于在表格中显示 Jira 配置信息。
pub struct JiraConfigRow {
    pub email: String,
    pub service_address: String,
    pub api_token: String,
}

impl Tabled for JiraConfigRow {
    fn headers() -> Vec<String> {
        vec![
            "邮箱".to_string(),
            "服务地址".to_string(),
            "API 令牌".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.email.clone(),
            self.service_address.clone(),
            self.api_token.clone(),
        ]
    }
}

impl VerificationResultFormatter for JiraVerificationResult {
    fn format(&self) {
        if !self.configured {
            return;
        }

        if let Some(ref config) = self.config {
            let row = JiraConfigRow {
                email: config.email.clone(),
                service_address: config.service_address.clone(),
                api_token: config.api_token.clone(),
            };

            let table_builder = TableBuilder::from_tabled(vec![row]);
            let _ = table_builder.display();
        }

        // 显示验证结果
        if let Some(ref verification) = self.verification {
            match verification {
                JiraVerificationStatus::Success { email, account_id } => {
                    if !account_id.is_empty() {
                        success!("Jira 验证成功！邮箱: {} (账户 ID: {})", email, account_id);
                    } else {
                        success!("Jira 验证成功！邮箱: {}", email);
                    }
                }
                JiraVerificationStatus::Failed { reason, .. } => {
                    warning!("Jira 验证错误: {}", reason);
                }
            }
        }

        br!();
    }
}
