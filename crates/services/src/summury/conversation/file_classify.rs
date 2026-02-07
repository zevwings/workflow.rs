//! 提交文件分类对话（阶段一）
//!
//! 根据文件变更列表 JSON 进行智能分类，输出结构化分类结果。

use domain::{CommitFileClassification, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 提交文件分类对话
///
/// 输入为设计文档中的「文件变更信息」JSON 字符串，输出为阶段一分类结果。
pub(crate) struct FileClassifyConversation {
    input_json: String,
}

impl FileClassifyConversation {
    pub fn new(input_json: String) -> Self {
        Self { input_json }
    }
}

impl LLMConversation for FileClassifyConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        super::file_classify().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        format!("## 文件变更信息\n\n{}", self.input_json)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))
    }
}
