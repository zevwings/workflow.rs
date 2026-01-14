//! JIRA 表格显示相关结构体
//!
//! 提供统一的 JIRA 信息表格行结构，用于表格格式显示。

use crate::interactive::Tabled;

/// JIRA 附件表格行
///
/// 用于在表格中显示 JIRA 附件信息。
pub struct AttachmentRow {
    pub index: String,
    pub filename: String,
    pub size: String,
    pub mime_type: String,
}

impl Tabled for AttachmentRow {
    fn headers() -> Vec<String> {
        vec![
            "#".to_string(),
            "Filename".to_string(),
            "Size".to_string(),
            "MIME Type".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.index.clone(),
            self.filename.clone(),
            self.size.clone(),
            self.mime_type.clone(),
        ]
    }
}

/// JIRA 日志文件表格行
///
/// 用于在表格中显示 JIRA 日志文件信息。
#[derive(Clone)]
pub struct FileRow {
    pub file_type: String,
    pub name: String,
    pub size: String,
}

impl Tabled for FileRow {
    fn headers() -> Vec<String> {
        vec!["Type".to_string(), "Name".to_string(), "Size".to_string()]
    }

    fn row(&self) -> Vec<String> {
        vec![self.file_type.clone(), self.name.clone(), self.size.clone()]
    }
}
