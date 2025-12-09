//! Git 表格显示相关结构体
//!
//! 提供统一的 Git 信息表格行结构，用于表格格式显示。

use tabled::Tabled;

/// 分支表格行
///
/// 用于在表格中显示分支信息。
#[derive(Tabled)]
pub struct BranchRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Branch Name")]
    pub name: String,
}
