//! Jira 配置处理模块

use crate::config::settings::paths::Paths;
use crate::core::prompt::{Condition, ConfirmFormField, FormBuilder, GroupConfig, InputFormField};
use crate::services::jira::config::{ConfigManager, JiraConfig};
use crate::settings::JiraSettings;
use crate::{br, info, print};
use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;
use std::sync::Arc;

/// 处理 Jira 配置
///
/// 参考 Go 版本的实现逻辑：
/// 1. 检测现有配置（检查服务地址或 API Token）
/// 2. 已有配置时询问是否保留（默认 Yes）
/// 3. 配置收集：服务地址、邮箱、API Token（按 Go 版本的顺序）
/// 4. 已有值时提示可回车保留
/// 5. 仅更新非空字段
pub fn handle_jira_config(existing: JiraSettings) -> Result<JiraSettings> {
    // 显示标题
    br!();

    // 显示检测信息（借鉴 Go 版本）
    // Go 版本检查：服务地址或 API Token 是否存在
    let has_jira_address = existing.service_address.is_some();
    let has_jira_token = existing.api_token.is_some();
    let has_jira = has_jira_address || has_jira_token;
    if has_jira {
        info!("Jira configuration detected.");
        // 尝试从本地缓存获取 account_id
        if let Some(email) = &existing.email {
            let account_id = get_account_id_from_cache(email).ok();
            if let Some(account_id) = account_id {
                print!("  - Email: {} (Account ID: {})", email, account_id);
            } else {
                print!("  - Email: {}", email);
            }
        }
    }

    // 使用 FormBuilder 收集配置
    // 如果已有配置，会在表单的第一个 step 中询问是否保留
    let has_jira_email = existing.email.is_some();
    let form_result = build_jira_form_builder(
        &existing,
        has_jira_email,
        has_jira_address,
        has_jira_token,
        has_jira,
    )
    .run()
    .wrap_err("Failed to collect Jira configuration")?;

    // 检查是否选择保留现有配置
    if let Some(keep) = form_result.get_bool_opt("keep_existing_config") {
        if keep {
            return Ok(existing);
        }
    }

    // 处理结果（仅更新非空字段，参考 Go 版本）
    let jira_service_address = if let Some(address) = form_result.get("jira_service_address") {
        if !address.trim().is_empty() {
            Some(address.trim().to_string())
        } else if has_jira_address {
            existing.service_address.clone()
        } else {
            return Err(color_eyre::eyre::eyre!("Jira service address is required"));
        }
    } else if has_jira_address {
        existing.service_address.clone()
    } else {
        return Err(color_eyre::eyre::eyre!("Jira service address is required"));
    };

    let jira_email = if let Some(email) = form_result.get("jira_email") {
        if !email.trim().is_empty() {
            Some(email.trim().to_string())
        } else if has_jira_email {
            existing.email.clone()
        } else {
            return Err(color_eyre::eyre::eyre!("Jira email address is required"));
        }
    } else if has_jira_email {
        existing.email.clone()
    } else {
        return Err(color_eyre::eyre::eyre!("Jira email address is required"));
    };

    let jira_api_token = if let Some(token) = form_result.get("jira_api_token") {
        if !token.trim().is_empty() {
            Some(token.trim().to_string())
        } else if has_jira_token {
            existing.api_token.clone()
        } else {
            return Err(color_eyre::eyre::eyre!("Jira API token is required"));
        }
    } else if has_jira_token {
        existing.api_token.clone()
    } else {
        return Err(color_eyre::eyre::eyre!("Jira API token is required"));
    };

    Ok(JiraSettings {
        email: jira_email,
        service_address: jira_service_address,
        api_token: jira_api_token,
    })
}

