use std::sync::Arc;

use domain::{sanitize_branch_name, BranchService, BranchServiceError};
use llm::{IntoLLMRequestParameters, JsonParser, LLMClient, LLMConfigContext};

use crate::branch::conversation::BranchNameConversation;

/// 分支服务实现
///
/// 使用 LLM 生成语义化的分支名称。
pub(crate) struct BranchServiceImpl {
    llm_client: Arc<dyn LLMClient>,
    llm_context: Arc<dyn LLMConfigContext>,
}

impl BranchServiceImpl {
    pub fn new(llm_client: Arc<dyn LLMClient>, llm_context: Arc<dyn LLMConfigContext>) -> Self {
        Self {
            llm_client,
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
        let conversation = BranchNameConversation::new(title, exists_branches);

        // 执行 LLM 调用
        let response = self
            .llm_client
            .call(&conversation.to_params(&self.llm_context.get_language()))
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
