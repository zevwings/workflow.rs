//! LLM 配置相关结构体

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// 单个 LLM Provider 的配置
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMProviderSettings {
    /// Provider URL（仅 proxy 使用）
    pub url: Option<String>,
    /// Provider API Key
    pub key: Option<String>,
    /// 模型名称
    pub model: Option<String>,
}

impl LLMProviderSettings {
    /// 检查 Provider 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.url.is_none() && self.key.is_none() && self.model.is_none()
    }
}

/// LLM 配置（TOML）
/// 支持按 provider 分组，每个 provider 有独立的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSettings {
    /// 当前使用的 LLM Provider (openai, deepseek, proxy)
    #[serde(
        default = "LLMSettings::default_provider",
        skip_serializing_if = "LLMSettings::is_default_provider"
    )]
    pub provider: String,
    /// LLM 输出语言（en, zh, zh-CN, zh-TW 等，默认 en），用于控制 AI 生成内容（如 PR 总结等）的语言
    /// 所有 provider 共享此语言设置
    #[serde(
        default = "LLMSettings::default_language",
        skip_serializing_if = "String::is_empty"
    )]
    pub language: String,
    /// OpenAI 配置
    #[serde(default, skip_serializing_if = "LLMProviderSettings::is_empty")]
    pub openai: LLMProviderSettings,
    /// DeepSeek 配置
    #[serde(default, skip_serializing_if = "LLMProviderSettings::is_empty")]
    pub deepseek: LLMProviderSettings,
    /// Proxy 配置
    #[serde(default, skip_serializing_if = "LLMProviderSettings::is_empty")]
    pub proxy: LLMProviderSettings,
}

impl Default for LLMSettings {
    fn default() -> Self {
        Self {
            provider: Self::default_provider(),
            language: Self::default_language(),
            openai: LLMProviderSettings::default(),
            deepseek: LLMProviderSettings::default(),
            proxy: LLMProviderSettings::default(),
        }
    }
}

impl LLMSettings {
    /// 默认 LLM Provider
    pub fn default_provider() -> String {
        "openai".to_string()
    }

    /// 检查 provider 是否为默认值
    fn is_default_provider(provider: &str) -> bool {
        provider == Self::default_provider()
    }

    /// 根据 Provider 获取默认模型
    pub fn default_model(provider: impl AsRef<str>) -> String {
        match provider.as_ref() {
            "openai" => "gpt-4.0".to_string(),
            "deepseek" => "deepseek-chat".to_string(),
            _ => String::new(), // proxy 必须输入，没有默认值
        }
    }

    /// 默认 LLM 输出语言
    pub fn default_language() -> String {
        "en".to_string()
    }

    /// 获取当前 provider 的配置
    pub fn current_provider(&self) -> &LLMProviderSettings {
        match self.provider.as_str() {
            "openai" => &self.openai,
            "deepseek" => &self.deepseek,
            "proxy" => &self.proxy,
            _ => &self.openai, // 默认返回 openai
        }
    }

    /// 获取当前 provider 的配置（可变引用）
    pub fn current_provider_mut(&mut self) -> &mut LLMProviderSettings {
        match self.provider.as_str() {
            "openai" => &mut self.openai,
            "deepseek" => &mut self.deepseek,
            "proxy" => &mut self.proxy,
            _ => &mut self.openai, // 默认返回 openai
        }
    }

    /// 检查 LLM 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.openai.is_empty()
            && self.deepseek.is_empty()
            && self.proxy.is_empty()
            && self.provider == Self::default_provider()
            && self.language == Self::default_language()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_provider_settings_is_empty() {
        let settings = LLMProviderSettings::default();
        assert!(settings.is_empty());
    }

    #[test]
    fn test_llm_settings_default_values() {
        let settings = LLMSettings::default();
        assert_eq!(settings.provider, "openai");
        assert_eq!(settings.language, "en");
        assert!(settings.is_empty());
    }

    #[test]
    fn test_llm_settings_current_provider() {
        let settings = LLMSettings {
            provider: "deepseek".to_string(),
            language: LLMSettings::default_language(),
            openai: LLMProviderSettings::default(),
            deepseek: LLMProviderSettings {
                url: Some("https://api.deepseek.com".to_string()),
                key: Some("token".to_string()),
                model: Some("deepseek-chat".to_string()),
            },
            proxy: LLMProviderSettings::default(),
        };

        let current = settings.current_provider();
        assert_eq!(current.model.as_deref(), Some("deepseek-chat"));
    }

    #[test]
    fn test_llm_settings_default_model() {
        assert_eq!(LLMSettings::default_model("openai"), "gpt-4.0");
        assert_eq!(LLMSettings::default_model("deepseek"), "deepseek-chat");
        assert_eq!(LLMSettings::default_model("proxy"), "");
    }

    #[test]
    fn test_llm_settings_serialize_skip_empty_providers() {
        let settings = LLMSettings::default();
        let toml = toml::to_string(&settings).unwrap();
        assert!(!toml.contains("openai"));
        assert!(!toml.contains("deepseek"));
        assert!(!toml.contains("proxy"));
    }
}
