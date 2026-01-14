//! LLM 配置处理模块

use crate::commands::setup::types::{CollectedConfig, LLMConfig};
use crate::interactive::{FormBuilder, GroupConfig, InputFormField, SelectFormField};
use crate::llm::{get_supported_language_display_names, SUPPORTED_LANGUAGES};
use crate::settings::LLMSettings;
use crate::{br, info};
use color_eyre::{eyre::WrapErr, Result};
use std::sync::Arc;

/// 处理 LLM 配置
pub fn handle_llm_config(existing: &CollectedConfig) -> Result<LLMConfig> {
    // 显示标题
    br!();
    info!("  LLM/AI Configuration (Optional)");
    br!('─', 65);

    // 显示检测信息（借鉴 Go 版本）
    let has_llm = !existing.llm_provider.is_empty();
    if has_llm {
        info!(
            "LLM configuration detected (Provider: {}).",
            existing.llm_provider
        );
    } else {
        info!("No LLM configuration detected.");
    }
    br!();

    // 如果存在配置，先询问是否保留（借鉴 Go 版本）
    if has_llm {
        let keep = crate::confirm!(
            "Existing LLM configuration detected (Provider: {}). Do you want to keep the current values?",
            existing.llm_provider
        )
        .default(true)
        .result_title("Keep LLM configuration")
        .prompt()
        .wrap_err("Failed to get confirmation")?;

        if keep {
            return Ok(LLMConfig {
                provider: existing.llm_provider.clone(),
                language: existing.llm_language.clone(),
                openai_key: existing.llm_openai_key.clone(),
                openai_model: existing.llm_openai_model.clone(),
                deepseek_key: existing.llm_deepseek_key.clone(),
                deepseek_model: existing.llm_deepseek_model.clone(),
                proxy_url: existing.llm_proxy_url.clone(),
                proxy_key: existing.llm_proxy_key.clone(),
                proxy_model: existing.llm_proxy_model.clone(),
            });
        }
    } else {
        // 如果不存在配置，询问是否配置
        let configure = crate::confirm!("Do you want to configure LLM?")
            .default(false)
            .result_title("Configure LLM")
            .prompt()
            .wrap_err("Failed to get confirmation")?;

        if !configure {
            return Ok(LLMConfig {
                provider: existing.llm_provider.clone(),
                language: existing.llm_language.clone(),
                openai_key: existing.llm_openai_key.clone(),
                openai_model: existing.llm_openai_model.clone(),
                deepseek_key: existing.llm_deepseek_key.clone(),
                deepseek_model: existing.llm_deepseek_model.clone(),
                proxy_url: existing.llm_proxy_url.clone(),
                proxy_key: existing.llm_proxy_key.clone(),
                proxy_model: existing.llm_proxy_model.clone(),
            });
        }
    }

    // 使用 FormBuilder 收集配置
    let form_result =
        build_llm_form(existing).run().wrap_err("Failed to collect LLM configuration")?;

    // 处理结果
    parse_llm_result(form_result, existing)
}

