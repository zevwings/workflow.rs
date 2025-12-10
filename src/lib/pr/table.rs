//! PR 表格显示相关结构体
//!
//! 提供统一的 PR 列表表格行结构，用于表格格式显示。

use tabled::Tabled;

/// PR 列表表格行
///
/// 统一的表格行结构，用于在表格中显示 PR 信息。
#[derive(Tabled)]
pub struct PullRequestRow {
    #[tabled(rename = "#")]
    pub number: String,
    #[tabled(rename = "State")]
    pub state: String,
    #[tabled(rename = "Branch")]
    pub branch: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Author")]
    pub author: String,
    #[tabled(rename = "URL")]
    pub url: String,
}
