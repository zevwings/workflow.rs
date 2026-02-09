//! Commit Message 生成服务实现
//!
//! 为单次提交场景提供轻量级的 commit message 生成功能。

use std::sync::Arc;

use domain::{
    CommitChangeType, CommitFileChange, CommitMessageService, CommitSummaryAnalysis, GitRepository,
    ServiceError,
};
use llm::{parsers::JsonParser, LLMConfigContext, LLMExecutor};

use crate::commit::message::CommitMessageConversation;

/// 分析输入数据
struct AnalysisInput {
    files: Vec<CommitFileChange>,
    diff: String,
    stats: FileStatistics,
}

/// 文件统计信息
#[derive(Debug, Clone)]
pub struct FileStatistics {
    pub total_files: u32,
    pub added_count: u32,
    pub modified_count: u32,
    pub deleted_count: u32,
    pub renamed_count: u32,
    pub total_additions: u32,
    pub total_deletions: u32,
}

/// 最大 diff 行数（避免超出 LLM 上下文窗口）
const MAX_DIFF_LINES: usize = 2000;

/// Commit Message 生成服务实现
pub struct CommitMessageServiceImpl {
    git_repo: Arc<dyn GitRepository>,
    llm_executor: Arc<dyn LLMExecutor>,
    llm_context: Arc<dyn LLMConfigContext>,
}

impl CommitMessageServiceImpl {
    /// 创建新的服务实例
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

    /// 准备 staged 变更的分析上下文
    fn prepare_staged(&self) -> Result<AnalysisInput, ServiceError> {
        // 1. 获取 staged 文件列表
        let files = self
            .git_repo
            .get_staged_files()
            .map_err(|e| ServiceError::Other(format!("Failed to get staged files: {}", e)))?;

        if files.is_empty() {
            return Err(ServiceError::Other(
                "No staged changes to commit".to_string(),
            ));
        }

        // 2. 获取 staged diff
        let diff = self
            .git_repo
            .get_staged_diff()
            .map_err(|e| ServiceError::Other(format!("Failed to get staged diff: {}", e)))?
            .unwrap_or_default();

        // 3. 统计信息
        let stats = calculate_statistics(&files);

        Ok(AnalysisInput { files, diff, stats })
    }

    /// 准备指定提交的分析上下文
    fn prepare_commit(&self, commit_ref: &str) -> Result<AnalysisInput, ServiceError> {
        // 1. 获取提交的变更文件
        let files = self
            .git_repo
            .get_commit_changed_files(commit_ref)
            .map_err(|e| ServiceError::Other(format!("Failed to get changed files: {}", e)))?;

        if files.is_empty() {
            return Err(ServiceError::Other(format!(
                "No changes found in commit {}",
                commit_ref
            )));
        }

        // 2. 获取提交的 diff
        let diff = self
            .git_repo
            .get_commit_diff(commit_ref)
            .map_err(|e| ServiceError::Other(format!("Failed to get commit diff: {}", e)))?
            .unwrap_or_default();

        // 3. 统计信息
        let stats = calculate_statistics(&files);

        Ok(AnalysisInput { files, diff, stats })
    }

    /// 生成 commit message（核心逻辑）
    fn generate(&self, input: AnalysisInput) -> Result<CommitSummaryAnalysis, ServiceError> {
        // 1. 格式化输入数据
        let file_summary = format_file_summary(&input.files);
        let diff_content = smart_truncate_diff(&input.diff, &input.files, MAX_DIFF_LINES);

        // 2. 构建 LLM 对话
        let language_code = self.llm_context.get_language();
        let conversation =
            CommitMessageConversation::new(file_summary, diff_content, input.stats.clone());

        // 3. 单次 LLM 调用
        let response = self
            .llm_executor
            .execute(&conversation, &language_code, "commit_message_generate")
            .map_err(|e| ServiceError::Other(e.to_string()))?;

        // 4. 解析结果
        JsonParser::to_model(&response)
            .map_err(|e| ServiceError::Other(format!("Failed to parse commit message: {}", e)))
    }
}

