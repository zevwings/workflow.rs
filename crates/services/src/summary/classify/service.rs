//! 阶段一：文件分类服务
//!
//! 根据 commit 元数据和文件变更列表，调用 LLM 进行智能分类。

use std::sync::Arc;

use domain::errors::ServiceError;
use domain::git::entity::{CommitChangeType, CommitFileChange};
use domain::summary::entity::{CommitFileClassification, DirectoryStats};
use llm::parsers::JsonParser;
use llm::LLMExecutor;

use super::FileClassifyConversation;

/// 阶段一：文件分类服务
pub(crate) struct FileClassifyService {
    llm_executor: Arc<dyn LLMExecutor>,
}

impl FileClassifyService {
    pub fn new(llm_executor: Arc<dyn LLMExecutor>) -> Self {
        Self { llm_executor }
    }

    /// 对文件变更列表执行 LLM 分类
    pub fn classify(
        &self,
        commit_id: &str,
        author: &str,
        timestamp: i64,
        files: &[CommitFileChange],
        directory_stats: &[DirectoryStats],
        language_code: &str,
    ) -> Result<CommitFileClassification, ServiceError> {
        let input_json = build_input_json(commit_id, author, timestamp, files, directory_stats);
        let conversation = FileClassifyConversation::new(input_json);
        let response = self
            .llm_executor
            .execute(&conversation, language_code, "file_classify")
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        JsonParser::to_model(&response).map_err(|e| {
            ServiceError::Other(format!(
                "Failed to parse file classification results: {}",
                e
            ))
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn build_input_json(
    commit_id: &str,
    author: &str,
    timestamp: i64,
    files: &[CommitFileChange],
    directory_stats: &[DirectoryStats],
) -> String {
    use serde_json::json;
    let files_json: Vec<_> = files
        .iter()
        .map(|f| {
            json!({
                "path": f.path,
                "status": status_to_str(f.change_type),
                "additions": f.additions.unwrap_or(0),
                "deletions": f.deletions.unwrap_or(0),
                "old_path": f.old_path
            })
        })
        .collect();

    // 序列化目录统计（取前10个最重要的目录）
    let directory_json: Vec<_> = directory_stats
        .iter()
        .take(10)
        .map(|d| {
            json!({
                "path": d.path,
                "file_count": d.file_count,
                "total_additions": d.total_additions,
                "total_deletions": d.total_deletions,
                "all_new": d.all_new,
                "all_deleted": d.all_deleted,
                "status_distribution": {
                    "added": d.status_distribution.added,
                    "deleted": d.status_distribution.deleted,
                    "modified": d.status_distribution.modified,
                    "renamed": d.status_distribution.renamed
                }
            })
        })
        .collect();

    json!({
        "commit_id": commit_id,
        "author": author,
        "timestamp": timestamp,
        "files": files_json,
        "directory_stats": directory_json
    })
    .to_string()
}

fn status_to_str(t: CommitChangeType) -> &'static str {
    match t {
        CommitChangeType::Added => "added",
        CommitChangeType::Modified => "modified",
        CommitChangeType::Deleted => "deleted",
        CommitChangeType::Renamed => "renamed",
        CommitChangeType::Copied => "copied",
        CommitChangeType::TypeChanged => "type_changed",
    }
}
