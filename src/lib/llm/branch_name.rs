use anyhow::Result;

use crate::pr::helpers::transform_to_branch_name;
use crate::settings::Settings;

use super::client::{openai, deepseek, proxy};

/// 生成分支名生成的 system prompt
fn branch_name_system_prompt() -> String {
    "You're a git branch naming assistant. Generate a concise, descriptive git branch name based on the commit title. IMPORTANT: The branch name MUST be in English only. If the commit title contains non-English text (like Chinese), translate it to English first. The branch name should be all lowercase, use hyphens to separate words, be under 50 characters, and follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only). Only return the branch name, nothing else.".to_string()
}

/// 生成分支名生成的 user prompt
fn branch_name_user_prompt(commit_title: &str) -> String {
    format!("Generate an English-only git branch name for this commit title: {}", commit_title)
}

/// 使用 LLM 根据 commit_title 生成分支名
pub fn generate_branch_name_with_llm(commit_title: &str) -> Result<String> {
    let settings = Settings::load();
    let provider = settings.llm_provider.clone();

    // 先检查对应的 API key 是否设置
    let api_key_set = match provider.as_str() {
        "openai" => settings.openai_key.is_some(),
        "deepseek" => settings.deepseek_key.is_some(),
        "proxy" => settings.llm_proxy_key.is_some() && settings.llm_proxy_url.is_some(),
        _ => settings.openai_key.is_some(), // 默认检查 OpenAI
    };

    if !api_key_set {
        let error_msg = match provider.as_str() {
            "openai" => "LLM_OPENAI_KEY environment variable not set",
            "deepseek" => "LLM_DEEPSEEK_KEY environment variable not set",
            "proxy" => {
                if settings.llm_proxy_key.is_none() && settings.llm_proxy_url.is_none() {
                    "LLM_PROXY_KEY and LLM_PROXY_URL environment variables not set"
                } else if settings.llm_proxy_key.is_none() {
                    "LLM_PROXY_KEY environment variable not set"
                } else {
                    "LLM_PROXY_URL environment variable not set"
                }
            }
            _ => "LLM_OPENAI_KEY environment variable not set",
        };
        anyhow::bail!("{} (provider: {})", error_msg, provider);
    }

    match provider.as_str() {
        "openai" => generate_branch_name_with_openai(commit_title),
        "deepseek" => generate_branch_name_with_deepseek(commit_title),
        "proxy" => generate_branch_name_with_proxy(commit_title),
        _ => generate_branch_name_with_openai(commit_title), // 默认使用 OpenAI
    }
}

/// 使用 OpenAI API 生成分支名
fn generate_branch_name_with_openai(commit_title: &str) -> Result<String> {
    let params = openai::LLMRequestParams {
        system_prompt: branch_name_system_prompt(),
        user_prompt: branch_name_user_prompt(commit_title),
        max_tokens: 50,
        temperature: 0.3,
        model: "gpt-3.5-turbo".to_string(),
    };
    let branch_name = openai::call_llm(params)?;
    // 清理分支名，确保只保留 ASCII 字符
    let cleaned_branch_name = transform_to_branch_name(branch_name.trim());
    Ok(cleaned_branch_name)
}

/// 使用 DeepSeek API 生成分支名
fn generate_branch_name_with_deepseek(commit_title: &str) -> Result<String> {
    let params = deepseek::LLMRequestParams {
        system_prompt: branch_name_system_prompt(),
        user_prompt: branch_name_user_prompt(commit_title),
        max_tokens: 50,
        temperature: 0.3,
        model: "deepseek-chat".to_string(),
    };
    let branch_name = deepseek::call_llm(params)?;
    // 清理分支名，确保只保留 ASCII 字符
    let cleaned_branch_name = transform_to_branch_name(branch_name.trim());
    Ok(cleaned_branch_name)
}

/// 使用代理 API 生成分支名
fn generate_branch_name_with_proxy(commit_title: &str) -> Result<String> {
    let params = proxy::LLMRequestParams {
        system_prompt: branch_name_system_prompt(),
        user_prompt: branch_name_user_prompt(commit_title),
        max_tokens: 50,
        temperature: 0.3,
        model: "gpt-3.5-turbo".to_string(),
    };
    let branch_name = proxy::call_llm(params)?;
    // 清理分支名，确保只保留 ASCII 字符
    let cleaned_branch_name = transform_to_branch_name(branch_name.trim());
    Ok(cleaned_branch_name)
}
