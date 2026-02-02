//! LLM 服务接口

use crate::llm::entity::{PullRequestContent, PullRequestReword, PullRequestSummary};
use crate::llm::error::LLMError;
use crate::pr::entity::PrContent;

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

    /// 生成 PR 内容
    fn generate_pr_content(
        &self,
        branch_name: &str,
        commits: &[String],
    ) -> Result<PrContent, LLMError>;

    /// 生成提交信息
    fn generate_commit_message(&self, changes: &str) -> Result<String, LLMError>;

    /// 翻译文本为英文
    ///
    /// 使用 LLM 将非英文文本（中文、俄文等）翻译为英文。
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    ///
    /// # 返回
    ///
    /// 返回翻译后的英文文本
    fn translate_to_english(&self, text: &str) -> Result<String, LLMError>;

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

    /// 重写 PR 标题和描述
    ///
    /// 根据当前 PR 标题和 PR diff 生成更新的 PR 标题和描述，用于更新现有 PR。
    ///
    /// # 参数
    ///
    /// * `pr_diff` - PR 的 diff 内容
    /// * `current_title` - 当前 PR 标题（可选）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestReword` 结构体，包含更新的 PR 标题和描述
    fn reword_pr(
        &self,
        pr_diff: &str,
        current_title: Option<&str>,
    ) -> Result<PullRequestReword, LLMError>;

    /// 生成 PR 总结文档
    ///
    /// 根据 PR 的 diff 内容生成详细的总结文档和文件名。
    ///
    /// # 参数
    ///
    /// * `pr_title` - PR 标题
    /// * `pr_diff` - PR 的 diff 内容
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestSummary` 结构体，包含总结文档（Markdown 格式）和文件名
    fn summarize_pr(&self, pr_title: &str, pr_diff: &str) -> Result<PullRequestSummary, LLMError>;

    /// 生成单个文件的修改总结
    ///
    /// 根据文件的 diff 内容生成该文件的修改总结。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    /// * `file_diff` - 文件的 diff 内容
    ///
    /// # 返回
    ///
    /// 返回文件的修改总结（纯文本）
    fn summarize_file_change(&self, file_path: &str, file_diff: &str) -> Result<String, LLMError>;
}
