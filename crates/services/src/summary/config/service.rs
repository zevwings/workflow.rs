//! 阶段二 2.3：配置/文档分析服务
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use std::{collections::HashMap, sync::Arc};

use client::{IntoLLMRequestParameters, LLMClient};
use domain::{
    CommitConfigAnalysis, CommitFileChange, CommitFileClassification, CommitSummaryError,
};

use crate::summary::config::ConfigAnalyzeConversation;

/// 阶段二 2.3：配置/文档分析服务
pub(crate) struct ConfigAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
}

impl ConfigAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    /// 对配置和文档文件执行分析
    ///
    /// 若无配置/文档文件，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
    ) -> Result<String, CommitSummaryError> {
        let config_paths: Vec<&String> = stage1.categories.by_nature.configuration.iter().collect();
        let doc_paths: Vec<&String> = stage1.categories.by_nature.documentation.iter().collect();

        if config_paths.is_empty() && doc_paths.is_empty() {
            return Ok("{}".to_string());
        }

        // 配置文件：发送完整 diff
        let mut config_parts = String::new();
        for path in &config_paths {
            let additions =
                files.iter().find(|f| f.path == **path).and_then(|f| f.additions).unwrap_or(0);
            let deletions =
                files.iter().find(|f| f.path == **path).and_then(|f| f.deletions).unwrap_or(0);
            let diff = file_diffs.get(*path).map(String::as_str).unwrap_or("");
            config_parts.push_str(&format!(
                "\n### {}\nChanges: +{} -{}\n\n```diff\n{}\n```\n\n---\n",
                path, additions, deletions, diff
            ));
        }

        // 文档文件：仅发送路径 + 变更统计（不发送完整 diff）
        let mut doc_parts = String::new();
        for path in &doc_paths {
            let additions =
                files.iter().find(|f| f.path == **path).and_then(|f| f.additions).unwrap_or(0);
            let deletions =
                files.iter().find(|f| f.path == **path).and_then(|f| f.deletions).unwrap_or(0);
            let status = if additions > 0 && deletions == 0 {
                "Added"
            } else if additions == 0 && deletions > 0 {
                "Deleted"
            } else {
                "Modified"
            };
            doc_parts.push_str(&format!(
                "- {} [{}] (+{} -{})\n",
                path, status, additions, deletions
            ));
        }

        let user_prompt = format!(
            "## Configuration File Changes\n{}\n\n## Documentation File Changes (summary only, no need for deep content analysis)\n{}\n",
            config_parts, doc_parts
        );
        let conversation = ConfigAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_client
            .call(&conversation.to_params())
            .map_err(|e| CommitSummaryError::LLMError(e.to_string()))?;
        let result: CommitConfigAnalysis =
            response.to_model::<CommitConfigAnalysis>().map_err(|e| {
                CommitSummaryError::ParseFailed(format!(
                    "Failed to parse configuration analysis results: {}",
                    e
                ))
            })?;
        serde_json::to_string(&result)
            .map_err(|e| CommitSummaryError::SerializeFailed(e.to_string()))
    }
}
