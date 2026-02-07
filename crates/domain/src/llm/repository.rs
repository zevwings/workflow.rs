//! LLM 仓储接口
//!
//! 定义与大语言模型（LLM）交互的底层接口，支持各种 AI 辅助功能。

use crate::llm::error::LLMError;
use crate::summary::entity::{
    CommitBatchAnalysis, CommitConfigAnalysis, CommitFileClassification, CommitLogicAnalysis,
    CommitSummaryAnalysis, CommitTestAnalysis,
};

/// LLM 仓储接口
///
/// 提供与大语言模型 API 交互的底层接口，封装了各类智能分析功能。
///
/// # 功能范围
///
/// ## 代码生成
/// - 分支名生成
/// - Commit message 生成
///
/// ## 代码分析（三阶段 Commit 分析）
/// - **阶段一**：文件分类（logic、test、config、build）
/// - **阶段二**：分类分析（批量操作、核心逻辑、配置、测试）
/// - **阶段三**：全局总结（整合所有分析结果）
///
/// ## 配置管理
/// - LLM 配置验证
///
/// # 线程安全
///
/// 实现须满足 [`Send`] + [`Sync`]，以便在多线程或异步上下文中共享。
///
/// # 支持的 LLM 提供商
///
/// 实现可以支持多种 LLM 提供商，例如：
/// - OpenAI（GPT-3.5、GPT-4）
/// - Anthropic（Claude）
/// - 本地模型（Ollama、LLaMA）
///
/// 具体实现由 Storage 层负责。
///
/// # 错误处理
///
/// 所有方法返回 [`LLMError`]，包含：
/// - API 调用失败（网络错误、超时、限流）
/// - 响应解析失败（JSON 格式错误）
/// - 配置错误（API Key 无效）
///
/// # 性能考虑
///
/// - LLM API 调用通常需要 1-5 秒
/// - 三阶段分析会进行多次 API 调用（3-5 次）
/// - 建议在异步上下文中调用
/// - 考虑实现缓存和并行调用以优化性能
pub trait LLMRepository: Send + Sync {
    /// 验证 LLM 配置
    fn verify_config(&self) -> Result<String, LLMError>;

    /// 生成分支名
    fn generate_branch_name(
        &self,
        title: Option<&str>,
        exists_branches: Option<Vec<String>>,
    ) -> Result<String, LLMError>;

    /// 阶段一：对提交的文件变更列表进行智能分类
    ///
    /// 输入为设计文档中的 JSON（commit 元数据 + files 数组），返回分类结果。
    fn classify_commit_files(&self, input_json: &str)
        -> Result<CommitFileClassification, LLMError>;

    /// 阶段二 2.1：批量操作分析
    ///
    /// 当阶段一检测到批量重命名、格式化、配置更新等模式时使用。传入操作类型、文件数、模式描述及样本 diff。
    fn analyze_commit_batch(&self, user_prompt: &str) -> Result<CommitBatchAnalysis, LLMError>;

    /// 阶段二 2.2：核心逻辑分析
    ///
    /// 对业务代码、服务层等核心文件的完整 diff 进行深入分析。
    fn analyze_commit_logic(&self, user_prompt: &str) -> Result<CommitLogicAnalysis, LLMError>;

    /// 阶段二 2.3：配置/文档分析
    ///
    /// 对配置文件、环境变量、文档类文件的修改进行简要总结。
    fn analyze_commit_config(&self, user_prompt: &str) -> Result<CommitConfigAnalysis, LLMError>;

    /// 阶段二 2.4：测试文件分析
    ///
    /// 分析测试文件的变更及与业务代码的对应关系。
    fn analyze_commit_tests(&self, user_prompt: &str) -> Result<CommitTestAnalysis, LLMError>;

    /// 阶段三：全局总结
    ///
    /// 综合阶段一分类结果与阶段二各分析结果及统计信息，生成结构化的 commit 总结（标题、描述、影响分析等）。
    fn summarize_commit_analysis(
        &self,
        stage1_json: &str,
        stage2_batch_json: &str,
        stage2_logic_json: &str,
        stage2_config_json: &str,
        stage2_test_json: &str,
        total_files: u32,
        added_count: u32,
        deleted_count: u32,
        modified_count: u32,
        renamed_count: u32,
        total_additions: u32,
        total_deletions: u32,
    ) -> Result<CommitSummaryAnalysis, LLMError>;

}
