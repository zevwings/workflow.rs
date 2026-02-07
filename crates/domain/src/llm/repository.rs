//! LLM 服务接口

use crate::llm::entity::{
    CommitBatchAnalysis, CommitConfigAnalysis, CommitFileClassification, CommitLogicAnalysis,
    CommitSummaryAnalysis, CommitTestAnalysis, PullRequestContent,
};
use crate::llm::error::LLMError;

/// LLM 服务接口
///
/// 提供 LLM API 操作的接口定义。
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

    /// 创建 PR 内容（包含分支名、PR 标题、描述、scope 和详细总结）
    ///
    /// 根据 commit 标题和 git diff 生成符合规范的分支名、PR 标题、描述、scope 和详细总结。
    /// 分支名和 PR 标题都会自动翻译为英文（如果输入包含非英文内容）。
    ///
    /// 如果提供了 `git_diff`，还会自动生成详细的 PR 总结文档（Markdown 格式），
    /// 包含需求分析、技术细节、变更列表等完整信息。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    /// * `exists_branches` - 已存在的分支列表（可选）
    /// * `git_diff` - Git 工作区和暂存区的修改内容（可选，用于生成描述、提取 scope 和生成详细总结）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含分支名、PR 标题、描述、scope 和详细总结（如果有 git_diff）
    fn create_pr_content(
        &self,
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent, LLMError>;
}
