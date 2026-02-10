//! 提交总结服务接口
//!
//! 提供基于 LLM 的三阶段提交分析功能，自动生成结构化的 commit 总结。
//!
//! # 三阶段分析流程
//!
//! 1. **阶段一：文件分类** - 将变更文件分类为 logic（核心逻辑）、test（测试）、config（配置）、build（构建）等
//! 2. **阶段二：分类分析** - 针对每个分类进行专门的详细分析
//! 3. **阶段三：全局总结** - 整合所有分析结果，生成最终的 commit 总结
//!
//! # 设计理念
//!
//! 采用分阶段分析的原因：
//! - 不同类型的变更需要不同的分析策略（逻辑变更 vs 测试变更 vs 配置变更）
//! - 分阶段可以提供更精准的上下文给 LLM，提升分析质量
//! - 便于并行处理和缓存中间结果

use crate::{CommitSummaryAnalysis, ServiceError};

/// 提交总结服务接口
///
/// 提供基于 LLM 的智能提交分析功能，自动生成结构化的 commit 总结。
///
/// # 功能特性
///
/// - 三阶段渐进式分析（文件分类 → 分类分析 → 全局总结）
/// - 自动推断目标分支（如未指定）
/// - 支持多种文件类型的专门分析策略
/// - 生成符合规范的 commit message 和详细的变更说明
///
/// # 线程安全
///
/// 实现须满足 [`Send`] + [`Sync`]，以便在多线程或异步上下文中共享。
///
/// # 示例
///
/// ```ignore
/// use domain::CommitSummaryService;
///
/// fn example(service: &dyn CommitSummaryService) -> Result<(), Box<dyn std::error::Error>> {
///     // 使用默认基准分支（自动推断）
///     let analysis = service.run_analysis(None)?;
///     println!("Summary: {}", analysis.summary);
///     println!("Type: {}", analysis.commit_type);
///
///     // 指定基准分支
///     let analysis = service.run_analysis(Some("main"))?;
///     println!("Changes: {:?}", analysis.changes);
///
///     Ok(())
/// }
/// ```
pub trait CommitSummaryService: Send + Sync {
    /// 执行三阶段提交分析
    ///
    /// 对当前分支相对基准分支的所有变更执行完整的三阶段分析，返回结构化的总结结果。
    ///
    /// # 参数
    ///
    /// * `base_branch` - 基准分支名称。如果为 `None`，将自动推断目标分支
    ///   （通常是 `main`、`master` 或远程默认分支）。
    ///
    /// # 返回
    ///
    /// 返回 [`CommitSummaryAnalysis`]，包含：
    /// - `summary` - commit message 的简短总结（一行）
    /// - `commit_type` - 变更类型（feat、fix、refactor、docs 等）
    /// - `changes` - 详细的变更列表，按文件分类
    /// - `breaking_changes` - 是否包含破坏性变更
    /// - `scope` - 变更影响的范围
    ///
    /// # 错误
    ///
    /// * [`ServiceError::Git`] - Git 操作失败（无法获取 diff、分支不存在等）
    /// * [`ServiceError::Other`] - LLM API 调用失败、解析失败、网络错误等
    ///
    /// # 分析流程
    ///
    /// 1. **获取变更** - 获取当前分支相对基准分支的所有文件变更
    /// 2. **阶段一：文件分类** - 调用 LLM 将变更文件分类为：
    ///    - `logic` - 核心业务逻辑
    ///    - `test` - 测试代码
    ///    - `config` - 配置文件
    ///    - `build` - 构建脚本
    ///    - `docs` - 文档
    /// 3. **阶段二：分类分析** - 针对每个分类调用专门的分析策略：
    ///    - 逻辑变更：关注功能变化、算法改进、bug 修复
    ///    - 测试变更：关注测试覆盖率、测试用例
    ///    - 配置变更：关注配置项变化、环境差异
    /// 4. **阶段三：全局总结** - 整合所有分析结果，生成最终的 commit 总结
    ///
    /// # 性能考虑
    ///
    /// - 本方法会进行多次 LLM API 调用（3-5 次），总耗时约 5-15 秒
    /// - 建议在异步上下文中调用以避免阻塞
    /// - 考虑实现增量分析或缓存机制以提升性能
    ///
    /// # 典型用例
    ///
    /// - 自动生成 commit message
    /// - 生成 PR 描述
    /// - 代码审查辅助（快速了解变更内容）
    /// - 生成 changelog
    fn run_analysis(
        &self,
        base_branch: Option<&str>,
    ) -> Result<CommitSummaryAnalysis, ServiceError>;
}
