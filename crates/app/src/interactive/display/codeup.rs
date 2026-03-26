//! Codeup 验证结果格式化实现

use domain::{CodeupVerificationResult, CodeupVerificationStatus};
use prompt::{br, success, warning, TableBuilder, Tabled};

use crate::interactive::display::formatter::VerificationResultFormatter;

/// Codeup 配置表格行
pub struct CodeupConfigRow {
    pub project_id: String,
    pub csrf_token: String,
    pub cookie: String,
}

impl Tabled for CodeupConfigRow {
    fn headers() -> Vec<String> {
        vec![
            "Project ID".to_string(),
            "CSRF Token".to_string(),
            "Cookie".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.project_id.clone(),
            self.csrf_token.clone(),
            self.cookie.clone(),
        ]
    }
}

impl VerificationResultFormatter for CodeupVerificationResult {
    fn format(&self) {
        if !self.configured {
            return;
        }

        if let Some(ref config) = self.config {
            let row = CodeupConfigRow {
                project_id: config.project_id.clone(),
                csrf_token: config.csrf_token.clone(),
                cookie: config.cookie.clone(),
            };

            let table_builder = TableBuilder::from_tabled(vec![row]);
            let _ = table_builder.display();
        }

        if let Some(ref verification) = self.verification {
            match verification {
                CodeupVerificationStatus::Success { username } => {
                    success!("Codeup verification successful! User: {}", username);
                }
                CodeupVerificationStatus::Failed { reason, .. } => {
                    warning!("Codeup verification error: {}", reason);
                }
            }
        }

        br!();
    }
}
