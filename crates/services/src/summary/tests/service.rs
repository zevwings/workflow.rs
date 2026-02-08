use std::collections::HashMap;
use std::sync::Arc;

use domain::errors::ServiceError;
use domain::summary::entity::{CommitFileClassification, CommitTestAnalysis};
use llm::parsers::JsonParser;
use llm::LLMExecutor;

use super::TestAnalyzeConversation;

/// 阶段二 2.4：测试文件分析服务
pub(crate) struct TestAnalyzeService {
    llm_executor: Arc<dyn LLMExecutor>,
}

impl TestAnalyzeService {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>) -> Self {
        Self { llm_executor }
    }

    /// 对测试文件执行分析
    ///
    /// 若无测试文件，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        language_code: &str,
    ) -> Result<String, ServiceError> {
        let test_paths = &stage1.categories.by_nature.tests;
        if test_paths.is_empty() {
            return Ok("{}".to_string());
        }

        let mut combined = String::new();
        for path in test_paths {
            let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
            combined.push_str(&format!("\n### {}\n\n```diff\n{}\n```\n\n", path, diff));
        }

        let user_prompt = format!("## 测试文件变更\n{}\n", combined);
        let conversation = TestAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_executor
            .execute(&conversation, language_code, "test_analyze")
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        let result: CommitTestAnalysis = JsonParser::to_model(&response)
            .map_err(|e| ServiceError::Other(format!("解析测试分析结果失败: {}", e)))?;
        serde_json::to_string(&result).map_err(|e| ServiceError::Other(e.to_string()))
    }
}
