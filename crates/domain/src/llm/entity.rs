//! LLM 实体类型
//!
//! 包含 LLM 服务返回的实体类型定义

use serde::Deserialize;

/// PR Reword 结果，包含标题和描述
///
/// 由 LLM 基于当前 PR 标题和 PR diff 生成的标题和完整描述，用于更新现有 PR。
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestReword {
    /// PR 标题（简洁，不超过 8 个单词，主要基于当前标题）
    pub pr_title: String,
    /// PR 描述（基于 PR diff 生成的完整描述列表，包含所有重要变更，可选）
    pub description: Option<String>,
}

/// PR 总结结果，包含总结文档和文件名
///
/// 由 LLM 生成的 PR 总结文档和对应的文件名。
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestSummary {
    /// PR 总结文档（Markdown 格式）
    pub summary: String,
    /// 文件名（不含路径和扩展名）
    pub filename: String,
}
