//! LLM 配置处理模块

use crate::config::settings::LLMSettings;
use crate::core::prompt::{
    Condition, ConfirmFormField, FormBuilder, GroupConfig, InputFormField, SelectFormField,
};
use crate::services::llm::{get_supported_language_display_names, SUPPORTED_LANGUAGES};
use crate::{br, info};
use color_eyre::{eyre::WrapErr, Result};
use std::sync::Arc;

/// 处理 LLM 配置
///
/// 参考 Go 版本的实现逻辑：
/// 1. 检测现有配置（检查 Provider 是否存在）
/// 2. 已有配置时询问是否保留（默认 Yes）
/// 3. 无配置时询问是否配置（默认 No）
/// 4. Provider 选择：openai、deepseek、proxy
/// 5. 按 Provider 配置：API Key（密码）、Model、URL（proxy）
/// 6. 输出语言选择
pub fn handle_llm_config(existing: LLMSettings) -> Result<LLMSettings> {
    // 显示标题
    br!();

    // 显示检测信息（借鉴 Go 版本）
    // Go 版本检查：Provider 是否存在
    let has_llm =
        !existing.provider.is_empty() && existing.provider != LLMSettings::default_provider();
    if has_llm {
        info!(
            "LLM configuration detected (Provider: {}).",
            existing.provider
        );
    } else {
        info!("No LLM configuration detected.");
    }
    br!();

    // 使用 FormBuilder 收集配置
    // 如果已有配置，会在表单的第一个 step 中询问是否保留
    // 如果没有配置，会在表单的第一个 step 中询问是否配置
    let form_result = build_llm_form(&existing, has_llm)
        .run()
        .wrap_err("Failed to collect LLM configuration")?;

    // 检查是否选择保留现有配置或选择不配置
    if has_llm {
        // 如果有配置，检查是否选择保留
        if let Some(keep) = form_result.get_bool_opt("keep_existing_config") {
            if keep {
                return Ok(existing);
            }
        }
    } else {
        // 如果没有配置，检查是否选择配置
        if let Some(configure) = form_result.get_bool_opt("configure_llm") {
            if !configure {
                return Ok(existing);
            }
        }
    }

    // 处理结果
    parse_llm_result(form_result, &existing)
}

