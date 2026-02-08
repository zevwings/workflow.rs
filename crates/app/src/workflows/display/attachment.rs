//! JIRA 附件表格显示相关结构体
//!
//! 提供统一的 JIRA 附件信息表格行结构，用于表格格式显示。

use prompt::Tabled;

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
            "文件名".to_string(),
            "大小".to_string(),
            "MIME 类型".to_string(),
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
