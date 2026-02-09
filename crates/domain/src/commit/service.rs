//! Commit 服务接口定义

use crate::errors::ServiceError;
use crate::summary::entity::CommitSummaryAnalysis;

/// Commit Message 生成服务
///
/// 为单次提交场景提供轻量级的 commit message 生成功能。
/// 与 `CommitSummaryService` 的区别：
/// - `CommitMessageService`: 单次提交分析，单次 LLM 调用，快速简单
/// - `CommitSummaryService`: 分支合并分析，三阶段流程，深度分析
pub trait CommitMessageService: Send + Sync {
    /// 为 staged 变更生成 commit message
    ///
    /// 适用于 `git commit` 前的场景，分析当前暂存区的变更。
    ///
    /// # 返回
    ///
    /// 返回结构化的 commit 分析结果，包含：
    /// - `commit_message`: 符合 Conventional Commits 规范的 message
    /// - `structured_summary`: 结构化总结（按类别和领域分组）
    /// - `impact_analysis`: 影响分析和风险评估
    ///
    /// # 错误
    ///
    /// - 如果暂存区为空，返回错误
    /// - 如果 LLM 调用失败，返回错误
    /// - 如果解析结果失败，返回错误
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let service = get_commit_message_service();
    /// let analysis = service.generate_for_staged()?;
    /// println!("Title: {}", analysis.commit_message.title);
    /// ```
    fn generate_for_staged(&self) -> Result<CommitSummaryAnalysis, ServiceError>;

    /// 为指定提交生成 commit message
    ///
    /// 适用于分析已有提交的场景，如 `workflow commit analyze <sha>`。
    ///
    /// # 参数
    ///
    /// - `commit_ref`: 提交引用（SHA、HEAD、分支名等）
    ///
    /// # 返回
    ///
    /// 返回结构化的 commit 分析结果。
    ///
    /// # 错误
    ///
    /// - 如果提交不存在，返回错误
    /// - 如果 LLM 调用失败，返回错误
    /// - 如果解析结果失败，返回错误
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let service = get_commit_message_service();
    /// let analysis = service.generate_for_commit("HEAD")?;
    /// println!("Summary: {}", analysis.structured_summary.main_purpose);
    /// ```
    fn generate_for_commit(&self, commit_ref: &str) -> Result<CommitSummaryAnalysis, ServiceError>;
}
