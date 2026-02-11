//! 阶段二 2.1：批量操作分析服务
//!
//! 当阶段一检测到批量重命名、格式化、配置更新等模式时，对样本 diff 进行分析。

use std::{collections::HashMap, sync::Arc};

use domain::{CommitBatchAnalysis, CommitFileChange, CommitFileClassification, ServiceError};
use llm::{JsonParser, LLMExecutor};

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

        // 使用智能采样替换 take(3)
        let max_samples = 3;
        let sampling_result = select_representative_samples(batch_group, files, max_samples);
        let sample_paths = &sampling_result.selected_files;

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

        // 增强 user prompt，包含采样统计
        let user_prompt = format!(
            r##"## Batch Operation Information
- Operation type: {}
- Total files in batch: {}
- Sampled files: {}
- Files with zero changes: {}
- Operation pattern: {}

## Sample File Diffs ({} representative files)
Selection strategy: {}
{}
"##,
            pattern_type,
            batch_group.len(),
            sample_paths.len(),
            sampling_result.zero_change_count,
            pattern_desc,
            sample_paths.len(),
            sampling_result.strategy_description,
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

// ── Sampling Strategy ─────────────────────────────────────────

/// 采样结果
#[derive(Debug)]
struct SamplingResult<'a> {
    /// 选中的文件路径
    selected_files: Vec<&'a String>,
    /// 采样策略描述
    strategy_description: String,
    /// 零变更文件数量
    zero_change_count: usize,
}

/// 文件信息（用于采样）
#[derive(Debug, Clone)]
struct FileInfo<'a> {
    path: &'a String,
    change_amount: u32,
    directory: String,
}

/// 选择具有代表性的样本文件
///
/// 策略：
/// 1. 按变更量（additions + deletions）降序排序
/// 2. 确保样本来自不同子目录（提高多样性）
/// 3. 优先选择非零变更文件
fn select_representative_samples<'a>(
    batch_group: &'a [String],
    files: &[CommitFileChange],
    max_samples: usize,
) -> SamplingResult<'a> {
    use std::collections::HashSet;

    // 边界情况：文件数 ≤ max_samples，全部返回
    if batch_group.len() <= max_samples {
        let zero_count = count_zero_change_files(batch_group, files);
        return SamplingResult {
            selected_files: batch_group.iter().collect(),
            strategy_description: "All files included (total ≤ max samples)".to_string(),
            zero_change_count: zero_count,
        };
    }

    // 构建文件信息列表
    let mut file_infos: Vec<FileInfo<'a>> = batch_group
        .iter()
        .map(|path| {
            let file_change = files.iter().find(|f| &f.path == path);
            let additions = file_change.and_then(|f| f.additions).unwrap_or(0);
            let deletions = file_change.and_then(|f| f.deletions).unwrap_or(0);
            let change_amount = additions + deletions;
            let directory = extract_directory(path);

            FileInfo {
                path,
                change_amount,
                directory,
            }
        })
        .collect();

    let zero_change_count = file_infos.iter().filter(|f| f.change_amount == 0).count();

    // 策略1: 按变更量降序排序
    file_infos.sort_by(|a, b| b.change_amount.cmp(&a.change_amount));

    // 策略2: 确保目录多样性
    let mut selected = Vec::with_capacity(max_samples);
    let mut used_directories = HashSet::new();

    // 第一轮：选择不同目录的文件
    for info in &file_infos {
        if selected.len() >= max_samples {
            break;
        }
        if !used_directories.contains(&info.directory) {
            selected.push(info.path);
            used_directories.insert(info.directory.clone());
        }
    }

    // 第二轮：补齐到 max_samples
    if selected.len() < max_samples {
        for info in &file_infos {
            if selected.len() >= max_samples {
                break;
            }
            if !selected.contains(&info.path) {
                selected.push(info.path);
            }
        }
    }

    let strategy_desc = format!(
        "Top {} files by change volume (from {} directories)",
        selected.len(),
        used_directories.len()
    );

    SamplingResult {
        selected_files: selected,
        strategy_description: strategy_desc,
        zero_change_count,
    }
}

