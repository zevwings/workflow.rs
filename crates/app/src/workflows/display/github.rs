//! GitHub 验证结果格式化实现

use crate::workflows::display::formatter::VerificationResultFormatter;
use domain::GitHubVerificationResult;
use prompt::{br, success, warning, TableBuilder, Tabled};

/// GitHub 账号配置表格行
///
/// 用于在表格中显示 GitHub 账号配置信息（包含验证状态）。
pub struct GitHubAccountRow {
    pub name: String,
    pub email: String,
    pub token: String,
    pub status: String,
    pub verification: String,
}

impl Tabled for GitHubAccountRow {
    fn headers() -> Vec<String> {
        vec![
            "Name".to_string(),
            "Email".to_string(),
            "API Token".to_string(),
            "Status".to_string(),
            "Verification".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.email.clone(),
            self.token.clone(),
            self.status.clone(),
            self.verification.clone(),
        ]
    }
}

impl VerificationResultFormatter for GitHubVerificationResult {
    fn format(&self) {
        if !self.configured {
            return;
        }

        let rows: Vec<GitHubAccountRow> = self
            .accounts
            .iter()
            .map(|account| GitHubAccountRow {
                name: account.name.clone(),
                email: account.email.clone(),
                token: account.token.clone(),
                status: if account.is_current {
                    "Current".to_string()
                } else {
                    String::new()
                },
                verification: account.verification_status.clone().unwrap_or_default(),
            })
            .collect();

        let table_builder = TableBuilder::from_tabled(rows);
        let _ = table_builder.display();

        // 显示验证结果
        if self.summary.success_count == self.summary.total_count {
            success!(
                "All {} GitHub account(s) verified successfully!",
                self.summary.total_count
            );
        } else {
            warning!("Some GitHub account(s) verification failed. Please check the configuration.");
        }

        br!();
    }
}
