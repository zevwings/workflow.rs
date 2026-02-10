use std::sync::Arc;

use domain::{CommitSummaryAnalysis, ServiceError};
use llm::{JsonParser, LLMExecutor};

use super::{SummaryAnalyzeConversation, SummaryAnalyzeInput};

// ── Service ───────────────────────────────────────────────────

/// 阶段三：全局总结服务
pub(crate) struct SummaryAnalyzeService {
    llm_executor: Arc<dyn LLMExecutor>,
}

impl SummaryAnalyzeService {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>) -> Self {
        Self { llm_executor }
    }

    /// 综合各阶段结果生成全局总结
    pub fn summarize(
        &self,
        input: SummaryAnalyzeInput,
        language_code: &str,
    ) -> Result<CommitSummaryAnalysis, ServiceError> {
        let conversation = SummaryAnalyzeConversation::new(input);
        let response = self
            .llm_executor
            .execute(&conversation, language_code, "summary_analyze")
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        JsonParser::to_model(&response).map_err(|e| {
            ServiceError::Other(format!("Failed to parse summary analysis results: {}", e))
        })
    }
}
