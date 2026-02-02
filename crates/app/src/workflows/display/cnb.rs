//! CNB 验证结果格式化实现

use crate::workflows::display::formatter::VerificationResultFormatter;
use domain::CNBVerificationResult;
use prompt::{br, success, warning, TableBuilder, Tabled};

/// CNB 账号配置表格行
///
/// 用于在表格中显示 CNB 账号配置信息（包含验证状态）。
pub struct CNBAccountRow {
    pub name: String,
    pub login: String,
    pub email: String,
    pub status: String,
    pub verification: String,
}

impl Tabled for CNBAccountRow {
    fn headers() -> Vec<String> {
        vec![
            "Name".to_string(),
            "Username".to_string(),
            "Email".to_string(),
            "Status".to_string(),
            "Verification".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.login.clone(),
            self.email.clone(),
            self.status.clone(),
            self.verification.clone(),
        ]
    }
}

impl VerificationResultFormatter for CNBVerificationResult {
    fn format(&self) {
        if !self.is_configured {
            return;
        }

        let rows: Vec<CNBAccountRow> = self
            .accounts
            .iter()
            .map(|account| CNBAccountRow {
                name: account.name.clone(),
                login: account.login.clone(),
                email: account.email.clone(),
                status: if self
                    .current_account
                    .as_ref()
                    .map(|current| current.name == account.name)
                    .unwrap_or(false)
                {
                    "Current".to_string()
                } else {
                    String::new()
                },
                verification: if account.is_token_valid {
                    "✓ Valid".to_string()
                } else {
                    "✗ Invalid".to_string()
                },
            })
            .collect();

        let table_builder = TableBuilder::from_tabled(rows);
        let _ = table_builder.display();

        // 显示验证结果
        if self.is_success() {
            success!(
                "CNB account(s) verified successfully! Current: {}",
                self.current_account
                    .as_ref()
                    .map(|a| a.name.as_str())
                    .unwrap_or("None")
            );
        } else if let Some(error) = &self.error {
            warning!("CNB verification failed: {}", error);
        } else {
            warning!("Some CNB account(s) verification failed. Please check the configuration.");
        }

        br!();
    }
}
