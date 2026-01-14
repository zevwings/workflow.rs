//! PR 表格显示相关结构体
//!
//! 提供统一的 PR 列表表格行结构，用于表格格式显示。

use crate::interactive::Tabled;

/// PR 列表表格行
///
/// 统一的表格行结构，用于在表格中显示 PR 信息。
pub struct PullRequestRow {
    pub number: String,
    pub state: String,
    pub branch: String,
    pub title: String,
    pub author: String,
    pub url: String,
}

impl Tabled for PullRequestRow {
    fn headers() -> Vec<String> {
        vec![
            "#".to_string(),
            "State".to_string(),
            "Branch".to_string(),
            "Title".to_string(),
            "Author".to_string(),
            "URL".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.number.clone(),
            self.state.clone(),
            self.branch.clone(),
            self.title.clone(),
            self.author.clone(),
            self.url.clone(),
        ]
    }
}
