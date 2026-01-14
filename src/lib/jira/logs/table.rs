//! JIRA 日志搜索表格显示相关结构体
//!
//! 提供统一的日志搜索结果表格行结构，用于表格格式显示。

use crate::prompt::Tabled;

/// 日志搜索结果表格行
///
/// 用于在表格中显示日志搜索结果信息。
pub struct SearchResultRow {
    pub source: String,
    pub id: String,
    pub url: String,
}

impl Tabled for SearchResultRow {
    fn headers() -> Vec<String> {
        vec!["Source".to_string(), "ID".to_string(), "URL".to_string()]
    }

    fn row(&self) -> Vec<String> {
        vec![self.source.clone(), self.id.clone(), self.url.clone()]
    }
}