/// 构建 LLM 配置表单
///
/// 如果已有配置，会在第一个 step 中询问是否保留
/// 如果没有配置，会在第一个 step 中询问是否配置
fn build_llm_form(existing: &LLMSettings, has_llm: bool) -> FormBuilder {
    let llm_providers = vec![
        "openai".to_string(),
        "deepseek".to_string(),
        "proxy".to_string(),
    ];
    let llm_provider_prompt = format!(
        "Please select your LLM provider [current: {}]",
        existing.provider
    );

    // OpenAI 配置字段
    let openai_key_prompt = if existing.openai.key.is_some() {
        "Please enter your OpenAI API key [current: ***] (press Enter to keep)"
    } else {
        "Please enter your OpenAI API key (optional, press Enter to skip)"
    };
    let openai_model_default = existing
        .openai
        .model
        .clone()
        .unwrap_or_else(|| LLMSettings::default_model("openai"));
    let openai_model_prompt = if existing.openai.model.is_some() {
        "Please enter your OpenAI model (press Enter to keep)"
    } else {
        "Please enter your OpenAI model (optional, press Enter to skip)"
    };

    // DeepSeek 配置字段
    let deepseek_key_prompt = if existing.deepseek.key.is_some() {
        "Please enter your DeepSeek API key [current: ***] (press Enter to keep)"
    } else {
        "Please enter your DeepSeek API key (optional, press Enter to skip)"
    };
    let deepseek_model_default = existing
        .deepseek
        .model
        .clone()
        .unwrap_or_else(|| LLMSettings::default_model("deepseek"));
    let deepseek_model_prompt = if existing.deepseek.model.is_some() {
        "Please enter your DeepSeek model (press Enter to keep)"
    } else {
        "Please enter your DeepSeek model (optional, press Enter to skip)"
    };

    // Proxy 配置字段
    let proxy_url_prompt = if existing.proxy.url.is_some() {
        "Please enter your LLM proxy URL (required) (press Enter to keep)"
    } else {
        "Please enter your LLM proxy URL (required)"
    };
    let proxy_key_prompt = if existing.proxy.key.is_some() {
        "Please enter your LLM proxy key [current: ***] (press Enter to keep)"
    } else {
        "Please enter your LLM proxy key (required)"
    };
    let proxy_model_prompt = if existing.proxy.model.is_some() {
        "Please enter your LLM model (press Enter to keep)"
    } else {
        "Please enter your LLM model (required)"
    };

    let has_existing_proxy_url = existing.proxy.url.is_some();
    let has_existing_proxy_key = existing.proxy.key.is_some();
    let has_existing_proxy_model = existing.proxy.model.is_some();

    FormBuilder::new().add_group(
        "llm",
        |mut g| {
            // 第一步：确认逻辑
            if has_llm {
                // 如果有配置，询问是否保留
                g = g.add_step(|s| {
                    s.add_confirm(
                        ConfirmFormField::new(
                            "keep_existing_config",
                            format!(
                                "Existing LLM configuration detected (Provider: {}). Do you want to keep the current values?",
                                existing.provider
                            ),
                        )
                        .default(true)
                        .result_title("Keep LLM configuration"),
                    )
                });
            } else {
                // 如果没有配置，询问是否配置
                g = g.add_step(|s| {
                    s.add_confirm(
                        ConfirmFormField::new(
                            "configure_llm",
                            "Configure LLM/AI Configuration (Optional)?",
                        )
                        .default(false)
                        .result_title("Configure LLM"),
                    )
                });
            }

            // 第二步：Provider 选择
            // 只有当没有配置或用户选择不保留/选择配置时才执行
            g.add_step({
                let llm_providers_clone = llm_providers.clone();
                let llm_provider_prompt_clone = llm_provider_prompt.clone();
                let existing_provider = existing.provider.clone();
                move |s| {
                    // 创建条件函数
                    let condition: Option<Condition> = if has_llm {
                        // 如果有配置，只有当用户选择不保留时才执行
                        Some(Box::new(move |result: &crate::prompt::FormResult| {
                            if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                                !keep
                            } else {
                                true
                            }
                        }))
                    } else {
                        // 如果没有配置，只有当用户选择配置时才执行
                        Some(Box::new(move |result: &crate::prompt::FormResult| {
                            result.get_bool_opt("configure_llm").unwrap_or_default()
                        }))
                    };

                    let mut field = SelectFormField::new("llm_provider", &llm_provider_prompt_clone, llm_providers_clone.clone());
                    // 找到默认 provider 的索引
                    let default_idx =
                        llm_providers_clone.iter().position(|p| p == &existing_provider).unwrap_or(0);
                    field = field.default(default_idx).result_title("Your LLM provider");

                // 如果有条件，添加条件
                if let Some(cond) = condition {
                    field = field.condition(cond);
                }

                s.add_select(field)
                }
            })
            .step_if("llm_provider", "openai", |s| {
                // 创建条件函数：只有当用户选择配置时才执行
                let has_llm_clone = has_llm;
                let condition_key: Option<Condition> = if has_llm_clone {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let has_llm_clone2 = has_llm;
                let condition_model: Option<Condition> = if has_llm_clone2 {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let mut key_field = InputFormField::new("llm_openai_key", openai_key_prompt)
                    .allow_empty(true)
                    .result_title("Your OpenAI API key");
                if let Some(ref key) = existing.openai.key {
                    key_field = key_field.default(key.clone());
                }
                if let Some(cond) = condition_key {
                    key_field = key_field.condition(cond);
                }

                let mut model_field = InputFormField::new("llm_openai_model", openai_model_prompt)
                    .allow_empty(true)
                    .default(openai_model_default)
                    .result_title("Your OpenAI model");
                if let Some(cond) = condition_model {
                    model_field = model_field.condition(cond);
                }

                s.add_input(key_field).add_input(model_field)
            })
            .step_if("llm_provider", "deepseek", |s| {
                // 创建条件函数：只有当用户选择配置时才执行
                let has_llm_clone = has_llm;
                let condition_key: Option<Condition> = if has_llm_clone {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let has_llm_clone2 = has_llm;
                let condition_model: Option<Condition> = if has_llm_clone2 {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let mut key_field = InputFormField::new("llm_deepseek_key", deepseek_key_prompt)
                    .allow_empty(true)
                    .result_title("Your DeepSeek API key");
                if let Some(ref key) = existing.deepseek.key {
                    key_field = key_field.default(key.clone());
                }
                if let Some(cond) = condition_key {
                    key_field = key_field.condition(cond);
                }

                let mut model_field = InputFormField::new("llm_deepseek_model", deepseek_model_prompt)
                    .allow_empty(true)
                    .default(deepseek_model_default)
                    .result_title("Your DeepSeek model");
                if let Some(cond) = condition_model {
                    model_field = model_field.condition(cond);
                }

                s.add_input(key_field).add_input(model_field)
            })
            .step_if("llm_provider", "proxy", |s| {
                // 创建条件函数：只有当用户选择配置时才执行
                let has_llm_clone = has_llm;
                let condition_url: Option<Condition> = if has_llm_clone {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let has_llm_clone2 = has_llm;
                let condition_key: Option<Condition> = if has_llm_clone2 {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let has_llm_clone3 = has_llm;
                let condition_model: Option<Condition> = if has_llm_clone3 {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

                let mut url_field = InputFormField::new("llm_proxy_url", proxy_url_prompt)
                    .result_title("Your LLM proxy URL");
                if has_existing_proxy_url {
                    url_field = url_field.allow_empty(true);
                    if let Some(ref url) = existing.proxy.url {
                        url_field = url_field.default(url.clone());
                    }
                } else {
                    url_field = url_field.required();
                }
                if let Some(cond) = condition_url {
                    url_field = url_field.condition(cond);
                }
                let has_existing_proxy_url_clone = has_existing_proxy_url;
                url_field = url_field.validator(Arc::new(move |input: &str| {
                    if input.trim().is_empty() && !has_existing_proxy_url_clone {
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
                if let Some(cond) = condition_key {
                    key_field = key_field.condition(cond);
                }
                let has_existing_proxy_key_clone = has_existing_proxy_key;
                key_field = key_field.validator(Arc::new(move |input: &str| {
                    if input.trim().is_empty() && !has_existing_proxy_key_clone {
                        Err("LLM proxy key is required".to_string())
                    } else {
                        Ok(())
                    }
                }));

                let mut model_field = InputFormField::new("llm_proxy_model", proxy_model_prompt)
                    .result_title("Your LLM model");
                if has_existing_proxy_model {
                    model_field = model_field.allow_empty(true);
                    if let Some(ref model) = existing.proxy.model {
                        model_field = model_field.default(model.clone());
                    }
                } else {
                    model_field = model_field.required();
                    let default_model = LLMSettings::default_model("proxy");
                    model_field = model_field.default(default_model);
                }
                if let Some(cond) = condition_model {
                    model_field = model_field.condition(cond);
                }
                let has_existing_proxy_model_clone = has_existing_proxy_model;
                model_field = model_field.validator(Arc::new(move |input: &str| {
                    if input.trim().is_empty() && !has_existing_proxy_model_clone {
                        Err("Model is required for proxy provider".to_string())
                    } else {
                        Ok(())
                    }
                }));

                s.add_input(url_field).add_input(key_field).add_input(model_field)
            })
            .add_step(|s| {
                // LLM output language (所有 provider 共享)
                // 只有当用户选择配置时才执行
                let condition: Option<Condition> = if has_llm {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        result.get_bool_opt("configure_llm").unwrap_or_default()
                    }))
                };

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
                let llm_language_prompt = format!(
                    "Please select your output language [current: {}]",
                    current_language
                );
                // 找到默认显示名称的索引
                let default_idx = language_display_names
                    .iter()
                    .position(|name| name == &default_display_name)
                    .unwrap_or(0);
                let mut field = SelectFormField::new(
                    "llm_language_display",
                    &llm_language_prompt,
                    language_display_names,
                )
                .default(default_idx)
                .result_title("Your output language");

                // 如果有条件，添加条件
                if let Some(cond) = condition {
                    field = field.condition(cond);
                }

                s.add_select(field)
            })
        },
        // 用户已经确认要配置（无论是首次配置还是更新配置），所以使用 required（不再询问）
        GroupConfig::required().with_title("LLM/AI Configuration (Optional)"),
    )
}

/// 解析 LLM 配置结果
fn parse_llm_result(
    form_result: crate::prompt::FormResult,
    existing: &LLMSettings,
) -> Result<LLMSettings> {
    // 如果用户选择不配置 LLM 组，使用现有值
    let (provider, openai, deepseek, proxy) =
        if let Some(selected_provider) = form_result.get("llm_provider") {
            // 用户配置了 LLM 组，处理配置
            let provider = selected_provider.clone();

            // 初始化各 provider 的配置（从 existing 加载，保持其他 provider 的配置不变）
            let mut openai = existing.openai.clone();
            let mut deepseek = existing.deepseek.clone();
            let mut proxy = existing.proxy.clone();

            // 根据选择的 provider 更新对应的配置
            let has_existing_proxy_url = existing.proxy.url.is_some();
            let has_existing_proxy_key = existing.proxy.key.is_some();
            let has_existing_proxy_model = existing.proxy.model.is_some();

            match provider.as_str() {
                "openai" => {
                    // 更新 OpenAI 配置（仅更新非空字段，参考 Go 版本）
                    if let Some(key) = form_result.get("llm_openai_key") {
                        if !key.trim().is_empty() {
                            openai.key = Some(key.trim().to_string());
                        }
                    }
                    if let Some(model) = form_result.get("llm_openai_model") {
                        if !model.trim().is_empty() {
                            openai.model = Some(model.trim().to_string());
                        } else if openai.model.is_none() {
                            openai.model = None;
                        }
                    }
                }
                "deepseek" => {
                    // 更新 DeepSeek 配置（仅更新非空字段，参考 Go 版本）
                    if let Some(key) = form_result.get("llm_deepseek_key") {
                        if !key.trim().is_empty() {
                            deepseek.key = Some(key.trim().to_string());
                        }
                    }
                    if let Some(model) = form_result.get("llm_deepseek_model") {
                        if !model.trim().is_empty() {
                            deepseek.model = Some(model.trim().to_string());
                        } else if deepseek.model.is_none() {
                            deepseek.model = None;
                        }
                    }
                }
                "proxy" => {
                    // 更新 Proxy 配置（仅更新非空字段，参考 Go 版本）
                    if let Some(url) = form_result.get("llm_proxy_url") {
                        if !url.trim().is_empty() {
                            proxy.url = Some(url.trim().to_string());
                        } else if has_existing_proxy_url {
                            // 用户按 Enter 保留现有值
                            proxy.url = existing.proxy.url.clone();
                        } else {
                            color_eyre::eyre::bail!("LLM proxy URL is required");
                        }
                    }
                    if let Some(key) = form_result.get("llm_proxy_key") {
                        if !key.trim().is_empty() {
                            proxy.key = Some(key.trim().to_string());
                        } else if has_existing_proxy_key {
                            // 用户按 Enter 保留现有值
                            proxy.key = existing.proxy.key.clone();
                        } else {
                            color_eyre::eyre::bail!("LLM proxy key is required");
                        }
                    }
                    if let Some(model) = form_result.get("llm_proxy_model") {
                        if !model.trim().is_empty() {
                            proxy.model = Some(model.trim().to_string());
                        } else if has_existing_proxy_model {
                            // 用户按 Enter 保留现有值
                            proxy.model = existing.proxy.model.clone();
                        } else {
                            color_eyre::eyre::bail!("Model is required for proxy provider");
                        }
                    }
                }
                _ => {}
            }

            (provider, openai, deepseek, proxy)
        } else {
            // 用户选择不配置 LLM 组，使用现有值
            (
                existing.provider.clone(),
                existing.openai.clone(),
                existing.deepseek.clone(),
                existing.proxy.clone(),
            )
        };

    // 处理结果：LLM 输出语言
    // 如果用户选择不配置 LLM 组，使用现有值
    let language = if let Some(display_name) = form_result.get("llm_language_display") {
        // 从显示名称中提取语言代码
        // 格式："{native_name} ({name}) - {code}"
        let language_code = display_name
            .split(" - ")
            .nth(1)
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid language display name format"))?;
        language_code.to_string()
    } else {
        // 用户选择不配置 LLM 组，使用现有值
        existing.language.clone()
    };

    Ok(LLMSettings {
        provider,
        language,
        openai,
        deepseek,
        proxy,
    })
}
