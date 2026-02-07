use std::sync::Arc;

use domain::{sanitize_branch_name, BranchService, BranchServiceError};
use llm::parsers::JsonParser;
use llm::{LLMConfigContext, LLMExecutor};

use super::BranchNameConversation;

pub struct BranchServiceImpl {
    llm_executor: Arc<dyn LLMExecutor>,
    llm_context: Arc<dyn LLMConfigContext>,
}

impl BranchServiceImpl {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>, llm_context: Arc<dyn LLMConfigContext>) -> Self {
        Self {
            llm_executor,
            llm_context,
        }
    }
}

impl BranchService for BranchServiceImpl {
    fn generate_branch_name(
        &self,
        title: Option<&str>,
        exists_branches: &[String],
    ) -> Result<String, BranchServiceError> {
        let conversation = BranchNameConversation::new((
            title.map(String::from),
            if exists_branches.is_empty() {
                None
            } else {
                Some(exists_branches.to_vec())
            },
        ));

        // 执行 LLM 调用
        let response = self
            .llm_executor
            .execute(
                &conversation,
                &self.llm_context.get_language(),
                "Generate branch name",
            )
            .map_err(|e| BranchServiceError::LLMError(e.to_string()))?;

        // 转换为 map 对象
        let map = JsonParser::to_map(response)
            .map_err(|e| BranchServiceError::JsonParseFailed(format!("JSON parse error: {}", e)))?;

        // 提取分支名字段
        let branch_name = map.get("branch_name").and_then(|v| v.as_str()).ok_or_else(|| {
            BranchServiceError::GenerateBranchNameFailed(
                "branch_name is missing or empty".to_string(),
            )
        })?;

        // 清理分支名，确保只保留 ASCII 字符
        let cleaned_branch_name: String = sanitize_branch_name(branch_name.trim());

        Ok(cleaned_branch_name)
    }
}
