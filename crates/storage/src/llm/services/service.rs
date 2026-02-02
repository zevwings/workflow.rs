//! LLM 服务
//!
//! 提供统一的 LLM API 调用逻辑，封装请求构建、调用和基础错误处理。

use std::sync::Arc;

use domain::{
    LLMConfigContext, LLMError, PrContent, PullRequestContent, PullRequestReword,
    PullRequestSummary,
};

use toolkit::log_debug;

use crate::llm::client::{LLMClient, LLMRequestParameters};
use crate::llm::services::conversations::VerifyConversation;
use crate::llm::services::{
    conversations::{
        BranchNameConversation, CommitMessageConversation, CreateConversation,
        FileSummaryConversation, PrContentConversation, RewordConversation, SummaryConversation,
        TranslateConversation,
    },
    LLMConversation,
};

/// LLM 服务 Trait
///
/// 定义 LLM 服务的通用接口，提供各种 LLM 功能。
pub trait LLMService: Send + Sync {
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
    fn translate_to_english(&self, text: &str) -> Result<String, LLMError>;

    /// 创建 PR 内容（包含分支名、PR 标题、描述、scope 和详细总结）
    fn create_pr_content(
        &self,
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent, LLMError>;

    /// 重写 PR 标题和描述
    fn reword_pr(
        &self,
        pr_diff: &str,
        current_title: Option<&str>,
    ) -> Result<PullRequestReword, LLMError>;

    /// 总结 PR
    fn summarize_pr(&self, pr_title: &str, pr_diff: &str) -> Result<PullRequestSummary, LLMError>;

    /// 总结文件变更
    fn summarize_file_change(&self, file_path: &str, file_diff: &str) -> Result<String, LLMError>;
}

/// LLM 服务实现
///
/// 负责统一的 LLM API 调用，包括请求构建、调用和基础错误处理。
pub struct LLMServiceImpl {
    client: Arc<dyn LLMClient>,
    context: Arc<dyn LLMConfigContext>,
}

impl LLMServiceImpl {
    /// 创建新的 LLM 服务实例
    ///
    /// # 参数
    ///
    /// * `client` - LLM 客户端实例
    /// * `context` - LLM 配置上下文
    ///
    /// # 返回
    ///
    /// 返回 `LLMService` 实例
    pub fn new(client: Arc<dyn LLMClient>, context: Arc<dyn LLMConfigContext>) -> Self {
        Self { client, context }
    }

    /// 执行 LLM 调用并解析响应
    ///
    /// 从 conversation 获取 prompt 和参数，调用 LLM API，然后解析响应。
    ///
    /// # 参数
    ///
    /// * `conversation` - 实现了 `LLMConversation` trait 的对话实例
    /// * `language_code` - 语言代码（如 "en", "zh"）
    /// * `context` - 上下文信息，用于错误提示
    ///
    /// # 返回
    ///
    /// 返回解析后的结果
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应解析失败，返回相应的错误信息。
    fn execute<C: LLMConversation>(
        &self,
        conversation: C,
        language_code: &str,
        context: &str,
    ) -> Result<C::Output, LLMError> {
        // 从 conversation 获取 prompt 和参数
        let system_prompt = conversation.get_system_prompt(language_code);
        let user_prompt = conversation.get_user_prompt(language_code);
        let (max_tokens, temperature) = conversation.get_execution_params();

        // 调用 LLM API
        let params = LLMRequestParameters {
            system_prompt,
            user_prompt,
            max_tokens,
            temperature,
        };

        let response = self.client.call(&params).map_err(|e| {
            // 提取原始错误消息，避免重复的 "LLM API 调用失败: " 前缀
            let original_msg = match &e {
                LLMError::ApiError(msg) => msg
                    .strip_prefix("LLM API 调用失败: ")
                    .unwrap_or(msg)
                    .to_string(),
                _ => e.to_string(),
            };
            LLMError::ApiError(format!(
                "Failed to call LLM API ({}): {}",
                context, original_msg
            ))
        })?;

        // 解析响应
        conversation.parse_response(response).map_err(|e| {
            // 提取原始错误消息，避免重复的 "LLM API 调用失败: " 前缀
            let original_msg = match &e {
                LLMError::ApiError(msg) => msg
                    .strip_prefix("LLM API 调用失败: ")
                    .unwrap_or(msg)
                    .to_string(),
                _ => e.to_string(),
            };
            LLMError::ApiError(format!(
                "Failed to parse LLM response ({}): {}",
                context, original_msg
            ))
        })
    }
}

impl LLMService for LLMServiceImpl {
    fn verify_config(&self) -> Result<String, LLMError> {
        // execute 已经返回 LLMError，不需要再次包装
        let result = self.execute(
            VerifyConversation::new(),
            self.context.get_language().as_str(),
            "verifying LLM configuration",
        )?;

        if result.is_empty() {
            return Err(LLMError::AuthenticationFailed);
        }

        Ok(result)
    }

