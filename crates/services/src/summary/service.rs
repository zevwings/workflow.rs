//! 提交总结服务实现
//!
//! 编排三阶段提交分析：阶段一文件分类 → 阶段二分类分析 → 阶段三全局总结。

use std::{collections::HashMap, sync::Arc};

use client::{LLMClient, LLMConfigContext, LanguageManager};
use domain::{
    CommitChangeType, CommitFileChange, CommitFileClassification, CommitInfo,
    CommitSummaryAnalysis, CommitSummaryError, CommitSummaryService, DirectoryStats,
    DirectoryStatusDistribution, GitRepository,
};

use crate::summary::{
    BatchAnalyzeService, ConfigAnalyzeService, FileClassifyService, LogicAnalyzeService,
    SummaryAnalyzeInput, SummaryAnalyzeService, TestAnalyzeService,
};

/// 文件状态统计
#[derive(Debug, Clone, Copy)]
struct FileStatusCount {
    /// 新增文件数
    added: u32,
    /// 删除文件数
    deleted: u32,
    /// 修改文件数
    modified: u32,
    /// 重命名文件数
    renamed: u32,
}

/// 准备阶段产出的分析上下文
///
/// 汇总 Git 数据和统计信息，供三阶段子服务使用。
struct AnalysisContext {
    /// HEAD commit 信息（用于阶段一分类）
    commit_info: CommitInfo,
    /// 变更文件列表
    files: Vec<CommitFileChange>,
    /// 按文件路径索引的 diff 内容
    file_diffs: HashMap<String, String>,
    /// LLM 输出语言代码
    language_code: String,
    /// 文件状态统计
    status_count: FileStatusCount,
    /// 总新增行数
    total_additions: u32,
    /// 总删除行数
    total_deletions: u32,
    /// 是否存在未提交的变更
    has_uncommitted_changes: bool,
    /// 分支提交历史（从旧到新）
    commit_history: Vec<CommitInfo>,
    /// 提交总数
    commit_count: u32,
    /// 目录聚合统计
    directory_stats: Vec<DirectoryStats>,
    /// 纯格式化变更的文件列表
    formatting_only_files: Vec<String>,
}

/// 提交总结服务实现
pub(crate) struct CommitSummaryServiceImpl {
    git_repo: Arc<dyn GitRepository>,
    llm_client: Arc<dyn LLMClient>,
    llm_context: Arc<dyn LLMConfigContext>,
    llm_language_manager: Arc<dyn LanguageManager>,
}

impl CommitSummaryServiceImpl {
    pub fn new(
        git_repo: Arc<dyn GitRepository>,
        llm_client: Arc<dyn LLMClient>,
        llm_context: Arc<dyn LLMConfigContext>,
        llm_language_manager: Arc<dyn LanguageManager>,
    ) -> Self {
        Self {
            git_repo,
            llm_client,
            llm_context,
            llm_language_manager,
        }
    }