/// 构建 LLM 配置表单
fn build_llm_form(existing: &CollectedConfig) -> FormBuilder {
    let llm_providers = vec![
        "openai".to_string(),
        "deepseek".to_string(),
        "proxy".to_string(),
    ];
    let llm_provider_prompt = format!(
        "Please select your LLM provider [current: {}]",
        existing.llm_provider
    );

    // OpenAI 配置字段
    let openai_key_prompt = if existing.llm_openai_key.is_some() {
        "Please enter your OpenAI API key [current: ***] (press Enter to keep)"
    } else {
        "Please enter your OpenAI API key (optional, press Enter to skip)"
    };
    let openai_model_default = existing
        .llm_openai_model
        .clone()
        .unwrap_or_else(|| LLMSettings::default_model("openai"));
    let openai_model_prompt = if existing.llm_openai_model.is_some() {
        "Please enter your OpenAI model (press Enter to keep)"
    } else {
        "Please enter your OpenAI model (optional, press Enter to skip)"
    };

    // DeepSeek 配置字段
    let deepseek_key_prompt = if existing.llm_deepseek_key.is_some() {
        "Please enter your DeepSeek API key [current: ***] (press Enter to keep)"
    } else {
        "Please enter your DeepSeek API key (optional, press Enter to skip)"
    };
    let deepseek_model_default = existing
        .llm_deepseek_model
        .clone()
        .unwrap_or_else(|| LLMSettings::default_model("deepseek"));
    let deepseek_model_prompt = if existing.llm_deepseek_model.is_some() {
        "Please enter your DeepSeek model (press Enter to keep)"
    } else {
        "Please enter your DeepSeek model (optional, press Enter to skip)"
    };

    // Proxy 配置字段
    let proxy_url_prompt = if existing.llm_proxy_url.is_some() {
        "Please enter your LLM proxy URL (required) (press Enter to keep)"
    } else {
        "Please enter your LLM proxy URL (required)"
    };
    let proxy_key_prompt = if existing.llm_proxy_key.is_some() {
        "Please enter your LLM proxy key [current: ***] (press Enter to keep)"
    } else {
        "Please enter your LLM proxy key (required)"
    };
    let proxy_model_prompt = if existing.llm_proxy_model.is_some() {
        "Please enter your LLM model (press Enter to keep)"
    } else {
        "Please enter your LLM model (required)"
    };

    let has_existing_proxy_url = existing.llm_proxy_url.is_some();
    let has_existing_proxy_key = existing.llm_proxy_key.is_some();
    let has_existing_proxy_model = existing.llm_proxy_model.is_some();

    FormBuilder::new().add_group(
        "llm",
        |g| {
            g.add_step(|s| {
                // 找到默认 provider 的索引
                let default_idx =
                    llm_providers.iter().position(|p| p == &existing.llm_provider).unwrap_or(0);
                s.add_select(
                    SelectFormField::new("llm_provider", &llm_provider_prompt, llm_providers)
                        .default(default_idx)
                        .result_title("Your LLM provider"),
                )
            })
            .step_if("llm_provider", "openai", |s| {
                let mut key_field = InputFormField::new("llm_openai_key", openai_key_prompt)
                    .allow_empty(true)
                    .result_title("Your OpenAI API key");
                if let Some(ref key) = existing.llm_openai_key {
                    key_field = key_field.default(key.clone());
                }
                let model_field = InputFormField::new("llm_openai_model", openai_model_prompt)
                    .allow_empty(true)
                    .default(openai_model_default)
                    .result_title("Your OpenAI model");
                s.add_input(key_field).add_input(model_field)
            })
            .step_if("llm_provider", "deepseek", |s| {
                let mut key_field = InputFormField::new("llm_deepseek_key", deepseek_key_prompt)
                    .allow_empty(true)
                    .result_title("Your DeepSeek API key");
                if let Some(ref key) = existing.llm_deepseek_key {
                    key_field = key_field.default(key.clone());
                }
                let model_field = InputFormField::new("llm_deepseek_model", deepseek_model_prompt)
                    .allow_empty(true)
                    .default(deepseek_model_default)
                    .result_title("Your DeepSeek model");
                s.add_input(key_field).add_input(model_field)
            })
            .step_if("llm_provider", "proxy", |s| {
                let mut url_field = InputFormField::new("llm_proxy_url", proxy_url_prompt)
                    .result_title("Your LLM proxy URL");
                if has_existing_proxy_url {
                    url_field = url_field.allow_empty(true);
                    if let Some(ref url) = existing.llm_proxy_url {
                        url_field = url_field.default(url.clone());
                    }
                } else {
                    url_field = url_field.required();
                }
                let has_existing_proxy_url_clone = has_existing_proxy_url;
                url_field = url_field.validator(Arc::new(move |input: &str| {
                    if input.is_empty() && !has_existing_proxy_url_clone {
                        Err("LLM proxy URL is required".to_string())
                    } else {
                        Ok(())
                    }
                }));

                let mut key_field = InputFormField::new("llm_proxy_key", proxy_key_prompt)
                    .result_title("Your LLM proxy key");
                if has_existing_proxy_key {
                    key_field = key_field.allow_empty(true);
                } else {
                    key_field = key_field.required();
                }
                let has_existing_proxy_key_clone = has_existing_proxy_key;
                key_field = key_field.validator(Arc::new(move |input: &str| {
                    if input.is_empty() && !has_existing_proxy_key_clone {
                        Err("LLM proxy key is required".to_string())
                    } else {
                        Ok(())
                    }
                }));

                let mut model_field = InputFormField::new("llm_proxy_model", proxy_model_prompt)
                    .result_title("Your LLM model");
                if has_existing_proxy_model {
                    model_field = model_field.allow_empty(true);
                    if let Some(ref model) = existing.llm_proxy_model {
                        model_field = model_field.default(model.clone());
                    }
                } else {
                    model_field = model_field.required();
                    let default_model = LLMSettings::default_model("proxy");
                    model_field = model_field.default(default_model);
                }
                let has_existing_proxy_model_clone = has_existing_proxy_model;
                model_field = model_field.validator(Arc::new(move |input: &str| {
                    if input.is_empty() && !has_existing_proxy_model_clone {
                        Err("Model is required for proxy provider".to_string())
                    } else {
                        Ok(())
                    }
                }));

                s.add_input(url_field).add_input(key_field).add_input(model_field)
            })
            .add_step(|s| {
                // LLM output language (所有 provider 共享)
                let language_display_names = get_supported_language_display_names();
                let current_language = if !existing.llm_language.is_empty() {
                    existing.llm_language.as_str()
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
                let llm_language_prompt = format!(
                    "Please select your output language [current: {}]",
                    current_language
                );
                // 找到默认显示名称的索引
                let default_idx = language_display_names
                    .iter()
                    .position(|name| name == &default_display_name)
                    .unwrap_or(0);
                s.add_select(
                    SelectFormField::new(
                        "llm_language_display",
                        &llm_language_prompt,
                        language_display_names,
                    )
                    .default(default_idx)
                    .result_title("Your output language"),
                )
            })
        },
        GroupConfig::optional().with_title("LLM/AI Configuration (Optional)"),
    )
}

/// 解析 LLM 配置结果
fn parse_llm_result(
    form_result: crate::interactive::FormResult,
    existing: &CollectedConfig,
) -> Result<LLMConfig> {
    // 如果用户选择不配置 LLM 组，使用现有值
    let (
        llm_provider,
        llm_openai_key,
        llm_openai_model,
        llm_deepseek_key,
        llm_deepseek_model,
        llm_proxy_url,
        llm_proxy_key,
        llm_proxy_model,
    ) = if let Some(provider) = form_result.get("llm_provider") {
        // 用户配置了 LLM 组，处理配置
        let provider = provider.clone();

        // 初始化各 provider 的配置（从 existing 加载，保持其他 provider 的配置不变）
        let mut llm_openai_key = existing.llm_openai_key.clone();
        let mut llm_openai_model = existing.llm_openai_model.clone();
        let mut llm_deepseek_key = existing.llm_deepseek_key.clone();
        let mut llm_deepseek_model = existing.llm_deepseek_model.clone();
        let mut llm_proxy_url = existing.llm_proxy_url.clone();
        let mut llm_proxy_key = existing.llm_proxy_key.clone();
        let mut llm_proxy_model = existing.llm_proxy_model.clone();

        // 根据选择的 provider 更新对应的配置
        let has_existing_proxy_url = existing.llm_proxy_url.is_some();
        let has_existing_proxy_key = existing.llm_proxy_key.is_some();
        let has_existing_proxy_model = existing.llm_proxy_model.is_some();

        match provider.as_str() {
            "openai" => {
                // 更新 OpenAI 配置
                if let Some(key) = form_result.get("llm_openai_key") {
                    if !key.is_empty() {
                        llm_openai_key = Some(key.clone());
                    }
                }
                if let Some(model) = form_result.get("llm_openai_model") {
                    if !model.is_empty() {
                        llm_openai_model = Some(model.clone());
                    } else if llm_openai_model.is_none() {
                        llm_openai_model = None;
                    }
                }
            }
            "deepseek" => {
                // 更新 DeepSeek 配置
                if let Some(key) = form_result.get("llm_deepseek_key") {
                    if !key.is_empty() {
                        llm_deepseek_key = Some(key.clone());
                    }
                }
                if let Some(model) = form_result.get("llm_deepseek_model") {
                    if !model.is_empty() {
                        llm_deepseek_model = Some(model.clone());
                    } else if llm_deepseek_model.is_none() {
                        llm_deepseek_model = None;
                    }
                }
            }
            "proxy" => {
                // 更新 Proxy 配置
                if let Some(url) = form_result.get("llm_proxy_url") {
                    if !url.is_empty() {
                        llm_proxy_url = Some(url.clone());
                    } else if has_existing_proxy_url {
                        // 用户按 Enter 保留现有值
                        llm_proxy_url = existing.llm_proxy_url.clone();
                    } else {
                        color_eyre::eyre::bail!("LLM proxy URL is required");
                    }
                }
                if let Some(key) = form_result.get("llm_proxy_key") {
                    if !key.is_empty() {
                        llm_proxy_key = Some(key.clone());
                    } else if has_existing_proxy_key {
                        // 用户按 Enter 保留现有值
                        llm_proxy_key = existing.llm_proxy_key.clone();
                    } else {
                        color_eyre::eyre::bail!("LLM proxy key is required");
                    }
                }
                if let Some(model) = form_result.get("llm_proxy_model") {
                    if !model.is_empty() {
                        llm_proxy_model = Some(model.clone());
                    } else if has_existing_proxy_model {
                        // 用户按 Enter 保留现有值
                        llm_proxy_model = existing.llm_proxy_model.clone();
                    } else {
                        color_eyre::eyre::bail!("Model is required for proxy provider");
                    }
                }
            }
            _ => {}
        }

        (
            provider,
            llm_openai_key,
            llm_openai_model,
            llm_deepseek_key,
            llm_deepseek_model,
            llm_proxy_url,
            llm_proxy_key,
            llm_proxy_model,
        )
    } else {
        // 用户选择不配置 LLM 组，使用现有值
        (
            existing.llm_provider.clone(),
            existing.llm_openai_key.clone(),
            existing.llm_openai_model.clone(),
            existing.llm_deepseek_key.clone(),
            existing.llm_deepseek_model.clone(),
            existing.llm_proxy_url.clone(),
            existing.llm_proxy_key.clone(),
            existing.llm_proxy_model.clone(),
        )
    };

    // 处理结果：LLM 输出语言
    // 如果用户选择不配置 LLM 组，使用现有值
    let llm_language = if let Some(display_name) = form_result.get("llm_language_display") {
        // 从显示名称中提取语言代码
        // 格式："{native_name} ({name}) - {code}"
        let language_code = display_name
            .split(" - ")
            .nth(1)
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid language display name format"))?;
        language_code.to_string()
    } else {
        // 用户选择不配置 LLM 组，使用现有值
        existing.llm_language.clone()
    };

    Ok(LLMConfig {
        provider: llm_provider,
        language: llm_language,
        openai_key: llm_openai_key,
        openai_model: llm_openai_model,
        deepseek_key: llm_deepseek_key,
        deepseek_model: llm_deepseek_model,
        proxy_url: llm_proxy_url,
        proxy_key: llm_proxy_key,
        proxy_model: llm_proxy_model,
    })
}
