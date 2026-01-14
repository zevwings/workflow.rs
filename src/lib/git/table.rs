//! Git 表格显示相关结构体
//!
//! 提供统一的 Git 信息表格行结构，用于表格格式显示。

use crate::interactive::Tabled;

/// 分支表格行
///
/// 用于在表格中显示分支信息。
pub struct BranchRow {
    pub index: String,
    pub name: String,
}

impl Tabled for BranchRow {
    fn headers() -> Vec<String> {
        vec!["#".to_string(), "Branch Name".to_string()]
    }

    fn row(&self) -> Vec<String> {
        vec![self.index.clone(), self.name.clone()]
    }
}
