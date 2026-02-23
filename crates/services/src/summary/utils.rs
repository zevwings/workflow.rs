//! 通用的 diff 采样工具

use domain::CommitFileChange;
use std::collections::HashSet;

/// 采样结果
pub(crate) struct DiffSamplingResult<'a> {
    /// 完整 diff 的文件（按变更量降序，保证目录多样性）
    pub full_diff_files: Vec<&'a String>,
    /// 仅摘要的文件
    pub summary_only_files: Vec<&'a String>,
    /// 采样策略描述
    pub strategy_description: String,
    /// 零变更文件数量
    pub zero_change_count: usize,
    /// 格式化文件数量
    pub formatting_only_count: usize,
}

/// 智能采样配置
pub(crate) struct SamplingConfig {
    /// 最多包含完整 diff 的文件数
    pub max_full_diff: usize,
    /// 单文件 diff 最大行数（超过则截断）
    pub max_lines_per_file: usize,
}

/// 文件信息（用于采样）
#[derive(Debug, Clone)]
struct FileInfo<'a> {
    path: &'a String,
    change_amount: u32,
    directory: String,
    priority: f32,
    weighted_score: f32,
}

/// 智能采样：选择变更量最大的文件发送完整 diff
///
/// 采样策略：
/// 1. 按加权评分（变更量 × 目录优先级）降序排序
/// 2. 确保样本来自不同目录（提高多样性）
/// 3. 统计零变更文件数量
/// 4. 排除纯格式化文件
pub(crate) fn sample_files_by_change_volume<'a>(
    file_paths: &'a [String],
    all_files: &[CommitFileChange],
    config: &SamplingConfig,
    formatting_only_files: &[String],
) -> DiffSamplingResult<'a> {
    // 过滤掉纯格式化文件
    let meaningful_files: Vec<&String> =
        file_paths.iter().filter(|path| !formatting_only_files.contains(path)).collect();

    let formatting_count = file_paths.len() - meaningful_files.len();

    // 边界情况：过滤后的文件数 ≤ max_full_diff，全部返回
    if meaningful_files.len() <= config.max_full_diff {
        let zero_count = count_zero_change_files(&meaningful_files, all_files);
        let meaningful_count = meaningful_files.len();
        return DiffSamplingResult {
            full_diff_files: meaningful_files,
            summary_only_files: Vec::new(),
            strategy_description: format!(
                "All {} files included (total ≤ max samples, {} formatting-only excluded)",
                meaningful_count, formatting_count
            ),
            zero_change_count: zero_count,
            formatting_only_count: formatting_count,
        };
    }

    // 构建文件信息列表（仅包含有意义的文件）
    let mut file_infos: Vec<FileInfo<'a>> = meaningful_files
        .iter()
        .map(|path| {
            let file_change = all_files.iter().find(|f| &f.path == *path);
            let additions = file_change.and_then(|f| f.additions).unwrap_or(0);
            let deletions = file_change.and_then(|f| f.deletions).unwrap_or(0);
            let change_amount = additions + deletions;
            let directory = extract_directory(path);
            let priority = calculate_directory_priority(path);
            let weighted_score = change_amount as f32 * priority;

            FileInfo {
                path,
                change_amount,
                directory,
                priority,
                weighted_score,
            }
        })
        .collect();

    let zero_change_count = file_infos.iter().filter(|f| f.change_amount == 0).count();

    // 策略1: 按加权评分降序排序（变更量 × 优先级）
    file_infos.sort_by(|a, b| {
        b.weighted_score
            .partial_cmp(&a.weighted_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 策略2: 确保目录多样性
    let mut selected = Vec::with_capacity(config.max_full_diff);
    let mut used_directories = HashSet::new();

    // 第一轮：优先选择不同目录的文件
    for info in &file_infos {
        if selected.len() >= config.max_full_diff {
            break;
        }
        if !used_directories.contains(&info.directory) {
            selected.push(info.path);
            used_directories.insert(info.directory.clone());
        }
    }

    // 第二轮：按变更量补齐到 max_full_diff
    if selected.len() < config.max_full_diff {
        for info in &file_infos {
            if selected.len() >= config.max_full_diff {
                break;
            }
            if !selected.contains(&info.path) {
                selected.push(info.path);
            }
        }
    }

    // 未被选中的文件
    let summary_only: Vec<&String> = file_infos
        .iter()
        .map(|info| info.path)
        .filter(|path| !selected.contains(path))
        .collect();

    // 计算平均优先级（用于展示）
    let avg_priority = if !selected.is_empty() {
        let total_priority: f32 = selected
            .iter()
            .filter_map(|path| file_infos.iter().find(|info| info.path == *path))
            .map(|info| info.priority)
            .sum();
        total_priority / selected.len() as f32
    } else {
        1.0
    };

    let strategy_desc = format!(
        "Top {} files by weighted score (from {} directories, avg priority: {:.2}x, {} formatting-only excluded)",
        selected.len(),
        used_directories.len(),
        avg_priority,
        formatting_count
    );

    DiffSamplingResult {
        full_diff_files: selected,
        summary_only_files: summary_only,
        strategy_description: strategy_desc,
        zero_change_count,
        formatting_only_count: formatting_count,
    }
}

/// 提取文件所在目录路径
fn extract_directory(path: &str) -> String {
    if let Some(pos) = path.rfind('/') {
        path[..pos].to_string()
    } else {
        ".".to_string()
    }
}

/// 统计零变更文件数量
fn count_zero_change_files(paths: &[&String], files: &[CommitFileChange]) -> usize {
    paths
        .iter()
        .filter(|path| {
            files
                .iter()
                .find(|f| &f.path == **path)
                .map(|f| {
                    let additions = f.additions.unwrap_or(0);
                    let deletions = f.deletions.unwrap_or(0);
                    additions + deletions == 0
                })
                .unwrap_or(true)
        })
        .count()
}

/// 压缩过长的 diff
pub(crate) fn compress_diff(diff: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = diff.lines().collect();

    if lines.len() <= max_lines {
        return diff.to_string();
    }

    let head_lines = max_lines / 2;
    let tail_lines = max_lines - head_lines;
    let omitted = lines.len() - max_lines;

    format!(
        "{}\n\n... [Omitted {} lines] ...\n\n{}",
        lines[..head_lines].join("\n"),
        omitted,
        lines[lines.len() - tail_lines..].join("\n")
    )
}

/// 格式化摘要信息（仅路径和统计）
pub(crate) fn format_file_summary(path: &str, files: &[CommitFileChange]) -> String {
    let file = files.iter().find(|f| f.path == path);
    let additions = file.and_then(|f| f.additions).unwrap_or(0);
    let deletions = file.and_then(|f| f.deletions).unwrap_or(0);
    format!("- {} (+{} -{})", path, additions, deletions)
}

/// Pre-filter files for extremely large commits (>500 files)
///
/// Removes obviously unimportant files to reduce token usage in stage 1.
/// Only applies aggressive filtering for very large commits.
pub(crate) fn prefilter_files_for_large_commits(
    files: Vec<CommitFileChange>,
) -> Vec<CommitFileChange> {
    // Only apply pre-filtering for extremely large commits
    if files.len() <= 500 {
        return files;
    }

    files
        .into_iter()
        .filter(|file| {
            let path = &file.path;
            let path_lower = path.to_lowercase();

            // Keep if has significant changes
            let has_changes = file.additions.unwrap_or(0) + file.deletions.unwrap_or(0) > 0;

            // Filter out lock files
            let is_lock_file = path.ends_with(".lock")
                || path.ends_with("package-lock.json")
                || path.ends_with("yarn.lock")
                || path.ends_with("pnpm-lock.yaml")
                || path.ends_with("Cargo.lock");

            // Filter out obvious generated/build files
            let is_generated = path_lower.contains("/target/")
                || path_lower.contains("/build/")
                || path_lower.contains("/dist/")
                || path_lower.contains("/node_modules/")
                || path_lower.contains("/.next/")
                || path_lower.contains("/__pycache__/");

            // Keep important files, filter trivial ones
            has_changes && !is_lock_file && !is_generated
        })
        .collect()
}

/// 计算目录优先级权重
///
/// 根据文件路径判断目录的重要性，返回优先级权重系数。
/// 核心模块优先级高，工具类优先级低。
fn calculate_directory_priority(path: &str) -> f32 {
    let path_lower = path.to_lowercase();

    // 核心领域模块 - 最高优先级 (2.0x)
    if path_lower.contains("/domain/")
        || path_lower.contains("/core/")
        || path_lower.contains("/engine/")
    {
        return 2.0;
    }

    // 服务层/业务逻辑 - 高优先级 (1.5x)
    if path_lower.contains("/services/")
        || path_lower.contains("/service/")
        || path_lower.contains("/api/")
        || path_lower.contains("/handlers/")
        || path_lower.contains("/controllers/")
        || path_lower.contains("/business/")
        || path_lower.contains("/logic/")
    {
        return 1.5;
    }

    // 基础设施/中间件 - 正常优先级 (1.2x)
    if path_lower.contains("/infrastructure/")
        || path_lower.contains("/middleware/")
        || path_lower.contains("/storage/")
        || path_lower.contains("/client/")
    {
        return 1.2;
    }

    // 工具类/帮助函数 - 低优先级 (0.7x)
    if path_lower.contains("/utils/")
        || path_lower.contains("/util/")
        || path_lower.contains("/helpers/")
        || path_lower.contains("/helper/")
        || path_lower.contains("/common/")
        || path_lower.contains("/shared/")
    {
        return 0.7;
    }

    // 测试文件 - 较低优先级 (0.8x)
    if path_lower.contains("/tests/")
        || path_lower.contains("/test/")
        || path_lower.contains("_test.")
        || path_lower.ends_with("_test.rs")
        || path_lower.ends_with(".test.js")
        || path_lower.ends_with(".test.ts")
    {
        return 0.8;
    }

    // UI/样式/静态资源 - 较低优先级 (0.6x)
    if path_lower.contains("/ui/")
        || path_lower.contains("/styles/")
        || path_lower.contains("/assets/")
        || path_lower.contains("/static/")
        || path_lower.contains("/public/")
    {
        return 0.6;
    }

    // 默认优先级 (1.0x)
    1.0
}
