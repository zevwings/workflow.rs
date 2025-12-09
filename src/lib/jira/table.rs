//! JIRA 表格显示相关结构体
//!
//! 提供统一的 JIRA 信息表格行结构，用于表格格式显示。

use tabled::Tabled;

/// JIRA 附件表格行
///
/// 用于在表格中显示 JIRA 附件信息。
#[derive(Tabled)]
pub struct AttachmentRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Filename")]
    pub filename: String,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "MIME Type")]
    pub mime_type: String,
}

/// JIRA 日志文件表格行
///
/// 用于在表格中显示 JIRA 日志文件信息。
#[derive(Tabled, Clone)]
pub struct FileRow {
    #[tabled(rename = "Type")]
    pub file_type: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Size")]
    pub size: String,
}
