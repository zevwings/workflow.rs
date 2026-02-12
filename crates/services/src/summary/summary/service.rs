use std::sync::Arc;

use domain::{CommitSummaryAnalysis, CommitSummaryError};
use llm::{IntoLLMRequestParameters, JsonParser, LLMClient};

use crate::summary::summary::{SummaryAnalyzeConversation, SummaryAnalyzeInput};

// ── Service ───────────────────────────────────────────────────

/// 阶段三：全局总结服务
pub(crate) struct SummaryAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
}

impl SummaryAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    /// 综合各阶段结果生成全局总结
    pub fn summarize(
        &self,
        input: SummaryAnalyzeInput,
        language_code: &str,
    ) -> Result<CommitSummaryAnalysis, CommitSummaryError> {
        let conversation = SummaryAnalyzeConversation::new(input);
        let response = self
            .llm_client
            .call(&conversation.to_params(language_code))
            .map_err(|e| CommitSummaryError::LLMError(e.to_string()))?;
        JsonParser::to_model(&response).map_err(|e| {
            CommitSummaryError::ParseFailed(format!(
                "Failed to parse summary analysis results: {}",
                e
            ))
        })
    }
}
