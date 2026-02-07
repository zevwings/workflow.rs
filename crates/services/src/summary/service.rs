//! 提交总结服务实现
//!
//! 编排三阶段提交分析：阶段一文件分类 → 阶段二分类分析 → 阶段三全局总结。

use std::collections::HashMap;
use std::sync::Arc;

use domain::git::entity::{CommitChangeType, CommitFileChange, CommitInfo};
use domain::summary::entity::{CommitFileClassification, CommitSummaryAnalysis};
use domain::{errors::ServiceError, GitRepository};
use llm::{LLMConfigContext, LLMExecutor};

use super::{
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
}

/// 提交总结服务实现
pub struct CommitSummaryServiceImpl {
    git_repo: Arc<dyn GitRepository>,
    llm_executor: Arc<dyn LLMExecutor>,
    llm_context: Arc<dyn LLMConfigContext>,
}

impl CommitSummaryServiceImpl {
    pub fn new(
        git_repo: Arc<dyn GitRepository>,
        llm_executor: Arc<dyn LLMExecutor>,
        llm_context: Arc<dyn LLMConfigContext>,
    ) -> Self {
        Self {
            git_repo,
            llm_executor,
            llm_context,
        }
    }

    /// 准备阶段：从 Git 仓库收集分析所需的全部数据
    ///
    /// 包括分支推断、文件变更列表、diff 解析和统计计算。
    fn prepare(&self, base_branch: Option<&str>) -> Result<AnalysisContext, ServiceError> {
        // 1. 确定当前分支和基准分支
        let current_branch = self
            .git_repo
            .get_current_branch()
            .map_err(|e| ServiceError::Other(format!("获取当前分支失败: {}", e)))?;

        let base_branch = match base_branch {
            Some(b) => b.to_string(),
            None => self
                .git_repo
                .infer_target_branch(&current_branch)
                .map_err(|e| ServiceError::Other(format!("推断基准分支失败: {}", e)))?
                .unwrap_or_else(|| "master".to_string()),
        };

        // 2. 获取变更文件列表
        let files = self
            .git_repo
            .get_merge_changed_files(&current_branch, &base_branch)
            .map_err(|e| ServiceError::Other(format!("获取变更文件列表失败: {}", e)))?;

        if files.is_empty() {
            return Err(ServiceError::Other(format!(
                "当前分支 {} 相对 {} 无变更文件，无需分析。",
                current_branch, base_branch
            )));
        }

        // 3. 获取 HEAD commit 元数据（阶段一分类需要 commit_id / author / timestamp）
        let commit_info = self
            .git_repo
            .get_commit_info("HEAD")
            .map_err(|e| ServiceError::Other(format!("获取 commit 信息失败: {}", e)))?;

        // 4. 获取完整 merge diff 并按文件拆分
        let full_diff = self
            .git_repo
            .get_merge_diff(&current_branch, &base_branch)
            .map_err(|e| ServiceError::Other(format!("获取 merge diff 失败: {}", e)))?
            .unwrap_or_default();
        let file_diffs = parse_diff_per_file(&full_diff);

        // 5. 统计信息
        let status_count = count_by_status(&files);
        let total_additions: u32 = files.iter().filter_map(|f| f.additions).sum();
        let total_deletions: u32 = files.iter().filter_map(|f| f.deletions).sum();

        // 6. 语言代码
        let language_code = self.llm_context.get_language();

        Ok(AnalysisContext {
            commit_info,
            files,
            file_diffs,
            language_code,
            status_count,
            total_additions,
            total_deletions,
        })
    }
}

impl CommitSummaryServiceImpl {
    /// 阶段一：文件分类
    ///
    /// 调用 LLM 对变更文件进行智能分类，产出 `CommitFileClassification`。
    fn run_stage1(&self, ctx: &AnalysisContext) -> Result<CommitFileClassification, ServiceError> {
        let classify_service = FileClassifyService::new(self.llm_executor.clone());
        classify_service.classify(
            &ctx.commit_info.sha,
            &ctx.commit_info.author_email,
            ctx.commit_info.author_time,
            &ctx.files,
            &ctx.language_code,
        )
    }

    /// 阶段二：分类分析
    ///
    /// 顺序执行 4 个子服务，每个接收阶段一结果 + diff map，返回 JSON 字符串。
    fn run_stage2(
        &self,
        ctx: &AnalysisContext,
        stage1: &CommitFileClassification,
    ) -> Result<Stage2Results, ServiceError> {
        // 2.1 批量操作分析
        let batch_json = BatchAnalyzeService::new(self.llm_executor.clone()).analyze(
            stage1,
            &ctx.file_diffs,
            &ctx.files,
            &ctx.language_code,
        )?;

        // 2.2 核心逻辑分析
        let logic_json = LogicAnalyzeService::new(self.llm_executor.clone()).analyze(
            stage1,
            &ctx.file_diffs,
            &ctx.files,
            &ctx.language_code,
        )?;

        // 2.3 配置/文档分析
        let config_json = ConfigAnalyzeService::new(self.llm_executor.clone()).analyze(
            stage1,
            &ctx.file_diffs,
            &ctx.files,
            &ctx.language_code,
        )?;

        // 2.4 测试文件分析
        let test_json = TestAnalyzeService::new(self.llm_executor.clone()).analyze(
            stage1,
            &ctx.file_diffs,
            &ctx.language_code,
        )?;

        Ok(Stage2Results {
            batch_json,
            logic_json,
            config_json,
            test_json,
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

impl domain::CommitSummaryService for CommitSummaryServiceImpl {
    fn run_analysis(
        &self,
        base_branch: Option<&str>,
    ) -> Result<CommitSummaryAnalysis, ServiceError> {
        // 准备阶段：收集 Git 数据和统计信息
        let ctx = self.prepare(base_branch)?;

        // 阶段一：文件分类
        let stage1 = self.run_stage1(&ctx)?;

        // 阶段二：分类分析（batch / logic / config / tests）
        let stage2 = self.run_stage2(&ctx, &stage1)?;

        // 阶段三：全局总结
        let stage1_json = serde_json::to_string(&stage1)
            .map_err(|e| ServiceError::Other(format!("序列化阶段一结果失败: {}", e)))?;

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
        };

        SummaryAnalyzeService::new(self.llm_executor.clone()).summarize(input, &ctx.language_code)
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
