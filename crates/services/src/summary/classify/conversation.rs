//! 阶段一：文件分类服务
//!
//! 根据 commit 元数据和文件变更列表，调用 LLM 进行智能分类。

use client::LLMConversation;

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
    fn get_system_prompt(&self) -> String {
        include_str!("prompt.md").to_string()
    }

    fn get_user_prompt(&self) -> String {
        format!("## File Change Information\n\n{}", self.input_json)
    }

    fn get_max_tokens(&self) -> Option<u32> {
        // Set a very high token limit for file classification to handle massive commits.
        // With the optimized prompt instructing representative sampling for by_status,
        // this should be more than enough even for 500+ file commits.
        Some(32000)
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
