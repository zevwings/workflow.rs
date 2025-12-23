//! LLM 配置设置命令
//! 交互式设置 LLM 相关配置（provider, url, key, model, language）

use crate::base::dialog::{FormBuilder, GroupConfig};
use crate::base::llm::{get_supported_language_display_names, SUPPORTED_LANGUAGES};
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::{LLMSettings, Settings};
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

        // 准备配置字段的提示信息
        let llm_providers = ["openai", "deepseek", "proxy"];
        let llm_provider_prompt = format!("Select LLM provider [current: {}]", existing.provider);
        let llm_providers_vec: Vec<String> = llm_providers.iter().map(|s| s.to_string()).collect();

        // OpenAI 配置字段
        let openai_key_prompt = if existing.openai.key.is_some() {
            "OpenAI API key [current: ***] (press Enter to keep)"
        } else {
            "OpenAI API key (optional, press Enter to skip)"
        };
        let openai_model_default = existing
            .openai
            .model
            .clone()
            .unwrap_or_else(|| LLMSettings::default_model("openai"));
        let openai_model_prompt = if existing.openai.model.is_some() {
            "OpenAI model (press Enter to keep)"
        } else {
            "OpenAI model (optional, press Enter to skip)"
        };

        // DeepSeek 配置字段
        let deepseek_key_prompt = if existing.deepseek.key.is_some() {
            "DeepSeek API key [current: ***] (press Enter to keep)"
        } else {
            "DeepSeek API key (optional, press Enter to skip)"
        };
        let deepseek_model_default = existing
            .deepseek
            .model
            .clone()
            .unwrap_or_else(|| LLMSettings::default_model("deepseek"));
        let deepseek_model_prompt = if existing.deepseek.model.is_some() {
            "DeepSeek model (press Enter to keep)"
        } else {
            "DeepSeek model (optional, press Enter to skip)"
        };

        // Proxy 配置字段
        let proxy_url_prompt = if existing.proxy.url.is_some() {
            "LLM proxy URL (required) (press Enter to keep)"
        } else {
            "LLM proxy URL (required)"
        };
        let proxy_key_prompt = if existing.proxy.key.is_some() {
            "LLM proxy key [current: ***] (press Enter to keep)"
        } else {
            "LLM proxy key (required)"
        };
        let proxy_model_prompt = if existing.proxy.model.is_some() {
            "LLM model (press Enter to keep)"
        } else {
            "LLM model (required)"
        };

        let has_existing_proxy_url = existing.proxy.url.is_some();
        let has_existing_proxy_key = existing.proxy.key.is_some();
        let has_existing_proxy_model = existing.proxy.model.is_some();

        // 使用 FormBuilder 收集 LLM 配置
        let form_result = FormBuilder::new()
            .add_group(
                "llm_config",
                |g| {
                    g.step(|f| {
                        f.add_selection("llm_provider", &llm_provider_prompt, llm_providers_vec)
                            .default(existing.provider.clone())
                    })
                    .step_if("llm_provider", "openai", |f| {
                        let mut form =
                            f.add_text("openai_key", openai_key_prompt).allow_empty(true);
                        if let Some(ref key) = existing.openai.key {
                            form = form.default(key.clone());
                        }
                        form.add_text("openai_model", openai_model_prompt)
                            .allow_empty(true)
                            .default(openai_model_default)
                    })
                    .step_if("llm_provider", "deepseek", |f| {
                        let mut form =
                            f.add_text("deepseek_key", deepseek_key_prompt).allow_empty(true);
                        if let Some(ref key) = existing.deepseek.key {
                            form = form.default(key.clone());
                        }
                        form.add_text("deepseek_model", deepseek_model_prompt)
                            .allow_empty(true)
                            .default(deepseek_model_default)
                    })
                    .step_if("llm_provider", "proxy", |f| {
                        let mut form = f.add_text("proxy_url", proxy_url_prompt);
                        if has_existing_proxy_url {
                            form = form.allow_empty(true);
                            if let Some(ref url) = existing.proxy.url {
                                form = form.default(url.clone());
                            }
                        } else {
                            form = form.required();
                        }
                        form = form.validate(move |input: &str| {
                            if input.is_empty() && !has_existing_proxy_url {
                                Err("LLM proxy URL is required".to_string())
                            } else {
                                Ok(())
                            }
                        });

                        let mut form = form.add_text("proxy_key", proxy_key_prompt);
                        if has_existing_proxy_key {
                            form = form.allow_empty(true);
                            // 不设置默认值，这样用户按 Enter 时会保留现有值（显示 *** 而不是明文）
                        } else {
                            form = form.required();
                        }
                        form = form.validate(move |input: &str| {
                            if input.is_empty() && !has_existing_proxy_key {
                                Err("LLM proxy key is required".to_string())
                            } else {
                                Ok(())
                            }
                        });

                        let mut form = form.add_text("proxy_model", proxy_model_prompt);
                        if has_existing_proxy_model {
                            form = form.allow_empty(true);
                            if let Some(ref model) = existing.proxy.model {
                                form = form.default(model.clone());
                            }
                        } else {
                            form = form.required();
                            let default_model = LLMSettings::default_model("proxy");
                            form = form.default(default_model);
                        }
                        form.validate(move |input: &str| {
                            if input.is_empty() && !has_existing_proxy_model {
                                Err("Model is required for proxy provider".to_string())
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .step(|f| {
                        // LLM output language (所有 provider 共享)
                        let language_display_names = get_supported_language_display_names();
                        let current_language = if !existing.language.is_empty() {
                            existing.language.as_str()
                        } else {
                            "en" // 默认英文
                        };
                        let current_idx = SUPPORTED_LANGUAGES
                            .iter()
                            .position(|lang| lang.code == current_language)
                            .unwrap_or(0);
                        let default_display_name = language_display_names
                            .get(current_idx)
                            .cloned()
                            .unwrap_or_else(|| language_display_names[0].clone());
                        let llm_language_prompt =
                            format!("Select LLM output language [current: {}]", current_language);
                        f.add_selection(
                            "llm_language_display",
                            &llm_language_prompt,
                            language_display_names,
                        )
                        .default(default_display_name)
                    })
                },
                GroupConfig::required().with_title("LLM/AI Configuration"),
            )
            .run()
            .wrap_err("Failed to collect LLM configuration")?;

        // 从 form 结果中提取值
        let llm_provider =
            form_result.get_required("llm_provider").wrap_err("LLM provider is required")?;

        // 根据选择的 provider 处理配置
        match llm_provider.as_str() {
            "openai" => {
                // 处理 OpenAI 配置
                let llm_key = if let Some(key) = form_result.get("openai_key") {
                    if !key.is_empty() {
                        Some(key.clone())
                    } else {
                        existing.openai.key.clone()
                    }
                } else {
                    existing.openai.key.clone()
                };

                let llm_model = if let Some(model) = form_result.get("openai_model") {
                    if !model.is_empty() {
                        Some(model.clone())
                    } else if existing.openai.model.is_none() {
                        None
                    } else {
                        existing.openai.model.clone()
                    }
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
                // 处理 DeepSeek 配置
                let llm_key = if let Some(key) = form_result.get("deepseek_key") {
                    if !key.is_empty() {
                        Some(key.clone())
                    } else {
                        existing.deepseek.key.clone()
                    }
                } else {
                    existing.deepseek.key.clone()
                };

                let llm_model = if let Some(model) = form_result.get("deepseek_model") {
                    if !model.is_empty() {
                        Some(model.clone())
                    } else if existing.deepseek.model.is_none() {
                        None
                    } else {
                        existing.deepseek.model.clone()
                    }
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
                // 处理 Proxy 配置
                let llm_url = if let Some(url) = form_result.get("proxy_url") {
                    if !url.is_empty() {
                        Some(url.clone())
                    } else if has_existing_proxy_url {
                        existing.proxy.url.clone()
                    } else {
                        return Err(color_eyre::eyre::eyre!("LLM proxy URL is required"));
                    }
                } else if has_existing_proxy_url {
                    existing.proxy.url.clone()
                } else {
                    return Err(color_eyre::eyre::eyre!("LLM proxy URL is required"));
                };

                let llm_key = if let Some(key) = form_result.get("proxy_key") {
                    if !key.is_empty() {
                        Some(key.clone())
                    } else if has_existing_proxy_key {
                        existing.proxy.key.clone()
                    } else {
                        return Err(color_eyre::eyre::eyre!("LLM proxy key is required"));
                    }
                } else if has_existing_proxy_key {
                    existing.proxy.key.clone()
                } else {
                    return Err(color_eyre::eyre::eyre!("LLM proxy key is required"));
                };

                let llm_model = if let Some(model) = form_result.get("proxy_model") {
                    if !model.is_empty() {
                        Some(model.clone())
                    } else if has_existing_proxy_model {
                        existing.proxy.model.clone()
                    } else {
                        return Err(color_eyre::eyre::eyre!(
                            "Model is required for proxy provider"
                        ));
                    }
                } else if has_existing_proxy_model {
                    existing.proxy.model.clone()
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "Model is required for proxy provider"
                    ));
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

        // 处理结果：LLM 输出语言
        let llm_language = if let Some(display_name) = form_result.get("llm_language_display") {
            // 从显示名称中提取语言代码
            // 格式："{native_name} ({name}) - {code}"
            let language_code = display_name
                .split(" - ")
                .nth(1)
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid language display name format"))?;
            language_code.to_string()
        } else {
            // 使用现有值
            existing.language.clone()
        };

        // 保存语言配置（与 provider 配置一起保存）
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);

        manager.update(|settings| {
            settings.llm.language = llm_language.clone();
        })?;

        log_info!("Output Language: {}", llm_language);

        Ok(())
    }
}
