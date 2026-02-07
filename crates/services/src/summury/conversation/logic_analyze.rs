//! 阶段二 2.2：核心逻辑分析对话
//!
//! 对业务代码、服务层等核心文件的完整 diff 进行深入分析。

use domain::{CommitLogicAnalysis, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 核心逻辑分析对话
///
/// 输入为已构建的 user prompt（包含修改文件列表及每个文件的 diff），输出为每个文件的详细分析。
pub(crate) struct LogicAnalyzeConversation {
    user_prompt: String,
}

impl LogicAnalyzeConversation {
    pub fn new(user_prompt: String) -> Self {
        Self { user_prompt }
    }
}

impl LLMConversation for LogicAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        super::logic_analyze().to_string()
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
