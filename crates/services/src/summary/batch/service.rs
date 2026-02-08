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
                "\n### 文件{}: {}\n变更：+{} -{}\n```diff\n{}\n```\n",
                i + 1,
                path,
                additions,
                deletions,
                diff
            ));
        }

        let user_prompt = format!(
            r##"## 批量操作信息
- 操作类型：{}
- 涉及文件数：{}
- 操作模式：{}

## 样本文件Diff（前{}个代表性文件）
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
        let result: CommitBatchAnalysis = JsonParser::to_model(&response)
            .map_err(|e| ServiceError::Other(format!("解析批量分析结果失败: {}", e)))?;
        serde_json::to_string(&result).map_err(|e| ServiceError::Other(e.to_string()))
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn detect_pattern_type(stage1: &CommitFileClassification) -> &'static str {
    if stage1.patterns.mass_rename.detected {
        "批量重命名"
    } else if stage1.patterns.formatting.detected {
        "批量格式化"
    } else if stage1.patterns.config_update.detected {
        "统一配置更新"
    } else if stage1.patterns.dependency_upgrade.detected {
        "依赖版本升级"
    } else if stage1.patterns.import_path_change.detected {
        "导入路径调整"
    } else {
        "批量操作"
    }
}

/// 综合 5 种批量操作模式的描述
fn build_batch_pattern_description(stage1: &CommitFileClassification) -> String {
    let p = &stage1.patterns;
    // 最多5种批量操作模式
    let mut parts: Vec<String> = Vec::with_capacity(5);
    if p.mass_rename.detected && !p.mass_rename.pattern.is_empty() {
        parts.push(format!(
            "批量重命名：{}（涉及 {} 个文件）",
            p.mass_rename.pattern, p.mass_rename.affected_files
        ));
    }
    if p.formatting.detected && !p.formatting.description.is_empty() {
        parts.push(format!("批量格式化：{}", p.formatting.description));
    }
    if p.config_update.detected && !p.config_update.type_desc.is_empty() {
        parts.push(format!("统一配置更新：{}", p.config_update.type_desc));
    }
    if p.dependency_upgrade.detected && !p.dependency_upgrade.packages.is_empty() {
        parts.push(format!(
            "依赖版本升级：{}",
            p.dependency_upgrade.packages.join(", ")
        ));
    }
    if p.import_path_change.detected && !p.import_path_change.pattern.is_empty() {
        parts.push(format!("导入路径调整：{}", p.import_path_change.pattern));
    }
    if parts.is_empty() {
        "（阶段一未识别到具体模式，请根据样本 diff 归纳）".to_string()
    } else {
        parts.join("；")
    }
}
