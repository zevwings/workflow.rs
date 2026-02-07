//! 阶段二 2.3：配置/文档分析对话
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use domain::{CommitConfigAnalysis, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 配置/文档分析对话
///
/// 输入为已构建的 user prompt（包含配置或文档文件的 diff 列表），输出为配置变更与文档更新摘要。
pub(crate) struct ConfigAnalyzeConversation {
    user_prompt: String,
}

impl ConfigAnalyzeConversation {
    pub fn new(user_prompt: String) -> Self {
        Self { user_prompt }
    }
}

impl LLMConversation for ConfigAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        super::config_analyze().to_string()
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
