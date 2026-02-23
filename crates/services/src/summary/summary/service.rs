use std::sync::Arc;

use client::{IntoLLMRequestParameters, LLMClient, SupportedLanguage};
use domain::{CommitSummaryAnalysis, CommitSummaryError};

use crate::summary::summary::{SummaryAnalyzeConversation, SummaryAnalyzeInput};

// ── Service ───────────────────────────────────────────────────

/// 阶段三：全局总结服务
pub(crate) struct SummaryAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
    language: SupportedLanguage,
}

impl SummaryAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>, language: SupportedLanguage) -> Self {
        Self {
            llm_client,
            language,
        }
    }

    /// 综合各阶段结果生成全局总结
    pub fn summarize(
        &self,
        input: SummaryAnalyzeInput,
    ) -> Result<CommitSummaryAnalysis, CommitSummaryError> {
        let conversation = SummaryAnalyzeConversation::new(input, self.language.clone());
        let response = self
            .llm_client
            .call(&conversation.to_params())
            .map_err(|e| CommitSummaryError::LLMError(e.to_string()))?;
        response.to_model::<CommitSummaryAnalysis>().map_err(|e| {
            CommitSummaryError::ParseFailed(format!(
                "Failed to parse summary analysis results: {}",
                e
            ))
        })
    }
}
