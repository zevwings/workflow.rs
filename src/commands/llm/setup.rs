//! LLM 配置设置命令
//! 交互式设置 LLM 相关配置（provider, url, key, model, language）

use crate::base::dialog::{InputDialog, SelectDialog};
use crate::base::settings::defaults::default_llm_model;
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::commands::config::helpers::select_language;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_info, log_message, log_success};
use anyhow::{Context, Result};

/// LLM 配置设置命令
pub struct LLMSetupCommand;

impl LLMSetupCommand {
    /// 交互式设置 LLM 配置
    pub fn setup() -> Result<()> {
        log_break!('=', 40, "LLM Configuration Setup");
        log_break!();

        // 加载当前配置
        let settings = Settings::load();
        let existing = &settings.llm;

        log_message!("  LLM/AI Configuration");
        log_break!('─', 65);

        // 1. 选择 Provider
        let llm_providers = ["openai", "deepseek", "proxy"];
        let current_provider_idx =
            llm_providers.iter().position(|&p| p == existing.provider.as_str()).unwrap_or(0);

        let llm_provider_prompt = format!("Select LLM provider [current: {}]", existing.provider);

        let llm_providers_vec: Vec<String> = llm_providers.iter().map(|s| s.to_string()).collect();
        let llm_provider = SelectDialog::new(&llm_provider_prompt, llm_providers_vec)
            .with_default(current_provider_idx)
            .prompt()
            .context("Failed to select LLM provider")?;
        let llm_provider_idx = llm_providers
            .iter()
            .position(|&p| p == llm_provider.as_str())
            .unwrap_or(current_provider_idx);
        let llm_provider = llm_providers[llm_provider_idx].to_string();

        // 2. 根据 provider 设置 URL（只有 proxy 需要输入和保存）
        let llm_url = match llm_provider.as_str() {
            "openai" => None,   // openai 不使用 proxy URL，必须为 None
            "deepseek" => None, // deepseek 不使用 proxy URL，必须为 None
            "proxy" => {
                let llm_url_prompt = if let Some(ref url) = existing.url {
                    format!("LLM proxy URL [current: {}] (press Enter to keep)", url)
                } else {
                    "LLM proxy URL (optional, press Enter to skip)".to_string()
                };

                let llm_url_input = InputDialog::new(&llm_url_prompt)
                    .allow_empty(true)
                    .prompt()
                    .context("Failed to get LLM proxy URL")?;

                if !llm_url_input.is_empty() {
                    Some(llm_url_input)
                } else {
                    existing.url.clone()
                }
            }
            _ => None,
        };

        // 3. 收集 API key
        let key_prompt = match llm_provider.as_str() {
            "openai" => {
                if existing.key.is_some() {
                    "OpenAI API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "OpenAI API key (optional, press Enter to skip)".to_string()
                }
            }
            "deepseek" => {
                if existing.key.is_some() {
                    "DeepSeek API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "DeepSeek API key (optional, press Enter to skip)".to_string()
                }
            }
            "proxy" => {
                if existing.key.is_some() {
                    "LLM proxy key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "LLM proxy key (optional, press Enter to skip)".to_string()
                }
            }
            _ => "LLM API key (optional, press Enter to skip)".to_string(),
        };

        let llm_key_input = InputDialog::new(&key_prompt)
            .allow_empty(true)
            .prompt()
            .context("Failed to get LLM API key")?;

        let llm_key = if !llm_key_input.is_empty() {
            Some(llm_key_input)
        } else {
            existing.key.clone()
        };

        // 4. 配置模型
        let default_model =
            existing.model.clone().unwrap_or_else(|| default_llm_model(&llm_provider));

        let model_prompt = match llm_provider.as_str() {
            "openai" => {
                if existing.model.is_some() {
                    "OpenAI model (press Enter to keep)".to_string()
                } else {
                    "OpenAI model (optional, press Enter to skip)".to_string()
                }
            }
            "deepseek" => {
                if existing.model.is_some() {
                    "DeepSeek model (press Enter to keep)".to_string()
                } else {
                    "DeepSeek model (optional, press Enter to skip)".to_string()
                }
            }
            "proxy" => {
                if existing.model.is_some() {
                    "LLM model (press Enter to keep)".to_string()
                } else {
                    "LLM model (required)".to_string()
                }
            }
            _ => "LLM model".to_string(),
        };

        let is_proxy = llm_provider == "proxy";
        let has_existing_model = existing.model.is_some();

        let llm_model_input = {
            let mut dialog = InputDialog::new(&model_prompt).allow_empty(!is_proxy);

            if has_existing_model {
                dialog = dialog.with_default(default_model.clone());
            }

            dialog
                .with_validator(move |input: &str| {
                    if input.is_empty() && is_proxy {
                        Err("Model is required for proxy provider".to_string())
                    } else {
                        Ok(())
                    }
                })
                .prompt()
                .context("Failed to get LLM model")?
        };

        let llm_model = if !llm_model_input.is_empty() {
            Some(llm_model_input)
        } else if is_proxy {
            anyhow::bail!("Model is required for proxy provider");
        } else {
            None
        };

        // 5. 配置 LLM 输出语言
        log_break!();
        let current_language = if !existing.language.is_empty() {
            Some(existing.language.as_str())
        } else {
            None
        };

        let llm_language =
            select_language(current_language).context("Failed to select LLM output language")?;

        // 保存配置
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.llm.provider = llm_provider.clone();
            settings.llm.url = llm_url.clone();
            settings.llm.key = llm_key.clone();
            settings.llm.model = llm_model.clone();
            settings.llm.language = llm_language.clone();
        })?;

        log_break!();
        log_success!("LLM configuration saved successfully!");
        log_info!("Provider: {}", llm_provider);
        if let Some(ref url) = llm_url {
            log_info!("Proxy URL: {}", url);
        }
        if let Some(ref model) = llm_model {
            log_info!("Model: {}", model);
        }
        log_info!("Output Language: {}", llm_language);

        Ok(())
    }
}
