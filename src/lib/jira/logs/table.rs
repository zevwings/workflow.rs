//! JIRA 日志搜索表格显示相关结构体
//!
//! 提供统一的日志搜索结果表格行结构，用于表格格式显示。

use tabled::Tabled;

/// 日志搜索结果表格行
///
/// 用于在表格中显示日志搜索结果信息。
#[derive(Tabled)]
pub struct SearchResultRow {
    #[tabled(rename = "Source")]
    pub source: String,
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "URL")]
    pub url: String,
}
