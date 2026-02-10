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

/// 阶段三全局总结的输入参数
///
/// 综合阶段一分类与阶段二各分析结果及统计信息，供 LLM 生成 commit 总结。
#[derive(Debug, Clone)]
pub struct SummarizeCommitInput<'a> {
    /// 阶段一文件分类 JSON
    pub stage1_json: &'a str,
    /// 阶段二批量分析 JSON
    pub stage2_batch_json: &'a str,
    /// 阶段二逻辑分析 JSON
    pub stage2_logic_json: &'a str,
    /// 阶段二配置分析 JSON
    pub stage2_config_json: &'a str,
    /// 阶段二测试分析 JSON
    pub stage2_test_json: &'a str,
    /// 总文件数
    pub total_files: u32,
    /// 新增文件数
    pub added_count: u32,
    /// 删除文件数
    pub deleted_count: u32,
    /// 修改文件数
    pub modified_count: u32,
    /// 重命名文件数
    pub renamed_count: u32,
    /// 总新增行数
    pub total_additions: u32,
    /// 总删除行数
    pub total_deletions: u32,
}
