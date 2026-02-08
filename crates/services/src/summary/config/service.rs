//! 阶段二 2.3：配置/文档分析服务
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use std::collections::HashMap;
use std::sync::Arc;

use domain::errors::ServiceError;
use domain::git::entity::CommitFileChange;
use domain::summary::entity::{CommitConfigAnalysis, CommitFileClassification};
use llm::parsers::JsonParser;
use llm::LLMExecutor;

use super::ConfigAnalyzeConversation;

/// 阶段二 2.3：配置/文档分析服务
pub(crate) struct ConfigAnalyzeService {
    llm_executor: Arc<dyn LLMExecutor>,
}

impl ConfigAnalyzeService {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>) -> Self {
        Self { llm_executor }
    }

    /// 对配置和文档文件执行分析
    ///
    /// 若无配置/文档文件，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
        language_code: &str,
    ) -> Result<String, ServiceError> {
        let mut config_paths: Vec<&String> =
            stage1.categories.by_nature.configuration.iter().collect();
        config_paths.extend(stage1.categories.by_nature.documentation.iter());
        if config_paths.is_empty() {
            return Ok("{}".to_string());
        }

        let mut parts = String::new();
        for path in config_paths {
            let additions =
                files.iter().find(|f| f.path == *path).and_then(|f| f.additions).unwrap_or(0);
            let deletions =
                files.iter().find(|f| f.path == *path).and_then(|f| f.deletions).unwrap_or(0);
            let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
            parts.push_str(&format!(
                "\n### {}\n变更：+{} -{}\n\n```diff\n{}\n```\n\n---\n",
                path, additions, deletions, diff
            ));
        }

        let user_prompt = format!("## 修改文件\n{}\n", parts);
        let conversation = ConfigAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_executor
            .execute(&conversation, language_code, "config_analyze")
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        let result: CommitConfigAnalysis = JsonParser::to_model(&response)
            .map_err(|e| ServiceError::Other(format!("解析配置分析结果失败: {}", e)))?;
        serde_json::to_string(&result).map_err(|e| ServiceError::Other(e.to_string()))
    }
}
