//! GitHub 配置相关结构体

use crate::core::prompt::Tabled;
use crate::core::util::format::Sensitive;
use crate::services::github::GitHub;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// GitHub 账号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAccount {
    /// 账号名称（用于标识和切换）
    pub name: String,
    /// 账号邮箱（必填，用于显示和区分）
    pub email: String,
    /// GitHub API Token
    pub api_token: String,
}

/// GitHub 配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// 多个 GitHub 账号列表
    #[serde(default)]
    pub accounts: Vec<GitHubAccount>,
    /// 当前激活的账号名称
    pub current: Option<String>,
}

impl GitHubSettings {
    /// 检查 GitHub 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.current.is_none()
    }

    /// 获取当前激活的账号
    ///
    /// 如果设置了 `current`，返回对应的账号；否则返回第一个账号。
    /// 如果没有账号，返回 `None`。
    pub fn get_current_account(&self) -> Option<&GitHubAccount> {
        if self.accounts.is_empty() {
            return None;
        }

        if let Some(ref current_name) = self.current {
            self.accounts.iter().find(|acc| acc.name == *current_name)
        } else {
            // 如果没有设置 current，返回第一个账号
            self.accounts.first()
        }
    }

    /// 获取当前账号的 API Token
    pub fn get_current_token(&self) -> Option<&str> {
        self.get_current_account().map(|acc| acc.api_token.as_str())
    }
}

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
    pub verification_status: String,
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

impl GitHubSettings {
    /// 验证 GitHub 配置并返回结果
    pub fn verify(&self) -> Result<GitHubVerificationResult> {
        if self.accounts.is_empty() {
            return Ok(GitHubVerificationResult {
                configured: false,
                accounts: Vec::new(),
                summary: GitHubVerificationSummary {
                    total_count: 0,
                    success_count: 0,
                    failed_accounts: Vec::new(),
                },
            });
        }

        let mut success_count = 0;
        let mut failed_accounts = Vec::new();
        let mut account_infos = Vec::new();

        for account in &self.accounts {
            let is_current =
                self.current.as_ref().map(|c| c == &account.name).unwrap_or_else(|| {
                    // 如果没有设置 current，第一个账号是当前账号
                    self.accounts.first().map(|a| &a.name) == Some(&account.name)
                });

            // 使用该账号的 token 验证
            let (verification_status, verification_error) =
                match GitHub::get_user_info(Some(&account.api_token)) {
                    Ok(_user) => {
                        success_count += 1;
                        ("Success".to_string(), None)
                    }
                    Err(e) => {
                        failed_accounts.push(account.name.clone());
                        ("Failed".to_string(), Some(format!("{}", e)))
                    }
                };

            account_infos.push(GitHubAccountInfo {
                name: account.name.clone(),
                is_current,
                email: account.email.clone(),
                token: account.api_token.mask(),
                verification_status,
                verification_error,
            });
        }

        let total_count = self.accounts.len();
        Ok(GitHubVerificationResult {
            configured: true,
            accounts: account_infos,
            summary: GitHubVerificationSummary {
                total_count,
                success_count,
                failed_accounts,
            },
        })
    }
}

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

/// GitHub 账号列表表格行
///
/// 用于在表格中显示 GitHub 账号列表信息（包含索引）。
pub struct GitHubAccountListRow {
    pub index: String,
    pub name: String,
    pub email: String,
    pub token: String,
    pub status: String,
}

impl Tabled for GitHubAccountListRow {
    fn headers() -> Vec<String> {
        vec![
            "#".to_string(),
            "Name".to_string(),
            "Email".to_string(),
            "API Token".to_string(),
            "Status".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.index.clone(),
            self.name.clone(),
            self.email.clone(),
            self.token.clone(),
            self.status.clone(),
        ]
    }
}
