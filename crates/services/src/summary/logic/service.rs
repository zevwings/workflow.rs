//! 阶段二 2.2：核心逻辑分析服务
//!
//! 对业务代码、服务层等核心文件的完整 diff 进行深入分析。

use std::{collections::HashMap, sync::Arc};

use client::{IntoLLMRequestParameters, LLMClient};
use domain::{CommitFileChange, CommitFileClassification, CommitLogicAnalysis, CommitSummaryError};

use crate::summary::logic::LogicAnalyzeConversation;

/// 阶段二 2.2：核心逻辑分析服务
pub(crate) struct LogicAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
}

impl LogicAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    /// 对核心逻辑文件执行深入分析
    ///
    /// 若 `focus_group` 为空，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
    ) -> Result<String, CommitSummaryError> {
        let focus_group = &stage1.analysis_strategy.focus_group;
        if focus_group.is_empty() {
            return Ok("{}".to_string());
        }

        let mut parts = String::new();
        for path in focus_group {
            let additions =
                files.iter().find(|f| f.path == *path).and_then(|f| f.additions).unwrap_or(0);
            let deletions =
                files.iter().find(|f| f.path == *path).and_then(|f| f.deletions).unwrap_or(0);
            let diff = file_diffs.get(path).map(String::as_str).unwrap_or("");
            let file_type = infer_file_nature(path.as_str(), stage1);
            parts.push_str(&format!(
                r##"
### File: {}
Change scale: +{} -{}
File type: {}

#### Diff content:
```diff
{}
```

---
"##,
                path, additions, deletions, file_type, diff
            ));
        }

        let user_prompt = format!(
            r##"## Modified File List
{}
"##,
            parts
        );

        let conversation = LogicAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_client
            .call(&conversation.to_params())
            .map_err(|e| CommitSummaryError::LLMError(e.to_string()))?;
        let result: CommitLogicAnalysis =
            response.to_model::<CommitLogicAnalysis>().map_err(|e| {
                CommitSummaryError::ParseFailed(format!(
                    "Failed to parse logic analysis results: {}",
                    e
                ))
            })?;
        serde_json::to_string(&result)
            .map_err(|e| CommitSummaryError::SerializeFailed(e.to_string()))
    }
}

// ── Helpers ───────────────────────────────────────────────────

/// 根据阶段一 by_nature 分类推断文件性质
fn infer_file_nature(path: &str, stage1: &CommitFileClassification) -> &'static str {
    let n = &stage1.categories.by_nature;
    if n.business_logic.iter().any(|p| p.as_str() == path) {
        "Business Logic"
    } else if n.configuration.iter().any(|p| p.as_str() == path) {
        "Configuration"
    } else if n.tests.iter().any(|p| p.as_str() == path) {
        "Test"
    } else if n.documentation.iter().any(|p| p.as_str() == path) {
        "Documentation"
    } else if n.dependencies.iter().any(|p| p.as_str() == path) {
        "Dependencies/Build"
    } else if n.ui_style.iter().any(|p| p.as_str() == path) {
        "UI/Style"
    } else if n.infrastructure.iter().any(|p| p.as_str() == path) {
        "Infrastructure"
    } else {
        "Other"
    }
}
