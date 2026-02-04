//! 提交操作实体

use serde::{Deserialize, Serialize};

/// Amend 预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendPreview {
    /// 原始 commit SHA
    pub original_sha: String,
    /// 新提交消息
    pub new_message: Option<String>,
    /// 原始提交消息
    pub original_message: String,
    /// 要添加的文件列表
    pub files_to_add: Vec<String>,
    /// 操作类型
    pub operation_type: String,
    /// 是否已推送到远程
    pub is_pushed: bool,
}
