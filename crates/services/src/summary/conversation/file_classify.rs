//! 提交文件分类对话（阶段一）
//!
//! 根据文件变更列表 JSON 进行智能分类，输出结构化分类结果。

use llm::LLMConversation;
use crate::summary::prompt::classify_files;

/// 提交文件分类对话
///
/// 输入为设计文档中的「文件变更信息」JSON 字符串，输出为阶段一分类结果。
pub(crate) struct FileClassifyConversation {
    input_json: String,
}

impl FileClassifyConversation {
    pub fn new(input_json: impl Into<String>) -> Self {
        Self {
            input_json: input_json.into(),
        }
    }
}

impl LLMConversation for FileClassifyConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        classify_files().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        format!("## 文件变更信息\n\n{}", self.input_json)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}
