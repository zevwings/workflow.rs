//! Pull Request LLM 服务
//!
//! 本模块提供了使用 LLM 生成 Pull Request 内容的功能。
//! 根据 commit 标题自动生成符合规范的分支名和 PR 标题。

use anyhow::{Context, Result};
use serde_json::Value;

use crate::pr::helpers::transform_to_branch_name;
use crate::settings::Settings;

use super::client::{openai, deepseek, proxy};

/// PR 内容，包含分支名和 PR 标题
///
/// 由 LLM 生成的分支名和 PR 标题，用于创建 Pull Request。
#[derive(Debug, Clone)]
pub struct PullRequestContent {
    /// 分支名称（小写，使用连字符分隔）
    pub branch_name: String,
    /// PR 标题（简洁，不超过 8 个单词）
    pub pr_title: String,
}

/// Pull Request LLM 服务
///
/// 提供使用 LLM 生成分支名和 PR 标题的功能。
/// 支持多种 LLM 提供商：OpenAI、DeepSeek、代理 API。
pub struct PullRequestLLM;

impl PullRequestLLM {
    /// 同时生成分支名和 PR 标题（通过一个 LLM 请求）
    ///
    /// 根据 commit 标题生成符合规范的分支名和 PR 标题。
    /// 分支名和 PR 标题都会自动翻译为英文（如果输入包含非英文内容）。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含：
    /// - `branch_name` - 分支名称（小写，使用连字符分隔）
    /// - `pr_title` - PR 标题（简洁，不超过 8 个单词）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn generate(commit_title: &str) -> Result<PullRequestContent> {
        let settings = Settings::load();
        let provider = settings.llm_provider.clone();

        Self::check_api_key(&settings, &provider)?;

        match provider.as_str() {
            "openai" => Self::generate_with_openai(commit_title),
            "deepseek" => Self::generate_with_deepseek(commit_title),
            "proxy" => Self::generate_with_proxy(commit_title),
            _ => Self::generate_with_openai(commit_title), // 默认使用 OpenAI
        }
    }

    /// 检查对应的 API key 是否设置
    ///
    /// 根据 LLM 提供商检查相应的 API key 是否已配置。
    ///
    /// # 参数
    ///
    /// * `settings` - 设置对象
    /// * `provider` - LLM 提供商名称（"openai"、"deepseek"、"proxy"）
    ///
    /// # 错误
    ///
    /// 如果对应的 API key 未设置，返回相应的错误信息。
    fn check_api_key(settings: &Settings, provider: &str) -> Result<()> {
        let api_key_set = match provider {
            "openai" => settings.openai_key.is_some(),
            "deepseek" => settings.deepseek_key.is_some(),
            "proxy" => settings.llm_proxy_key.is_some() && settings.llm_proxy_url.is_some(),
            _ => settings.openai_key.is_some(), // 默认检查 OpenAI
        };

        if !api_key_set {
            let error_msg = match provider {
                "openai" => "LLM_OPENAI_KEY environment variable not set",
                "deepseek" => "LLM_DEEPSEEK_KEY environment variable not set",
                "proxy" => match (settings.llm_proxy_key.is_none(), settings.llm_proxy_url.is_none()) {
                    (true, true) => "LLM_PROXY_KEY and LLM_PROXY_URL environment variables not set",
                    (true, false) => "LLM_PROXY_KEY environment variable not set",
                    (false, true) => "LLM_PROXY_URL environment variable not set",
                    (false, false) => unreachable!(),
                },
                _ => "LLM_OPENAI_KEY environment variable not set",
            };
            anyhow::bail!("{} (provider: {})", error_msg, provider);
        }

        Ok(())
    }

    /// 生成同时生成分支名和 PR 标题的 system prompt
    fn system_prompt() -> String {
        "You're a git assistant that generates both a branch name and a PR title based on the commit title. IMPORTANT: Both outputs MUST be in English only. If the commit title contains non-English text (like Chinese), translate it to English first.

For the branch name:
- Must be all lowercase
- Use hyphens to separate words
- Be under 50 characters
- Follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only)
- Examples:
  * \"Fix login bug\" → \"fix-login-bug\"
  * \"修复登录问题\" → \"fix-login-issue\"
  * \"Add user authentication\" → \"feature-add-user-authentication\"
  * \"新功能：用户认证\" → \"feature-user-authentication\"
  * \"Refactor code structure\" → \"refactoring-code-structure\"
  * \"重构代码结构\" → \"refactoring-code-structure\"
  * \"Update documentation\" → \"update-documentation\"
  * \"更新文档\" → \"update-documentation\"
  * \"Improve performance\" → \"improve-performance\"
  * \"优化性能\" → \"performance-optimization\"

