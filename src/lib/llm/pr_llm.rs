use anyhow::{Context, Result};
use serde_json::Value;

use crate::pr::helpers::transform_to_branch_name;
use crate::settings::Settings;

use super::client::{openai, deepseek, proxy};

/// PR 内容，包含分支名和 PR 标题
#[derive(Debug, Clone)]
pub struct PullRequestContent {
    pub branch_name: String,
    pub pr_title: String,
}

/// Pull Request LLM 服务
pub struct PullRequestLLM;

impl PullRequestLLM {
    /// 同时生成分支名和 PR 标题（通过一个 LLM 请求）
    ///
    /// # 参数
    /// * `commit_title` - commit 标题或描述
    ///
    /// # 返回
    /// 返回 `PullRequestContent` 结构体，包含 `branch_name` 和 `pr_title`
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
- MUST start with an appropriate prefix based on the commit title content:
  * If the commit title contains keywords related to bug fixes, issues, or problems (e.g., \"fix\", \"bug\", \"修复\", \"问题\", \"bug fix\", \"hotfix\"), use prefix \"fix-\"
  * If the commit title contains keywords related to new features or enhancements (e.g., \"feature\", \"new\", \"新功能\", \"功能\", \"add\", \"implement\"), use prefix \"feature-\"
  * If the commit title contains keywords related to refactoring (e.g., \"refactor\", \"重构\", \"refactoring\", \"restructure\"), use prefix \"refactoring-\"
  * If none of the above apply, use prefix \"feature-\" as default
- Examples:
  * \"Fix login bug\" → \"fix-login-bug\"
  * \"修复登录问题\" → \"fix-login-issue\"
  * \"Add user authentication\" → \"feature-add-user-authentication\"
  * \"新功能：用户认证\" → \"feature-user-authentication\"
  * \"Refactor code structure\" → \"refactoring-code-structure\"
  * \"重构代码结构\" → \"refactoring-code-structure\"

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