/// 构建 Jira 配置表单
///
/// 按 Go 版本的顺序：服务地址、邮箱、API Token
/// 如果已有配置，会在第一个 step 中询问是否保留
fn build_jira_form_builder(
    existing: &JiraSettings,
    has_jira_email: bool,
    has_jira_address: bool,
    has_jira_token: bool,
    has_jira: bool,
) -> FormBuilder {
    FormBuilder::new().add_group(
        "jira",
        |mut g| {
            // 第一步：如果已有配置，先询问是否保留
            if has_jira {
                g = g.add_step(|s| {
                    s.add_confirm(
                        ConfirmFormField::new(
                            "keep_existing_config",
                            "Existing Jira configuration detected. Do you want to keep the current values?",
                        )
                        .default(true)
                        .result_title("Keep Jira configuration"),
                    )
                });
            }

            // 第二步：Jira service address（按 Go 版本顺序）
            // 只有当没有配置或用户选择不保留时才执行
            g.add_step(|s| {
                // 创建条件函数：如果没有配置，或者用户选择不保留，才执行此字段
                let condition: Option<Condition> = if has_jira {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        // 如果存在 keep_existing_config 字段，检查其值
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep // 如果选择保留，跳过此字段
                        } else {
                            true // 如果没有这个字段，执行此字段
                        }
                    }))
                } else {
                    None // 没有配置，无条件执行
                };

                let jira_address_prompt = if has_jira_address {
                    "Please enter your Jira service address (press Enter to keep)"
                } else {
                    "Please enter your Jira service address (required)"
                };
                let mut field = InputFormField::new("jira_service_address", jira_address_prompt)
                    .result_title("Your Jira service address");

                // 如果有条件，添加条件
                if let Some(cond) = condition {
                    field = field.condition(cond);
                }
                if has_jira_address {
                    field = field.allow_empty(true);
                    if let Some(ref addr) = existing.service_address {
                        field = field.default(addr.clone());
                    }
                } else {
                    field = field.required();
                }
                let has_jira_address_clone = has_jira_address;
                field = field.validator(Arc::new(move |input: &str| {
                    if input.trim().is_empty() && !has_jira_address_clone {
                        Err("Jira service address is required".to_string())
                    } else if !input.trim().is_empty()
                        && !input.trim().starts_with("http://")
                        && !input.trim().starts_with("https://")
                    {
                        Err(
                            "Please enter a valid URL (must start with http:// or https://)"
                                .to_string(),
                        )
                    } else {
                        Ok(())
                    }
                }));
                s.add_input(field)
            })
            // 第三步：Jira email
            // 只有当没有配置或用户选择不保留时才执行
            .add_step(|s| {
                // 创建条件函数：如果没有配置，或者用户选择不保留，才执行此字段
                let condition: Option<Condition> = if has_jira {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    None
                };

                let jira_email_prompt = if has_jira_email {
                    "Please enter your Jira email address (press Enter to keep)"
                } else {
                    "Please enter your Jira email address (required)"
                };
                let mut field = InputFormField::new("jira_email", jira_email_prompt)
                    .result_title("Your Jira email");

                // 如果有条件，添加条件
                if let Some(cond) = condition {
                    field = field.condition(cond);
                }
                if has_jira_email {
                    field = field.allow_empty(true);
                    if let Some(ref email) = existing.email {
                        field = field.default(email.clone());
                    }
                } else {
                    field = field.required();
                }
                let has_jira_email_clone = has_jira_email;
                // 邮箱正则表达式：允许字母、数字、点、下划线、百分号、加号、减号，然后是 @，然后是域名
                let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                    .expect("Invalid email regex pattern");
                let email_regex_clone = email_regex.clone();
                field = field.validator(Arc::new(move |input: &str| {
                    if input.trim().is_empty() && !has_jira_email_clone {
                        Err("Jira email address is required".to_string())
                    } else if !input.trim().is_empty() && !email_regex_clone.is_match(input.trim()) {
                        Err("Please enter a valid email address".to_string())
                    } else {
                        Ok(())
                    }
                }));
                s.add_input(field)
            })
            // 第四步：Jira API token
            // 只有当没有配置或用户选择不保留时才执行
            .add_step(|s| {
                // 创建条件函数：如果没有配置，或者用户选择不保留，才执行此字段
                let condition: Option<Condition> = if has_jira {
                    Some(Box::new(move |result: &crate::prompt::FormResult| {
                        if let Some(keep) = result.get_bool_opt("keep_existing_config") {
                            !keep
                        } else {
                            true
                        }
                    }))
                } else {
                    None
                };

                let jira_token_prompt = if has_jira_token {
                    "Please enter your Jira API token [current: ***] (press Enter to keep)"
                } else {
                    "Please enter your Jira API token (required)"
                };
                let mut field = InputFormField::new("jira_api_token", jira_token_prompt)
                    .result_title("Your Jira API token");

                // 如果有条件，添加条件
                if let Some(cond) = condition {
                    field = field.condition(cond);
                }
                if has_jira_token {
                    field = field.allow_empty(true);
                } else {
                    field = field.required();
                }
                let has_jira_token_clone = has_jira_token;
                field = field.validator(Arc::new(move |input: &str| {
                    if input.trim().is_empty() && !has_jira_token_clone {
                        Err("Jira API token is required".to_string())
                    } else {
                        Ok(())
                    }
                }));
                s.add_input(field)
            })
        },
        GroupConfig::required().with_title("Jira Configuration (Required)"),
    )
}

/// 从本地缓存获取 account_id
fn get_account_id_from_cache(email: &str) -> Result<String> {
    let config_path = Paths::jira_config()?;
    let manager = ConfigManager::<JiraConfig>::new(config_path);
    let config = manager.read()?;

    if let Some(user_entry) = config.users.iter().find(|u| u.email == email) {
        Ok(user_entry.account_id.clone())
    } else {
        color_eyre::eyre::bail!("User with email '{}' not found in jira.toml", email)
    }
}
