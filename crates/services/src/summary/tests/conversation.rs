
use llm::LLMConversation;

use crate::summary::prompt;

/// 测试文件分析对话
pub(crate) struct TestAnalyzeConversation {
    user_prompt: String,
}

impl TestAnalyzeConversation {
    pub fn new(user_prompt: String) -> Self {
        Self { user_prompt }
    }
}

impl LLMConversation for TestAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::analyze_tests().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}
