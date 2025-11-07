use anyhow::Result;

use crate::log_info;
use crate::settings::Settings;

/// 生成翻译的 system prompt
fn translate_system_prompt() -> String {
    "You're a multilingual assistant skilled in translating content into concise English github pull request titles, within 8 words, and without any punctuation.".to_string()
}

/// 生成翻译的 user prompt
fn translate_user_prompt(desc: &str) -> String {
    desc.to_string()
}

/// 判断是否需要翻译
/// 规则：如果包含非英文或描述太长，需要翻译
pub fn should_translate(text: &str) -> bool {
    // 检查是否包含非英文字符
    let has_non_english = text.chars().any(|c| {
        let code = c as u32;
        // 检查是否在 ASCII 可打印字符范围之外（除了允许的标点）
        !(0x20..=0x7E).contains(&code)
    });

    // 检查是否太长（超过 100 字符）
    let is_too_long = text.len() > 100;

    has_non_english || is_too_long
}

/// 使用 LLM 翻译描述为简洁的英文 PR 标题
pub fn translate_with_llm(desc: &str) -> Result<String> {
    let settings = Settings::load();
    let provider = settings.llm_provider.clone();

    log_info!("Using LLM provider: {}", provider);

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
        "openai" => translate_with_openai(desc),
        "deepseek" => translate_with_deepseek(desc),
        "proxy" => translate_with_proxy(desc),
        _ => translate_with_openai(desc), // 默认使用 OpenAI
    }
}

/// 使用 OpenAI API 翻译
fn translate_with_openai(desc: &str) -> Result<String> {
    use super::client::openai;
    let params = openai::LLMRequestParams {
        system_prompt: translate_system_prompt(),
        user_prompt: translate_user_prompt(desc),
        max_tokens: 60,
        temperature: 0.7,
        model: "gpt-3.5-turbo".to_string(),
    };
    openai::call_llm(params)
}

/// 使用 DeepSeek API 翻译
fn translate_with_deepseek(desc: &str) -> Result<String> {
    use super::client::deepseek;
    let params = deepseek::LLMRequestParams {
        system_prompt: translate_system_prompt(),
        user_prompt: translate_user_prompt(desc),
        max_tokens: 60,
        temperature: 0.7,
        model: "deepseek-chat".to_string(),
    };
    deepseek::call_llm(params)
}

/// 使用代理 API 翻译
fn translate_with_proxy(desc: &str) -> Result<String> {
    use super::client::proxy;
    let params = proxy::LLMRequestParams {
        system_prompt: translate_system_prompt(),
        user_prompt: translate_user_prompt(desc),
        max_tokens: 60,
        temperature: 0.7,
        model: "gpt-3.5-turbo".to_string(),
    };
    proxy::call_llm(params)
}
