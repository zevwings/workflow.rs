//! 阶段二 2.4：测试文件分析对话
//!
//! 分析测试文件的变更及与业务代码的对应关系。

use domain::{CommitTestAnalysis, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 测试文件分析对话
///
/// 输入为已构建的 user prompt（包含测试文件的 diff），输出为测试摘要及与代码变更的匹配度。
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
        super::test_analyze().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))
    }
}