For the PR title:
- Must be concise, within 8 words
- No punctuation
- In English only

Return your response in JSON format with two fields: \"branch_name\" and \"pr_title\". Example:
{
  \"branch_name\": \"feature-add-user-authentication\",
  \"pr_title\": \"Add user authentication\"
}".to_string()
    }

    /// 生成同时生成分支名和 PR 标题的 user prompt
    fn user_prompt(commit_title: &str) -> String {
        format!("Generate a branch name and PR title for this commit title: {}", commit_title)
    }

    /// 使用 OpenAI API 同时生成分支名和 PR 标题
    ///
    /// 调用 OpenAI API 生成分支名和 PR 标题。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体。
    ///
    /// # 错误
    ///
    /// 如果 API 调用失败或响应格式不正确，返回相应的错误信息。
    fn generate_with_openai(commit_title: &str) -> Result<PullRequestContent> {
        let params = openai::LLMRequestParams {
            system_prompt: Self::system_prompt(),
            user_prompt: Self::user_prompt(commit_title),
            max_tokens: 100,
            temperature: 0.5,
            model: "gpt-3.5-turbo".to_string(),
        };
        let response = openai::call_llm(params)?;
        Self::parse_llm_response(response)
    }

    /// 使用 DeepSeek API 同时生成分支名和 PR 标题
    ///
    /// 调用 DeepSeek API 生成分支名和 PR 标题。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体。
    ///
    /// # 错误
    ///
    /// 如果 API 调用失败或响应格式不正确，返回相应的错误信息。
    fn generate_with_deepseek(commit_title: &str) -> Result<PullRequestContent> {
        let params = deepseek::LLMRequestParams {
            system_prompt: Self::system_prompt(),
            user_prompt: Self::user_prompt(commit_title),
            max_tokens: 100,
            temperature: 0.5,
            model: "deepseek-chat".to_string(),
        };
        let response = deepseek::call_llm(params)?;
        Self::parse_llm_response(response)
    }

    /// 使用代理 API 同时生成分支名和 PR 标题
    ///
    /// 调用代理 API 生成分支名和 PR 标题。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体。
    ///
    /// # 错误
    ///
    /// 如果 API 调用失败或响应格式不正确，返回相应的错误信息。
    fn generate_with_proxy(commit_title: &str) -> Result<PullRequestContent> {
        let params = proxy::LLMRequestParams {
            system_prompt: Self::system_prompt(),
            user_prompt: Self::user_prompt(commit_title),
            max_tokens: 100,
            temperature: 0.5,
            model: "gpt-3.5-turbo".to_string(),
        };
        let response = proxy::call_llm(params)?;
        Self::parse_llm_response(response)
    }

    /// 解析 LLM 返回的 JSON 响应，提取分支名和 PR 标题
    ///
    /// 从 LLM 的 JSON 响应中提取 `branch_name` 和 `pr_title` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含清理后的分支名和 PR 标题。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_llm_response(response: String) -> Result<PullRequestContent> {
        let trimmed = response.trim();

        // 尝试提取 JSON（可能包含 markdown 代码块）
        let json_str = if trimmed.starts_with("```json") {
            // 移除 ```json 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else if trimmed.starts_with("```") {
            // 移除 ``` 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else {
            trimmed
        };

        // 解析 JSON
        let json: Value = serde_json::from_str(json_str)
            .context("Failed to parse LLM response as JSON")?;

        let branch_name = json
            .get("branch_name")
            .and_then(|v| v.as_str())
            .context("Missing 'branch_name' field in LLM response")?
            .to_string();

        let pr_title = json
            .get("pr_title")
            .and_then(|v| v.as_str())
            .context("Missing 'pr_title' field in LLM response")?
            .to_string();

        // 清理分支名，确保只保留 ASCII 字符
        let cleaned_branch_name = transform_to_branch_name(branch_name.trim());

        Ok(PullRequestContent {
            branch_name: cleaned_branch_name,
            pr_title: pr_title.trim().to_string(),
        })
    }
}

