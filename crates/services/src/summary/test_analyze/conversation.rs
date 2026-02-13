use client::LLMConversation;

use crate::summary::prompt;

/// 测试文件分析对话
pub(crate) struct TestAnalyzeConversation {
    user_prompt: String,
}

impl TestAnalyzeConversation {
    pub fn new(user_prompt: impl Into<String>) -> Self {
        Self {
            user_prompt: user_prompt.into(),
        }
    }
}

impl LLMConversation for TestAnalyzeConversation {
    fn get_system_prompt(&self) -> String {
        prompt::analyze_tests().to_string()
    }

    fn get_user_prompt(&self) -> String {
        self.user_prompt.clone()
    }

    fn get_max_tokens(&self) -> Option<u32> {
        None
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
