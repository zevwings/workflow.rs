//! LLM 实体类型
//!
//! 包含 LLM 服务返回的实体类型定义

use serde::{Deserialize, Serialize};

/// PR 创建内容，包含分支名、PR 标题、描述、scope 和详细总结
///
/// 由 LLM 生成的分支名、PR 标题、描述、scope 和详细总结，用于创建 Pull Request。
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestContent {
    /// 分支名称（小写，使用连字符分隔）
    pub branch_name: String,
    /// PR 标题（简洁，不超过 8 个单词）
    pub pr_title: String,
    /// PR 描述（基于 Git 修改内容生成）
    pub description: Option<String>,
    /// Commit scope（从 git diff 提取，用于 Conventional Commits 格式）
    ///
    /// Scope 表示变更涉及的模块或功能区域，例如 "api", "auth", "jira" 等。
    /// 如果无法确定 scope，此字段为 `None`。
    pub scope: Option<String>,
    /// PR 详细总结（Markdown 格式，可选）
    ///
    /// 包含完整的 PR 总结文档，包括需求分析、技术细节、变更列表等。
    /// 只有在提供了 git diff 时才会生成此字段。
    pub summary: Option<String>,
}

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
