use client::LLMConversation;

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
        include_str!("prompt.md").to_string()
    }

    fn get_user_prompt(&self) -> String {
        self.user_prompt.clone()
    }

    fn get_max_tokens(&self) -> Option<u32> {
        // Test analysis processes test files only
        Some(6000)
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
