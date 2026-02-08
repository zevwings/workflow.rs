//! 阶段二 2.2：核心逻辑分析服务
//!
//! 对业务代码、服务层等核心文件的完整 diff 进行深入分析。

use std::collections::HashMap;
use std::sync::Arc;

use domain::errors::ServiceError;
use domain::git::entity::CommitFileChange;
use domain::summary::entity::{CommitFileClassification, CommitLogicAnalysis};
use llm::parsers::JsonParser;
use llm::LLMExecutor;

use super::LogicAnalyzeConversation;

/// 阶段二 2.2：核心逻辑分析服务
pub(crate) struct LogicAnalyzeService {
    llm_executor: Arc<dyn LLMExecutor>,
}

impl LogicAnalyzeService {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>) -> Self {
        Self { llm_executor }
    }

    /// 对核心逻辑文件执行深入分析
    ///
    /// 若 `focus_group` 为空，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
        language_code: &str,
    ) -> Result<String, ServiceError> {
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
### 文件：{}
修改规模：+{} -{}
文件类型：{}

#### Diff内容：
```diff
{}
```

---
"##,
                path, additions, deletions, file_type, diff
            ));
        }

        let user_prompt = format!(
            r##"## 修改文件列表
{}
"##,
            parts
        );

        let conversation = LogicAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_executor
            .execute(&conversation, language_code, "logic_analyze")
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        let result: CommitLogicAnalysis = JsonParser::to_model(&response)
            .map_err(|e| ServiceError::Other(format!("解析核心逻辑分析结果失败: {}", e)))?;
        serde_json::to_string(&result).map_err(|e| ServiceError::Other(e.to_string()))
    }
}

// ── Helpers ───────────────────────────────────────────────────

/// 根据阶段一 by_nature 分类推断文件性质
fn infer_file_nature(path: &str, stage1: &CommitFileClassification) -> &'static str {
    let n = &stage1.categories.by_nature;
    if n.business_logic.iter().any(|p| p.as_str() == path) {
        "核心业务逻辑"
    } else if n.configuration.iter().any(|p| p.as_str() == path) {
        "配置文件"
    } else if n.tests.iter().any(|p| p.as_str() == path) {
        "测试文件"
    } else if n.documentation.iter().any(|p| p.as_str() == path) {
        "文档"
    } else if n.dependencies.iter().any(|p| p.as_str() == path) {
        "依赖/构建"
    } else if n.ui_style.iter().any(|p| p.as_str() == path) {
        "UI/样式"
    } else if n.infrastructure.iter().any(|p| p.as_str() == path) {
        "基础设施"
    } else {
        "其他"
    }
}
