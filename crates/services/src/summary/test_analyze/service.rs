use std::{collections::HashMap, sync::Arc};

use client::{IntoLLMRequestParameters, LLMClient};
use domain::{CommitFileClassification, CommitSummaryError, CommitTestAnalysis};

use crate::summary::test_analyze::TestAnalyzeConversation;

/// 阶段二 2.4：测试文件分析服务
pub(crate) struct TestAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
}

impl TestAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    /// 对测试文件执行分析
    ///
    /// 若无测试文件，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
    ) -> Result<String, CommitSummaryError> {
        let test_paths = &stage1.categories.by_nature.tests;
        if test_paths.is_empty() {
            return Ok("{}".to_string());
        }

        let mut combined = String::new();
        for path in test_paths {
            let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
            combined.push_str(&format!("\n### {}\n\n```diff\n{}\n```\n\n", path, diff));
        }

        let user_prompt = format!("## Test File Changes\n{}\n", combined);
        let conversation = TestAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_client
            .call(&conversation.to_params())
            .map_err(|e| CommitSummaryError::LLMError(e.to_string()))?;
        let result: CommitTestAnalysis =
            response.to_model::<CommitTestAnalysis>().map_err(|e| {
                CommitSummaryError::ParseFailed(format!(
                    "Failed to parse test analysis results: {}",
                    e
                ))
            })?;
        serde_json::to_string(&result)
            .map_err(|e| CommitSummaryError::SerializeFailed(e.to_string()))
    }
}
