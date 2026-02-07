//! 阶段一：文件分类服务
//!
//! 根据 commit 元数据和文件变更列表，调用 LLM 进行智能分类。

use llm::LLMConversation;

use crate::summary::prompt;

/// 提交文件分类对话
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
        prompt::classify_files().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        format!("## 文件变更信息\n\n{}", self.input_json)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}