    fn generate_branch_name(
        &self,
        title: Option<&str>,
        exists_branches: Option<Vec<String>>,
    ) -> Result<String, LLMError> {
        // 验证至少有一个参数
        if title.is_none() {
            return Err(LLMError::GenerationFailed(
                "title must be provided".to_string(),
            ));
        }

        let input = (title.map(|s| s.to_string()), exists_branches);

        let conversation = BranchNameConversation::new(input);
        self.execute(
            conversation,
            self.context.get_language().as_str(),
            &format!(
                "generating branch name from title: '{}'",
                title.unwrap_or_default()
            ),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }

    fn generate_pr_content(
        &self,
        branch_name: &str,
        commits: &[String],
    ) -> Result<PrContent, LLMError> {
        let input = (branch_name.to_string(), commits.to_vec());
        self.execute(
            PrContentConversation::new(input),
            self.context.get_language().as_str(),
            &format!("generating PR content from branch: '{}'", branch_name),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }

    fn generate_commit_message(&self, changes: &str) -> Result<String, LLMError> {
        self.execute(
            CommitMessageConversation::new(changes.to_string()),
            self.context.get_language().as_str(),
            &format!("generating commit message from changes: '{}'", changes),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }

    fn translate_to_english(&self, text: &str) -> Result<String, LLMError> {
        self.execute(
            TranslateConversation::new(text.to_string()),
            self.context.get_language().as_str(),
            &format!("translating text to English: '{}'", text),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }

    fn create_pr_content(
        &self,
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent, LLMError> {
        // 第一步：生成基本信息（分支名、PR 标题、描述、scope）
        let input = (commit_title.to_string(), exists_branches.clone(), git_diff.clone());
        let mut content = self
            .execute(
                CreateConversation::new(input),
                self.context.get_language().as_str(),
                &format!(
                    "generating PR content from commit title: '{}'",
                    commit_title
                ),
            )
            .map_err(|e| LLMError::ApiError(e.to_string()))?;

        // 第二步：如果有 git diff，生成详细总结
        if let Some(ref diff) = git_diff {
            if !diff.trim().is_empty() {
                let summary_result = self.execute(
                    SummaryConversation::new((content.pr_title.clone(), diff.clone())),
                    self.context.get_language().as_str(),
                    &format!(
                        "generating detailed PR summary for: '{}'",
                        content.pr_title
                    ),
                );

                // 如果生成总结成功，添加到 content 中；失败则只记录错误但不影响主流程
                match summary_result {
                    Ok(summary) => {
                        content.summary = Some(summary.summary);
                    }
                    Err(e) => {
                        // 记录错误但不中断流程，summary 保持为 None
                        log_debug!(
                            "Failed to generate detailed summary: {}, continuing without summary",
                            e
                        );
                    }
                }
            }
        }

        Ok(content)
    }

    fn reword_pr(
        &self,
        pr_diff: &str,
        current_title: Option<&str>,
    ) -> Result<PullRequestReword, LLMError> {
        let input = (pr_diff.to_string(), current_title.map(|s| s.to_string()));
        self.execute(
            RewordConversation::new(input),
            self.context.get_language().as_str(),
            &format!(
                "rewording PR from diff (current title: '{}')",
                current_title.unwrap_or_default()
            ),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }

    fn summarize_pr(&self, pr_title: &str, pr_diff: &str) -> Result<PullRequestSummary, LLMError> {
        let input = (pr_title.to_string(), pr_diff.to_string());
        self.execute(
            SummaryConversation::new(input),
            self.context.get_language().as_str(),
            &format!("summarizing PR: '{}'", pr_title),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }

    fn summarize_file_change(&self, file_path: &str, file_diff: &str) -> Result<String, LLMError> {
        let input = (file_path.to_string(), file_diff.to_string());
        self.execute(
            FileSummaryConversation::new(input),
            self.context.get_language().as_str(),
            &format!("summarizing file change: '{}'", file_path),
        )
        .map_err(|e| LLMError::ApiError(e.to_string()))
    }
}
