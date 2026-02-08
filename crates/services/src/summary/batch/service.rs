//! 阶段二 2.1：批量操作分析服务
//!
//! 当阶段一检测到批量重命名、格式化、配置更新等模式时，对样本 diff 进行分析。

use std::collections::HashMap;
use std::sync::Arc;

use domain::errors::ServiceError;
use domain::git::entity::CommitFileChange;
use domain::summary::entity::{CommitBatchAnalysis, CommitFileClassification};
use llm::parsers::JsonParser;
use llm::LLMExecutor;

use super::BatchAnalyzeConversation;

// ── Service ───────────────────────────────────────────────────

/// 阶段二 2.1：批量操作分析服务
pub(crate) struct BatchAnalyzeService {
    llm_executor: Arc<dyn LLMExecutor>,
}

impl BatchAnalyzeService {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>) -> Self {
        Self { llm_executor }
    }

    /// 对批量操作文件执行分析
    ///
    /// 若 `batch_group` 为空，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
        language_code: &str,
    ) -> Result<String, ServiceError> {
        let batch_group = &stage1.analysis_strategy.batch_group;
        if batch_group.is_empty() {
            return Ok("{}".to_string());
        }

        let pattern_type = detect_pattern_type(stage1);
        let pattern_desc = build_batch_pattern_description(stage1);
        let sample_paths: Vec<&String> = batch_group.iter().take(3).collect();

        let mut sample_diffs = String::new();
        for (i, path) in sample_paths.iter().enumerate() {
            let additions =
                files.iter().find(|f| f.path == **path).and_then(|f| f.additions).unwrap_or(0);
            let deletions =
                files.iter().find(|f| f.path == **path).and_then(|f| f.deletions).unwrap_or(0);
            let diff = file_diffs.get(*path).map(String::as_str).unwrap_or("");
            sample_diffs.push_str(&format!(
                "\n### File {}: {}\nChanges: +{} -{}\n```diff\n{}\n```\n",
                i + 1,
                path,
                additions,
                deletions,
                diff
            ));
        }

        let user_prompt = format!(
            r##"## Batch Operation Information
- Operation type: {}
- Number of files affected: {}
- Operation pattern: {}

## Sample File Diffs (first {} representative files)
{}
"##,
            pattern_type,
            batch_group.len(),
            pattern_desc,
            sample_paths.len(),
            sample_diffs
        );

        let conversation = BatchAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_executor
            .execute(&conversation, language_code, "batch_analyze")
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        let result: CommitBatchAnalysis = JsonParser::to_model(&response).map_err(|e| {
            ServiceError::Other(format!("Failed to parse batch analysis results: {}", e))
        })?;
        serde_json::to_string(&result).map_err(|e| ServiceError::Other(e.to_string()))
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn detect_pattern_type(stage1: &CommitFileClassification) -> &'static str {
    if stage1.patterns.mass_rename.detected {
        "Mass Rename"
    } else if stage1.patterns.formatting.detected {
        "Mass Formatting"
    } else if stage1.patterns.config_update.detected {
        "Unified Configuration Update"
    } else if stage1.patterns.dependency_upgrade.detected {
        "Dependency Version Upgrade"
    } else if stage1.patterns.import_path_change.detected {
        "Import Path Adjustment"
    } else {
        "Batch Operation"
    }
}

/// 综合 5 种批量操作模式的描述
fn build_batch_pattern_description(stage1: &CommitFileClassification) -> String {
    let p = &stage1.patterns;
    // 最多5种批量操作模式
    let mut parts: Vec<String> = Vec::with_capacity(5);
    if p.mass_rename.detected && !p.mass_rename.pattern.is_empty() {
        parts.push(format!(
            "Mass rename: {} ({} files affected)",
            p.mass_rename.pattern, p.mass_rename.affected_files
        ));
    }
    if p.formatting.detected && !p.formatting.description.is_empty() {
        parts.push(format!("Mass formatting: {}", p.formatting.description));
    }
    if p.config_update.detected && !p.config_update.type_desc.is_empty() {
        parts.push(format!(
            "Unified configuration update: {}",
            p.config_update.type_desc
        ));
    }
    if p.dependency_upgrade.detected && !p.dependency_upgrade.packages.is_empty() {
        parts.push(format!(
            "Dependency version upgrade: {}",
            p.dependency_upgrade.packages.join(", ")
        ));
    }
    if p.import_path_change.detected && !p.import_path_change.pattern.is_empty() {
        parts.push(format!(
            "Import path adjustment: {}",
            p.import_path_change.pattern
        ));
    }
    if parts.is_empty() {
        "(Stage 1 did not identify specific patterns, please summarize based on sample diffs)"
            .to_string()
    } else {
        parts.join("; ")
    }
}
