use std::{collections::HashMap, sync::Arc};

use client::{IntoLLMRequestParameters, LLMClient};
use domain::{CommitFileChange, CommitFileClassification, CommitSummaryError, CommitTestAnalysis};

use crate::summary::{
    compress_diff, format_file_summary, sample_files_by_change_volume,
    test_analyze::TestAnalyzeConversation, SamplingConfig,
};

/// 采样配置常量
const TEST_MAX_FULL_DIFF: usize = 8;
const TEST_MAX_LINES_PER_FILE: usize = 250;

/// 阶段二 2.4：测试文件分析服务
pub(crate) struct TestAnalyzeService {
    llm_client: Arc<dyn LLMClient>,
}

impl TestAnalyzeService {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    /// 对测试文件执行分析（使用智能采样机制）
    ///
    /// 若无测试文件，直接返回空 JSON。
    pub fn analyze(
        &self,
        stage1: &CommitFileClassification,
        file_diffs: &HashMap<String, String>,
        files: &[CommitFileChange],
        formatting_only_files: &[String],
    ) -> Result<String, CommitSummaryError> {
        let test_paths = &stage1.categories.by_nature.tests;
        if test_paths.is_empty() {
            return Ok("{}".to_string());
        }

        // 智能采样（排除格式化文件）
        let config = SamplingConfig {
            max_full_diff: TEST_MAX_FULL_DIFF,
            max_lines_per_file: TEST_MAX_LINES_PER_FILE,
        };
        let sampling =
            sample_files_by_change_volume(test_paths, files, &config, formatting_only_files);

        // Tier 1: 完整 diff
        let mut full_diff_parts = String::new();
        for path in &sampling.full_diff_files {
            let diff = file_diffs.get(*path).map(String::as_str).unwrap_or("");
            let compressed_diff = compress_diff(diff, config.max_lines_per_file);
            full_diff_parts.push_str(&format!(
                "\n### {}\n\n```diff\n{}\n```\n\n",
                path, compressed_diff
            ));
        }

        // Tier 2: 仅摘要
        let mut summary_parts = String::new();
        if !sampling.summary_only_files.is_empty() {
            summary_parts.push_str("\n### Other Test Files (Summary Only)\n");
            for path in &sampling.summary_only_files {
                summary_parts.push_str(&format!("{}\n", format_file_summary(path, files)));
            }
        }

        let user_prompt = format!(
            "## Test File Changes\n\n### Detailed Analysis\n{}\n{}\n",
            full_diff_parts, summary_parts
        );

        let conversation = TestAnalyzeConversation::new(user_prompt);
        let response = self
            .llm_client
            .call(&conversation.to_params())
            .map_err(|e| CommitSummaryError::LLMError(e.to_string()))?;
        let result: CommitTestAnalysis =
            response.to_model::<CommitTestAnalysis>().map_err(|e| {
                CommitSummaryError::ParseFailed(format!(
                    "Failed to parse test analysis results: {}",
                    e
                ))
            })?;
        serde_json::to_string(&result)
            .map_err(|e| CommitSummaryError::SerializeFailed(e.to_string()))
    }
}
