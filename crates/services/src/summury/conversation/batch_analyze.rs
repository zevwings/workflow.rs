//! 阶段二 2.1：批量操作分析对话
//!
//! 当阶段一检测到批量重命名、格式化、配置更新等模式时，对样本 diff 进行分析。

use domain::{CommitBatchAnalysis, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 批量操作分析对话
///
/// 输入为已构建的 user prompt（包含批量操作信息 + 样本文件 diff），输出为结构化分析结果。
pub(crate) struct BatchAnalyzeConversation {
    user_prompt: String,
}

impl BatchAnalyzeConversation {
    pub fn new(user_prompt: String) -> Self {
        Self { user_prompt }
    }
}

impl LLMConversation for BatchAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        super::batch_analyze().to_string()
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
