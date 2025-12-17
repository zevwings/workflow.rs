//! LLM 配置设置命令
//! 交互式设置 LLM 相关配置（provider, url, key, model, language）

use crate::base::dialog::{InputDialog, SelectDialog};
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::{LLMSettings, Settings};
use crate::commands::config::helpers::select_language;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_info, log_message, log_success};
use color_eyre::{eyre::WrapErr, Result};

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
            .wrap_err("Failed to select LLM provider")?;
        let llm_provider_idx = llm_providers
            .iter()
            .position(|&p| p == llm_provider.as_str())
            .unwrap_or(current_provider_idx);
        let llm_provider = llm_providers[llm_provider_idx].to_string();

        // 2. 根据选择的 provider 配置对应的设置
        match llm_provider.as_str() {
            "openai" => {
                // 配置 OpenAI API key
                let key_prompt = if existing.openai.key.is_some() {
                    "OpenAI API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "OpenAI API key (optional, press Enter to skip)".to_string()
                };

                let llm_key_input = InputDialog::new(&key_prompt)
                    .allow_empty(true)
                    .prompt()
                    .wrap_err("Failed to get OpenAI API key")?;

                let llm_key = if !llm_key_input.is_empty() {
                    Some(llm_key_input)
                } else {
                    existing.openai.key.clone()
                };

                // 配置 OpenAI model
                let default_model = existing
                    .openai
                    .model
                    .clone()
                    .unwrap_or_else(|| LLMSettings::default_model("openai"));

                let model_prompt = if existing.openai.model.is_some() {
                    "OpenAI model (press Enter to keep)".to_string()
                } else {
                    "OpenAI model (optional, press Enter to skip)".to_string()
                };

                let llm_model_input = InputDialog::new(&model_prompt)
                    .allow_empty(true)
                    .with_default(default_model.clone())
                    .prompt()
                    .wrap_err("Failed to get OpenAI model")?;

                let llm_model = if !llm_model_input.is_empty() {
                    Some(llm_model_input)
                } else if existing.openai.model.is_none() {
                    None
                } else {
                    existing.openai.model.clone()
                };

                // 保存配置
                let config_path = Paths::workflow_config()?;
                let manager = ConfigManager::<Settings>::new(config_path);

                manager.update(|settings| {
                    settings.llm.provider = llm_provider.clone();
                    settings.llm.openai.key = llm_key.clone();
                    settings.llm.openai.model = llm_model.clone();
                })?;

                log_break!();
                log_success!("LLM configuration saved successfully!");
                log_info!("Provider: {}", llm_provider);
                if let Some(ref model) = llm_model {
                    log_info!("Model: {}", model);
                }
            }
            "deepseek" => {
                // 配置 DeepSeek API key
                let key_prompt = if existing.deepseek.key.is_some() {
                    "DeepSeek API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "DeepSeek API key (optional, press Enter to skip)".to_string()
                };

                let llm_key_input = InputDialog::new(&key_prompt)
                    .allow_empty(true)
                    .prompt()
                    .wrap_err("Failed to get DeepSeek API key")?;

                let llm_key = if !llm_key_input.is_empty() {
                    Some(llm_key_input)
                } else {
                    existing.deepseek.key.clone()
                };

                // 配置 DeepSeek model
                let default_model = existing
                    .deepseek
                    .model
                    .clone()
                    .unwrap_or_else(|| LLMSettings::default_model("deepseek"));

                let model_prompt = if existing.deepseek.model.is_some() {
                    "DeepSeek model (press Enter to keep)".to_string()
                } else {
                    "DeepSeek model (optional, press Enter to skip)".to_string()
                };

                let llm_model_input = InputDialog::new(&model_prompt)
                    .allow_empty(true)
                    .with_default(default_model.clone())
                    .prompt()
                    .wrap_err("Failed to get DeepSeek model")?;

                let llm_model = if !llm_model_input.is_empty() {
                    Some(llm_model_input)
                } else if existing.deepseek.model.is_none() {
                    None
                } else {
                    existing.deepseek.model.clone()
                };

                // 保存配置
                let config_path = Paths::workflow_config()?;
                let manager = ConfigManager::<Settings>::new(config_path);

                manager.update(|settings| {
                    settings.llm.provider = llm_provider.clone();
                    settings.llm.deepseek.key = llm_key.clone();
                    settings.llm.deepseek.model = llm_model.clone();
                })?;

                log_break!();
                log_success!("LLM configuration saved successfully!");
                log_info!("Provider: {}", llm_provider);
                if let Some(ref model) = llm_model {
                    log_info!("Model: {}", model);
                }
            }
            "proxy" => {
                // 配置 Proxy URL（必填）
                let llm_url_prompt = if existing.proxy.url.is_some() {
                    "LLM proxy URL (required) (press Enter to keep)".to_string()
                } else {
                    "LLM proxy URL (required)".to_string()
                };

                let has_existing_url = existing.proxy.url.is_some();
                let existing_url = existing.proxy.url.clone();

                let llm_url_input = {
                    let mut dialog = InputDialog::new(&llm_url_prompt);

                    // 如果存在现有值，允许空输入（表示保留现有值）
                    // 如果不存在现有值，不允许空输入（必须输入）
                    dialog = dialog.allow_empty(has_existing_url);

                    // 如果存在现有值，设置为默认值
                    if let Some(ref url) = existing.proxy.url {
                        dialog = dialog.with_default(url.clone());
                    }

                    // 验证器：只有当不存在现有值且输入为空时才报错
                    dialog = dialog.with_validator(move |input: &str| {
                        if input.is_empty() && !has_existing_url {
                            Err("LLM proxy URL is required".to_string())
                        } else {
                            Ok(())
                        }
                    });

                    dialog.prompt().wrap_err("Failed to get LLM proxy URL")?
                };

                let llm_url = if !llm_url_input.is_empty() {
                    Some(llm_url_input)
                } else if has_existing_url {
                    // 用户按 Enter 保留现有值
                    existing_url
                } else {
                    color_eyre::eyre::bail!("LLM proxy URL is required");
                };

                // 配置 Proxy API key（必填）
                let key_prompt = if existing.proxy.key.is_some() {
                    "LLM proxy key (required) (press Enter to keep)".to_string()
                } else {
                    "LLM proxy key (required)".to_string()
                };

                let has_existing_key = existing.proxy.key.is_some();
                let existing_key = existing.proxy.key.clone();

                let llm_key_input = {
                    let mut dialog = InputDialog::new(&key_prompt);

                    // 如果存在现有值，允许空输入（表示保留现有值）
                    // 如果不存在现有值，不允许空输入（必须输入）
                    dialog = dialog.allow_empty(has_existing_key);

                    // 验证器：只有当不存在现有值且输入为空时才报错
                    dialog = dialog.with_validator(move |input: &str| {
                        if input.is_empty() && !has_existing_key {
                            Err("LLM proxy key is required".to_string())
                        } else {
                            Ok(())
                        }
                    });

                    dialog.prompt().wrap_err("Failed to get LLM proxy key")?
                };

                let llm_key = if !llm_key_input.is_empty() {
                    Some(llm_key_input)
                } else if has_existing_key {
                    // 用户按 Enter 保留现有值
                    existing_key
                } else {
                    color_eyre::eyre::bail!("LLM proxy key is required");
                };

                // 配置 Proxy model（必填）
                let default_model = existing
                    .proxy
                    .model
                    .clone()
                    .unwrap_or_else(|| LLMSettings::default_model("proxy"));

                let model_prompt = if existing.proxy.model.is_some() {
                    "LLM model (required) (press Enter to keep)".to_string()
                } else {
                    "LLM model (required)".to_string()
                };

                let llm_model_input = InputDialog::new(&model_prompt)
                    .allow_empty(false)
                    .with_default(default_model.clone())
                    .with_validator(|input: &str| {
                        if input.is_empty() {
                            Err("Model is required for proxy provider".to_string())
                        } else {
                            Ok(())
                        }
                    })
                    .prompt()
                    .wrap_err("Failed to get LLM model")?;

                let llm_model = if !llm_model_input.is_empty() {
                    Some(llm_model_input)
                } else {
                    color_eyre::eyre::bail!("Model is required for proxy provider");
                };

                // 保存配置
                let config_path = Paths::workflow_config()?;
                let manager = ConfigManager::<Settings>::new(config_path);

                manager.update(|settings| {
                    settings.llm.provider = llm_provider.clone();
                    settings.llm.proxy.url = llm_url.clone();
                    settings.llm.proxy.key = llm_key.clone();
                    settings.llm.proxy.model = llm_model.clone();
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
            }
            _ => {
                color_eyre::eyre::bail!("Unsupported provider: {}", llm_provider);
            }
        }

        // 5. 配置 LLM 输出语言（所有 provider 共享）
        log_break!();
        let current_language = if !existing.language.is_empty() {
            Some(existing.language.as_str())
        } else {
            None
        };

        let llm_language =
            select_language(current_language).wrap_err("Failed to select LLM output language")?;

        // 保存语言配置
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.llm.language = llm_language.clone();
        })?;

        log_info!("Output Language: {}", llm_language);

        Ok(())
    }
}
