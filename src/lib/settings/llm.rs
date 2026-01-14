//! LLM 配置相关结构体

use crate::llm::{LLMClient, LLMRequestParams};
use crate::mask_sensitive_value;
use crate::prompt::Tabled;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// LLM 配置信息
#[derive(Debug, Clone)]
pub struct LLMConfigInfo {
    /// Provider
    pub provider: String,
    /// Model（包含 URL 信息，如果适用）
    pub model: String,
    /// Key（掩码显示）
    pub key: String,
    /// Output Language
    pub language: String,
}

/// LLM 验证状态
#[derive(Debug, Clone)]
pub enum LLMVerificationStatus {
    /// 验证成功
    Success {
        /// 测试响应内容
        test_response: String,
    },
    /// 验证失败
    Failed {
        /// 失败原因
        reason: String,
        /// 详细错误信息
        details: Vec<String>,
    },
}

/// LLM 验证结果
#[derive(Debug, Clone)]
pub struct LLMVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<LLMConfigInfo>,
    /// 验证结果
    pub verification: Option<LLMVerificationStatus>,
}

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
    #[serde(default = "LLMSettings::default_provider")]
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

    /// 根据 Provider 获取默认模型
    pub fn default_model(provider: &str) -> String {
        match provider {
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

impl LLMSettings {
    /// 获取 LLM 配置信息
    pub fn get_llm_config(&self) -> LLMConfigInfo {
        let current = self.current_provider();

        // 获取 model（如果有保存的值，否则显示默认值）
        let model = if let Some(ref model) = current.model {
            model.clone()
        } else {
            LLMSettings::default_model(&self.provider)
        };

        // 组合 model 和 URL（仅在 provider 为 "proxy" 时显示 URL）
        let model_display = if self.provider == "proxy" {
            if let Some(ref url) = current.url {
                if !url.is_empty() {
                    format!("{}({})", model, url)
                } else {
                    model
                }
            } else {
                model
            }
        } else {
            model
        };

        // 获取 Key（掩码显示）
        let key = current
            .key
            .as_ref()
            .map(|k| mask_sensitive_value(k))
            .unwrap_or_else(|| "-".to_string());

        // 获取 Language（如果有保存的值，否则显示默认值）
        let language = if !self.language.is_empty() {
            self.language.clone()
        } else {
            LLMSettings::default_language()
        };

        LLMConfigInfo {
            provider: self.provider.clone(),
            model: model_display,
            key,
            language,
        }
    }

    /// 验证 LLM 配置并返回结果
    ///
    /// 通过发送一个简单的测试请求（"Say hello"）来验证 LLM 配置的有效性。
    /// 验证包括：
    /// - 配置完整性检查
    /// - API 连接测试
    /// - API Key 有效性验证
    /// - LLM 响应验证
    pub fn verify(&self) -> Result<LLMVerificationResult> {
        // 检查配置是否完整
        let current = self.current_provider();
        let api_key = current.key.as_deref().unwrap_or_default();

        if api_key.is_empty() {
            return Ok(LLMVerificationResult {
                configured: false,
                config: None,
                verification: None,
            });
        }

        // 获取配置信息
        let config = self.get_llm_config();

        // 发送测试请求
        let verification = {
            let client = LLMClient::global();
            let params = LLMRequestParams {
                system_prompt: "You are a helpful assistant.".to_string(),
                user_prompt: "Say hello".to_string(),
                temperature: 0.7,
                ..Default::default()
            };

            match client.call(&params) {
                Ok(response) => {
                    if response.trim().is_empty() {
                        Some(LLMVerificationStatus::Failed {
                            reason: "LLM returned empty response".to_string(),
                            details: vec![
                                "The LLM API call succeeded but returned an empty response."
                                    .to_string(),
                            ],
                        })
                    } else {
                        Some(LLMVerificationStatus::Success {
                            test_response: response.trim().to_string(),
                        })
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let (reason, details) = if error_msg.contains("API key")
                        || error_msg.contains("API Key")
                    {
                        (
                            "API Key invalid or not configured".to_string(),
                            vec![
                                error_msg,
                                "Please check your LLM API key configuration.".to_string(),
                            ],
                        )
                    } else if error_msg.contains("timeout") {
                        (
                            "Connection timeout".to_string(),
                            vec![
                                error_msg,
                                "The LLM API request timed out. Please check your network connection.".to_string(),
                            ],
                        )
                    } else if error_msg.contains("network") || error_msg.contains("connection") {
                        (
                            "Network connection failed".to_string(),
                            vec![
                                error_msg,
                                "Please check your network connection and proxy settings."
                                    .to_string(),
                            ],
                        )
                    } else {
                        (
                            format!("Verification failed: {}", error_msg),
                            vec![error_msg],
                        )
                    };

                    Some(LLMVerificationStatus::Failed { reason, details })
                }
            }
        };

        Ok(LLMVerificationResult {
            configured: true,
            config: Some(config),
            verification,
        })
    }
}

/// LLM 配置表格行
///
/// 用于在表格中显示 LLM 配置信息。
pub struct LLMConfigRow {
    pub provider: String,
    pub model: String,
    pub key: String,
    pub language: String,
}

impl Tabled for LLMConfigRow {
    fn headers() -> Vec<String> {
        vec![
            "Provider".to_string(),
            "Model".to_string(),
            "Key".to_string(),
            "Output Language".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.provider.clone(),
            self.model.clone(),
            self.key.clone(),
            self.language.clone(),
        ]
    }
}
