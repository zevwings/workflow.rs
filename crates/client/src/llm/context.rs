//! LLM 配置提供者接口
//!
//! 定义 LLM 配置的抽象接口，实现依赖倒置原则。
//! LLM 模块定义此接口，由其他模块（如 infra）实现。

use crate::http::Authorization;
use serde_with::skip_serializing_none;

use crate::LLMError;

// ==================== LLM 配置数据模型 ====================

/// LLM 配置
#[skip_serializing_none]
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub url: String,
    pub auth: Authorization,
    pub model: String,
    pub language: String,
}

/// 可转换为 LLM 请求参数的 trait
///
/// 用于快速将任意类型转换为 `LLMRequestParameters`。
/// 对 `LLMConversation` 有默认实现，也可为其他类型单独实现。
pub trait IntoLLMConfig {
    fn to_config(&self) -> Result<LLMConfig, LLMError>;
}

impl<T: ?Sized + LLMConfigContext> IntoLLMConfig for T {
    fn to_config(&self) -> Result<LLMConfig, LLMError> {
        let provider = self.get_provider();

        let url = match provider.as_str() {
            "openai" => Ok("https://api.openai.com/v1/chat/completions".to_string()),
            "deepseek" => Ok("https://api.deepseek.com/chat/completions".to_string()),
            "proxy" => {
                let base_url = self.get_current_provider_url();
                if base_url.is_empty() {
                    return Err(LLMError::ApiError("URL is empty in settings".to_string()));
                }
                Ok(format!(
                    "{}/chat/completions",
                    base_url.trim_end_matches('/')
                ))
            }
            _ => Err(LLMError::ApiError(format!(
                "Unsupported LLM provider: {}",
                provider
            ))),
        }?;

        let llm_key = self.get_current_provider_key();
        if llm_key.is_empty() {
            return Err(LLMError::ApiError(
                "LLM key is empty in settings".to_string(),
            ));
        }
        let auth = Authorization::bearer(llm_key);

        let model = self.get_current_provider_model();
        if model.is_empty() {
            return Err(LLMError::ApiError(format!(
                "Model is required for {} provider",
                provider
            )));
        }

        Ok(LLMConfig {
            provider,
            url,
            auth,
            model,
            language: self.get_language(),
        })
    }
}

/// LLM 配置提供者 trait
///
/// 提供 LLM 相关的配置信息，包括 provider、URL、API key、model 和语言设置。
/// 通过此 trait，LLM 模块可以独立于具体的配置实现（如 Settings）。
///
/// # 实现者
///
/// 此 trait 应该由基础设施层（如 `infra::adapters`）实现，将不同的配置源
/// （如 Settings、环境变量等）适配为统一的接口。
///
/// # 线程安全
///
/// 此 trait 要求实现 `Send + Sync`，以便在多线程环境中安全使用。
pub trait LLMConfigContext: Send + Sync {
    /// 获取当前 LLM Provider 名称
    ///
    /// # 返回
    ///
    /// 返回 provider 名称（如 "openai", "deepseek", "proxy"）。
    fn get_provider(&self) -> String;

    /// 获取当前 Provider 的 URL
    ///
    /// # 返回
    ///
    /// 返回当前 provider 的 URL（仅 proxy provider 需要），如果未配置则返回默认值。
    fn get_current_provider_url(&self) -> String;

    /// 获取当前 Provider 的 API Key
    ///
    /// # 返回
    ///
    /// 返回当前 provider 的 API key，如果未配置则返回默认值。
    fn get_current_provider_key(&self) -> String;

    /// 获取当前 Provider 的模型名称
    ///
    /// # 返回
    ///
    /// 返回当前 provider 的模型名称，如果未配置则返回默认值。
    fn get_current_provider_model(&self) -> String;

    /// 获取 LLM 输出语言
    ///
    /// # 返回
    ///
    /// 返回语言代码（如 "en", "zh-CN"），如果未配置则返回默认值 "en"。
    fn get_language(&self) -> String;
}
