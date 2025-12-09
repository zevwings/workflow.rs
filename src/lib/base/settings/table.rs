//! 配置表格显示相关结构体
//!
//! 提供统一的配置信息表格行结构，用于表格格式显示。

use tabled::Tabled;

/// LLM 配置表格行
///
/// 用于在表格中显示 LLM 配置信息。
#[derive(Tabled)]
pub struct LLMConfigRow {
    #[tabled(rename = "Provider")]
    pub provider: String,
    #[tabled(rename = "Model")]
    pub model: String,
    #[tabled(rename = "Key")]
    pub key: String,
    #[tabled(rename = "Output Language")]
    pub language: String,
}

/// JIRA 配置表格行
///
/// 用于在表格中显示 JIRA 配置信息。
#[derive(Tabled)]
pub struct JiraConfigRow {
    #[tabled(rename = "Email")]
    pub email: String,
    #[tabled(rename = "Service Address")]
    pub service_address: String,
    #[tabled(rename = "API Token")]
    pub api_token: String,
}

/// GitHub 账号配置表格行
///
/// 用于在表格中显示 GitHub 账号配置信息（包含验证状态）。
#[derive(Tabled)]
pub struct GitHubAccountRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Email")]
    pub email: String,
    #[tabled(rename = "API Token")]
    pub token: String,
    #[tabled(rename = "Branch Prefix")]
    pub prefix: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Verification")]
    pub verification: String,
}

/// GitHub 账号列表表格行
///
/// 用于在表格中显示 GitHub 账号列表信息（包含索引）。
#[derive(Tabled)]
pub struct GitHubAccountListRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Email")]
    pub email: String,
    #[tabled(rename = "API Token")]
    pub token: String,
    #[tabled(rename = "Branch Prefix")]
    pub prefix: String,
    #[tabled(rename = "Status")]
    pub status: String,
}
