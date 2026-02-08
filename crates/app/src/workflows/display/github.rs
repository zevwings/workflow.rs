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
            "名称".to_string(),
            "邮箱".to_string(),
            "API 令牌".to_string(),
            "状态".to_string(),
            "验证".to_string(),
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
                    "当前".to_string()
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
            success!("所有 {} 个 GitHub 账户验证成功！", self.summary.total_count);
        } else {
            warning!("部分 GitHub 账户验证失败。请检查配置。");
        }

        br!();
    }
}