    /// 准备阶段：从 Git 仓库收集分析所需的全部数据
    ///
    /// 包括分支推断、文件变更列表、diff 解析和统计计算。
    fn prepare(&self, base_branch: Option<&str>) -> Result<AnalysisContext, CommitSummaryError> {
        // 1. 确定当前分支和基准分支
        let current_branch = self.git_repo.get_current_branch()?;

        let mut base_branch = match base_branch {
            Some(b) => b.to_string(),
            None => self
                .git_repo
                .infer_target_branch(&current_branch)?
                .unwrap_or_else(|| "master".to_string()),
        };

        // 若推断出的基准分支与当前分支相同，则使用默认分支，避免 "nothing to analyze"
        if base_branch == current_branch {
            base_branch =
                self.git_repo.get_default_branch().unwrap_or_else(|_| "master".to_string());
        }

        // 2. 获取变更文件列表（仅已提交：merge_base..current）
        let mut files = self.git_repo.get_merge_changed_files(&current_branch, &base_branch)?;

        let (full_diff, commit_shas) = if files.is_empty() {
            // 无已提交变更时，尝试使用暂存区变更进行分析
            let staged_files = self.git_repo.get_staged_files()?;
            if !staged_files.is_empty() {
                files = staged_files;
                let diff = self.git_repo.get_staged_diff()?.unwrap_or_default();
                (diff, Vec::new())
            } else {
                return Err(CommitSummaryError::NoChangesToAnalyze);
            }
        } else {
            let diff =
                self.git_repo.get_merge_diff(&current_branch, &base_branch)?.unwrap_or_default();
            let shas = self.git_repo.commits_to_merge(&current_branch, &base_branch)?;
            (diff, shas)
        };

        // 3. 获取 HEAD commit 元数据（阶段一分类需要 commit_id / author / timestamp）
        let commit_info = self.git_repo.get_commit_info("HEAD")?;

        let commit_count = commit_shas.len() as u32;
        const MAX_HISTORY: usize = 50;
        let commit_history: Vec<CommitInfo> = commit_shas
            .iter()
            .take(MAX_HISTORY)
            .filter_map(|sha| self.git_repo.get_commit_info(sha).ok())
            .collect();

        // 4. 按文件拆分的 diff
        let file_diffs = parse_diff_per_file(&full_diff);

        // 5. 检查工作区状态
        let has_uncommitted_changes = self
            .git_repo
            .get_working_tree_status()
            .map(|status| !status.is_clean())
            .unwrap_or(false);

        // 6. 统计信息
        let status_count = count_by_status(&files);
        let total_additions: u32 = files.iter().filter_map(|f| f.additions).sum();
        let total_deletions: u32 = files.iter().filter_map(|f| f.deletions).sum();

        // 6.5. 聚合目录统计
        let directory_stats = aggregate_by_directory(&files);

        // 6.6. 检测纯格式化文件
        let formatting_only_files =
            detect_formatting_files(&self.git_repo, &base_branch, &current_branch, &files)?;

        // 7. 语言代码
        let language_code = self.llm_context.get_language();

        Ok(AnalysisContext {
            commit_info,
            files,
            file_diffs,
            language_code,
            status_count,
            total_additions,
            total_deletions,
            has_uncommitted_changes,
            commit_history,
            commit_count,
            directory_stats,
            formatting_only_files,
        })
    }
}

impl CommitSummaryServiceImpl {
    /// 阶段一：文件分类
    ///
    /// 调用 LLM 对变更文件进行智能分类，产出 `CommitFileClassification`。
    fn run_stage1(
        &self,
        ctx: &AnalysisContext,
    ) -> Result<CommitFileClassification, CommitSummaryError> {
        // Pre-filter files for extremely large commits to reduce token usage
        use crate::summary::prefilter_files_for_large_commits;
        let filtered_files = prefilter_files_for_large_commits(ctx.files.clone());

        if filtered_files.len() < ctx.files.len() {
            eprintln!(
                "Pre-filtered {} files down to {} for stage 1 classification",
                ctx.files.len(),
                filtered_files.len()
            );
        }

        let classify_service = FileClassifyService::new(self.llm_client.clone());
        classify_service.classify(
            &ctx.commit_info.sha,
            &ctx.commit_info.author_email,
            ctx.commit_info.author_time,
            &filtered_files,
            &ctx.directory_stats,
        )
    }

    /// 阶段二：分类分析
    ///
    /// 并行执行 4 个子服务，每个接收阶段一结果 + diff map，返回 JSON 字符串。
    ///
    /// 使用 rayon 并行执行，总耗时从 T1+T2+T3+T4 降低到 max(T1,T2,T3,T4)。
    fn run_stage2(
        &self,
        ctx: &AnalysisContext,
        stage1: &CommitFileClassification,
    ) -> Result<Stage2Results, CommitSummaryError> {
        // 使用 rayon::join 并行执行四个子服务
        // 采用嵌套 join 结构：((batch, logic), (config, test))
        let ((batch_result, logic_result), (config_result, test_result)) = rayon::join(
            || {
                rayon::join(
                    // 2.1 批量操作分析（不需要格式化检测）
                    || {
                        BatchAnalyzeService::new(self.llm_client.clone()).analyze(
                            stage1,
                            &ctx.file_diffs,
                            &ctx.files,
                        )
                    },
                    // 2.2 核心逻辑分析（排除格式化文件）
                    || {
                        LogicAnalyzeService::new(self.llm_client.clone()).analyze(
                            stage1,
                            &ctx.file_diffs,
                            &ctx.files,
                            &ctx.formatting_only_files,
                        )
                    },
                )
            },
            || {
                rayon::join(
                    // 2.3 配置/文档分析（排除格式化文件）
                    || {
                        ConfigAnalyzeService::new(self.llm_client.clone()).analyze(
                            stage1,
                            &ctx.file_diffs,
                            &ctx.files,
                            &ctx.formatting_only_files,
                        )
                    },
                    // 2.4 测试文件分析（排除格式化文件）
                    || {
                        TestAnalyzeService::new(self.llm_client.clone()).analyze(
                            stage1,
                            &ctx.file_diffs,
                            &ctx.files,
                            &ctx.formatting_only_files,
                        )
                    },
                )
            },
        );

        // 收集所有结果，任何一个失败都会中止
        Ok(Stage2Results {
            batch_json: batch_result?,
            logic_json: logic_result?,
            config_json: config_result?,
            test_json: test_result?,
        })
    }
}