/// 提取文件所在目录
fn extract_directory(path: &str) -> String {
    if let Some(pos) = path.rfind('/') {
        path[..pos].to_string()
    } else {
        ".".to_string()
    }
}

/// 统计零变更文件数量
fn count_zero_change_files(paths: &[String], files: &[CommitFileChange]) -> usize {
    paths
        .iter()
        .filter(|path| {
            files
                .iter()
                .find(|f| &f.path == *path)
                .map(|f| {
                    let additions = f.additions.unwrap_or(0);
                    let deletions = f.deletions.unwrap_or(0);
                    additions + deletions == 0
                })
                .unwrap_or(true)
        })
        .count()
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use domain::CommitChangeType;

    use super::*;

    fn create_test_file(path: &str, additions: u32, deletions: u32) -> CommitFileChange {
        CommitFileChange {
            path: path.to_string(),
            change_type: CommitChangeType::Modified,
            old_path: None,
            additions: Some(additions),
            deletions: Some(deletions),
        }
    }

    #[test]
    fn test_select_samples_all_files_when_below_max() {
        let batch_group = vec!["file1.rs".to_string(), "file2.rs".to_string()];
        let files = vec![
            create_test_file("file1.rs", 10, 5),
            create_test_file("file2.rs", 20, 10),
        ];

        let result = select_representative_samples(&batch_group, &files, 3);

        assert_eq!(result.selected_files.len(), 2);
        assert!(result.strategy_description.contains("All files included"));
    }

    #[test]
    fn test_select_samples_by_change_volume() {
        let batch_group = vec![
            "a/file1.rs".to_string(),
            "b/file2.rs".to_string(),
            "c/file3.rs".to_string(),
            "d/file4.rs".to_string(),
        ];
        let files = vec![
            create_test_file("a/file1.rs", 5, 2),    // 7
            create_test_file("b/file2.rs", 50, 30),  // 80
            create_test_file("c/file3.rs", 20, 10),  // 30
            create_test_file("d/file4.rs", 100, 50), // 150
        ];

        let result = select_representative_samples(&batch_group, &files, 2);

        assert_eq!(result.selected_files.len(), 2);
        // 应选择变更量最大的两个：file4 (150) 和 file2 (80)
        assert!(result.selected_files.contains(&&"d/file4.rs".to_string()));
        assert!(result.selected_files.contains(&&"b/file2.rs".to_string()));
    }

    #[test]
    fn test_select_samples_directory_diversity() {
        let batch_group = vec![
            "src/a/file1.rs".to_string(),
            "src/a/file2.rs".to_string(),
            "src/b/file3.rs".to_string(),
            "src/c/file4.rs".to_string(),
        ];
        let files = vec![
            create_test_file("src/a/file1.rs", 100, 50),
            create_test_file("src/a/file2.rs", 80, 40),
            create_test_file("src/b/file3.rs", 60, 30),
            create_test_file("src/c/file4.rs", 40, 20),
        ];

        let result = select_representative_samples(&batch_group, &files, 3);

        assert_eq!(result.selected_files.len(), 3);
        // 应优先保证目录多样性：file1 (src/a), file3 (src/b), file4 (src/c)
        let selected_dirs: std::collections::HashSet<_> =
            result.selected_files.iter().map(|p| extract_directory(p)).collect();
        assert_eq!(selected_dirs.len(), 3);
    }

    #[test]
    fn test_count_zero_change_files() {
        let paths = vec![
            "file1.rs".to_string(),
            "file2.rs".to_string(),
            "file3.rs".to_string(),
        ];
        let files = vec![
            create_test_file("file1.rs", 0, 0),
            create_test_file("file2.rs", 10, 5),
            create_test_file("file3.rs", 0, 0),
        ];

        let count = count_zero_change_files(&paths, &files);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_extract_directory() {
        assert_eq!(extract_directory("src/main.rs"), "src");
        assert_eq!(extract_directory("src/services/summary.rs"), "src/services");
        assert_eq!(extract_directory("main.rs"), ".");
        assert_eq!(extract_directory("a/b/c/d/e.rs"), "a/b/c/d");
    }
}