impl CommitMessageService for CommitMessageServiceImpl {
    fn generate_for_staged(&self) -> Result<CommitSummaryAnalysis, ServiceError> {
        let input = self.prepare_staged()?;
        self.generate(input)
    }

    fn generate_for_commit(&self, commit_ref: &str) -> Result<CommitSummaryAnalysis, ServiceError> {
        let input = self.prepare_commit(commit_ref)?;
        self.generate(input)
    }
}

// ────────────────────────────────────────────────────────────────
// Helper Functions
// ────────────────────────────────────────────────────────────────

/// 计算文件统计信息
fn calculate_statistics(files: &[CommitFileChange]) -> FileStatistics {
    let mut stats = FileStatistics {
        total_files: files.len() as u32,
        added_count: 0,
        modified_count: 0,
        deleted_count: 0,
        renamed_count: 0,
        total_additions: 0,
        total_deletions: 0,
    };

    for file in files {
        match file.change_type {
            CommitChangeType::Added => stats.added_count += 1,
            CommitChangeType::Deleted => stats.deleted_count += 1,
            CommitChangeType::Modified
            | CommitChangeType::TypeChanged
            | CommitChangeType::Copied => stats.modified_count += 1,
            CommitChangeType::Renamed => stats.renamed_count += 1,
        }

        stats.total_additions += file.additions.unwrap_or(0);
        stats.total_deletions += file.deletions.unwrap_or(0);
    }

    stats
}

/// 格式化文件列表摘要
fn format_file_summary(files: &[CommitFileChange]) -> String {
    let mut lines = Vec::new();
    lines.push("## Changed Files\n".to_string());

    for file in files {
        let status = match file.change_type {
            CommitChangeType::Added => "A",
            CommitChangeType::Deleted => "D",
            CommitChangeType::Modified => "M",
            CommitChangeType::Renamed => "R",
            CommitChangeType::Copied => "C",
            CommitChangeType::TypeChanged => "T",
        };

        let additions = file.additions.unwrap_or(0);
        let deletions = file.deletions.unwrap_or(0);
        let file_type = infer_file_type(&file.path);

        lines.push(format!(
            "- [{}] {} (+{} -{}) {}",
            status, file.path, additions, deletions, file_type
        ));
    }

    lines.join("\n")
}

/// 推断文件类型
fn infer_file_type(path: &str) -> &'static str {
    if path.ends_with("_test.rs")
        || path.ends_with(".test.ts")
        || path.ends_with(".test.js")
        || path.contains("/tests/")
        || path.contains("/__tests__/")
    {
        "[test]"
    } else if path.ends_with(".md") || path.contains("/docs/") {
        "[docs]"
    } else if path.ends_with(".toml")
        || path.ends_with(".json")
        || path.ends_with(".yaml")
        || path.ends_with(".yml")
        || path.ends_with(".config.ts")
        || path.ends_with(".config.js")
    {
        "[config]"
    } else if path.ends_with(".rs")
        || path.ends_with(".ts")
        || path.ends_with(".js")
        || path.ends_with(".go")
        || path.ends_with(".py")
        || path.ends_with(".java")
    {
        "[code]"
    } else {
        ""
    }
}

/// 智能截断 diff（保留关键部分）
fn smart_truncate_diff(diff: &str, _files: &[CommitFileChange], max_lines: usize) -> String {
    let lines: Vec<&str> = diff.lines().collect();

    // 如果 diff 不大，直接返回
    if lines.len() <= max_lines {
        return diff.to_string();
    }

    // 简化实现：直接截断前 N 行
    // TODO: 实现更智能的截断逻辑
    // 1. 按文件拆分 diff
    // 2. 小文件（< 100 行）保留完整 diff
    // 3. 大文件只保留前后各 50 行 + 函数签名行

    format!(
        "{}\n\n[... Diff truncated: {} lines omitted for brevity ...]",
        lines[..max_lines].join("\n"),
        lines.len() - max_lines
    )
}
