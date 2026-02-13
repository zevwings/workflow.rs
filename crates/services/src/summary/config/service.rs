//! 阶段二 2.3：配置/文档分析服务
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use std::{collections::HashMap, sync::Arc};

use client::{IntoLLMRequestParameters, LLMClient};
use domain::{
    CommitConfigAnalysis, CommitFileChange, CommitFileClassification, CommitSummaryError,
};

use crate::summary::{
    compress_diff, config::ConfigAnalyzeConversation, format_file_summary,
    sample_files_by_change_volume, SamplingConfig,
};

/// 采样配置常量
const CONFIG_MAX_FULL_DIFF: usize = 5;
const CONFIG_MAX_LINES_PER_FILE: usize = 200;

/// 阶段二 2.3：配置/文档分析服务
pub(crate) struct ConfigAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
}

impl ConfigAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    /// 对配置和文档文件执行分析（使用智能采样机制）
    ///
    /// 若无配置/文档文件，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
        formatting_only_files: &[String],
    ) -> Result<String, CommitSummaryError> {
        // 过滤 lock 文件
        let config_paths: Vec<String> = stage1
            .categories
            .by_nature
            .configuration
            .iter()
            .filter(|p| !is_lock_file(p))
            .cloned()
            .collect();
        let doc_paths: Vec<&String> = stage1.categories.by_nature.documentation.iter().collect();

        if config_paths.is_empty() && doc_paths.is_empty() {
            return Ok("{}".to_string());
        }

        // 配置文件智能采样（排除格式化文件）
        let sampling_config = SamplingConfig {
            max_full_diff: CONFIG_MAX_FULL_DIFF,
            max_lines_per_file: CONFIG_MAX_LINES_PER_FILE,
        };
        let sampling = sample_files_by_change_volume(
            &config_paths,
            files,
            &sampling_config,
            formatting_only_files,
        );

        // Tier 1: 完整 diff
        let mut config_parts = String::new();
        for path in &sampling.full_diff_files {
            let additions =
                files.iter().find(|f| &f.path == *path).and_then(|f| f.additions).unwrap_or(0);
            let deletions =
                files.iter().find(|f| &f.path == *path).and_then(|f| f.deletions).unwrap_or(0);
            let diff = file_diffs.get(*path).map(String::as_str).unwrap_or("");
            let compressed_diff = compress_diff(diff, sampling_config.max_lines_per_file);
            config_parts.push_str(&format!(
                "\n### {}\nChanges: +{} -{}\n\n```diff\n{}\n```\n\n---\n",
                path, additions, deletions, compressed_diff
            ));
        }

        // Tier 2: 仅摘要
        if !sampling.summary_only_files.is_empty() {
            config_parts.push_str("\n### Other Config Files (Summary Only)\n");
            for path in &sampling.summary_only_files {
                config_parts.push_str(&format!("{}\n", format_file_summary(path, files)));
            }
        }

        // 文档文件：仅发送路径 + 变更统计
        let mut doc_parts = String::new();
        for path in &doc_paths {
            doc_parts.push_str(&format!("{}\n", format_file_summary(path, files)));
        }

        let user_prompt = format!(
            "## Configuration File Changes\n{}\n\n## Documentation File Changes (summary only)\n{}\n",
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

// ── Helpers ───────────────────────────────────────────────────

/// 判断是否为 lock 文件
fn is_lock_file(path: &str) -> bool {
    path.ends_with(".lock")
        || path.ends_with("package-lock.json")
        || path.ends_with("yarn.lock")
        || path.ends_with("Cargo.lock")
        || path.ends_with("pnpm-lock.yaml")
}