/// 阶段二产出的分类分析结果
struct Stage2Results {
    /// 批量操作分析 JSON
    batch_json: String,
    /// 核心逻辑分析 JSON
    logic_json: String,
    /// 配置/文档分析 JSON
    config_json: String,
    /// 测试文件分析 JSON
    test_json: String,
}

impl CommitSummaryService for CommitSummaryServiceImpl {
    fn run_analysis(
        &self,
        base_branch: Option<&str>,
    ) -> Result<CommitSummaryAnalysis, CommitSummaryError> {
        // 准备阶段：收集 Git 数据和统计信息
        let ctx = self.prepare(base_branch)?;

        // 阶段一：文件分类
        let stage1 = self.run_stage1(&ctx)?;

        // 阶段二：分类分析（batch / logic / config / tests）
        let stage2 = self.run_stage2(&ctx, &stage1)?;

        // 阶段三：全局总结
        let stage1_json = serde_json::to_string(&stage1)
            .map_err(|e| CommitSummaryError::SerializeFailed(e.to_string()))?;

        // 格式化提交历史摘要
        let commit_history_summary = if ctx.commit_history.is_empty() {
            "No commit history available".to_string()
        } else {
            ctx.commit_history
                .iter()
                .map(|c| {
                    format!(
                        "{} - {}",
                        &c.sha[..8],
                        c.message.lines().next().unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let input = SummaryAnalyzeInput {
            stage1_classification: stage1_json,
            stage2_batch_analysis: stage2.batch_json,
            stage2_logic_analysis: stage2.logic_json,
            stage2_config_analysis: stage2.config_json,
            stage2_test_analysis: stage2.test_json,
            total_files: ctx.files.len() as u32,
            added_count: ctx.status_count.added,
            deleted_count: ctx.status_count.deleted,
            modified_count: ctx.status_count.modified,
            renamed_count: ctx.status_count.renamed,
            total_additions: ctx.total_additions,
            total_deletions: ctx.total_deletions,
            has_uncommitted_changes: ctx.has_uncommitted_changes,
            commit_history_summary,
            commit_count: ctx.commit_count,
        };

        let language = self
            .llm_language_manager
            .find_language(&ctx.language_code)
            .unwrap_or_else(|| self.llm_language_manager.get_default_language());
        SummaryAnalyzeService::new(self.llm_client.clone(), language.clone()).summarize(input)
    }
}

fn parse_diff_per_file(full_diff: &str) -> HashMap<String, String> {
    if full_diff.trim().is_empty() {
        return HashMap::new();
    }

    let normalized = format!("\n{}", full_diff.trim_start());
    // 估算文件数量并预分配容量
    let estimated_files = normalized.matches("\ndiff --git ").count();
    let mut map = HashMap::with_capacity(estimated_files);
    let segments: Vec<&str> = normalized.split("\ndiff --git ").collect();
    for seg in segments {
        if seg.is_empty() {
            continue;
        }
        let first_line_end = seg.find('\n').unwrap_or(seg.len());
        let first_line = seg[..first_line_end].trim();
        let path = first_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.strip_prefix("b/"))
            .map(String::from)
            .unwrap_or_default();
        if path.is_empty() {
            continue;
        }
        let diff_content = seg[first_line_end..].trim();
        if !diff_content.is_empty() {
            let full_block = format!("diff --git a/{} b/{}\n{}", path, path, diff_content);
            map.insert(path, full_block);
        }
    }
    map
}

/// 统计文件状态
///
/// 按照变更类型统计文件数量。
fn count_by_status(files: &[CommitFileChange]) -> FileStatusCount {
    let mut count = FileStatusCount {
        added: 0,
        deleted: 0,
        modified: 0,
        renamed: 0,
    };

    for f in files {
        match f.change_type {
            CommitChangeType::Added => count.added += 1,
            CommitChangeType::Deleted => count.deleted += 1,
            CommitChangeType::Modified
            | CommitChangeType::Copied
            | CommitChangeType::TypeChanged => count.modified += 1,
            CommitChangeType::Renamed => count.renamed += 1,
        }
    }

    count
}

// ========== 目录聚合统计 ==========

/// 按目录聚合文件变更统计
///
/// 提取前3级目录路径作为分组键，计算每个目录的变更指标。
fn aggregate_by_directory(files: &[CommitFileChange]) -> Vec<DirectoryStats> {
    use std::collections::HashMap;

    let mut dir_map: HashMap<String, DirectoryStatsBuilder> = HashMap::new();

    for file in files {
        let dir_path = extract_top_level_directory(&file.path, 3);
        let builder = dir_map.entry(dir_path).or_default();
        builder.add_file(file);
    }

    let mut stats: Vec<DirectoryStats> =
        dir_map.into_iter().map(|(path, builder)| builder.build(path)).collect();

    // 按变更量降序排序（additions + deletions）
    stats.sort_by(|a, b| {
        let a_total = a.total_additions + a.total_deletions;
        let b_total = b.total_additions + b.total_deletions;
        b_total.cmp(&a_total)
    });

    stats
}

/// 提取目录路径的前 N 级
///
/// 例如: "src/services/summary/batch/service.rs" 提取前3级 -> "src/services/summary"
fn extract_top_level_directory(path: &str, levels: usize) -> String {
    let parts: Vec<&str> = path.split('/').collect();

    // 根目录文件
    if parts.len() == 1 {
        return ".".to_string();
    }

    // 路径层级少于指定层数
    if parts.len() <= levels {
        if parts.len() > 1 {
            return parts[..parts.len() - 1].join("/");
        }
        return ".".to_string();
    }

    parts[..levels].join("/")
}

/// DirectoryStats 构建器（内部辅助结构）
#[derive(Debug, Default)]
struct DirectoryStatsBuilder {
    file_count: u32,
    total_additions: u32,
    total_deletions: u32,
    added_count: u32,
    deleted_count: u32,
    modified_count: u32,
    renamed_count: u32,
}

impl DirectoryStatsBuilder {
    fn add_file(&mut self, file: &CommitFileChange) {
        self.file_count += 1;
        self.total_additions += file.additions.unwrap_or(0);
        self.total_deletions += file.deletions.unwrap_or(0);

        match file.change_type {
            CommitChangeType::Added => self.added_count += 1,
            CommitChangeType::Deleted => self.deleted_count += 1,
            CommitChangeType::Modified
            | CommitChangeType::Copied
            | CommitChangeType::TypeChanged => self.modified_count += 1,
            CommitChangeType::Renamed => self.renamed_count += 1,
        }
    }

    fn build(self, path: String) -> DirectoryStats {
        let all_new = self.added_count == self.file_count && self.file_count > 0;
        let all_deleted = self.deleted_count == self.file_count && self.file_count > 0;

        DirectoryStats {
            path,
            file_count: self.file_count,
            total_additions: self.total_additions,
            total_deletions: self.total_deletions,
            all_new,
            all_deleted,
            status_distribution: DirectoryStatusDistribution {
                added: self.added_count,
                deleted: self.deleted_count,
                modified: self.modified_count,
                renamed: self.renamed_count,
            },
        }
    }
}

// ========== 格式化检测 ==========

/// 检测纯格式化文件列表
///
/// 遍历所有变更文件，调用 GitRepository 检测哪些是纯格式化变更。
fn detect_formatting_files(
    git_repo: &Arc<dyn GitRepository>,
    base_branch: &str,
    target_branch: &str,
    files: &[CommitFileChange],
) -> Result<Vec<String>, CommitSummaryError> {
    let mut formatting_files = Vec::new();

    for file in files {
        // 只检测修改的文件（新增/删除文件不可能是纯格式化）
        if !matches!(file.change_type, CommitChangeType::Modified) {
            continue;
        }

        match git_repo.is_formatting_only_change(base_branch, target_branch, &file.path) {
            Ok(true) => formatting_files.push(file.path.clone()),
            Ok(false) => {} // 有实质性变更，跳过
            Err(e) => {
                // 记录错误但不中断流程
                eprintln!(
                    "Warning: Failed to check formatting for {}: {}",
                    file.path, e
                );
            }
        }
    }

    Ok(formatting_files)
}
