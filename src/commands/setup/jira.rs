//! Jira 配置处理模块

use crate::base::interactive::{FormBuilder, GroupConfig, InputFormField};
use crate::commands::setup::types::{CollectedConfig, JiraConfig};
use crate::{br, info};
use color_eyre::{eyre::WrapErr, Result};
use std::sync::Arc;

/// 处理 Jira 配置
pub fn handle_jira_config(existing: &CollectedConfig) -> Result<JiraConfig> {
    // 显示标题
    br!();
    info!("  Jira Configuration (Required)");
    br!('─', 65);

    // 显示检测信息（借鉴 Go 版本）
    let has_jira_email = existing.jira_email.is_some();
    let has_jira_address = existing.jira_service_address.is_some();
    let has_jira_token = existing.jira_api_token.is_some();
    let has_jira = has_jira_email || has_jira_address || has_jira_token;
    if has_jira {
        info!("Jira configuration detected.");
    } else {
        info!("No Jira configuration detected.");
    }
    br!();

    // 如果存在配置，先询问是否保留（借鉴 Go 版本）
    if has_jira {
        let keep = crate::confirm!(
            "Existing Jira configuration detected. Do you want to keep the current values?"
        )
        .default(true)
        .result_title("Keep Jira configuration")
        .prompt()
        .wrap_err("Failed to get confirmation")?;

        if keep {
            return Ok(JiraConfig {
                email: existing.jira_email.clone(),
                service_address: existing.jira_service_address.clone(),
                api_token: existing.jira_api_token.clone(),
            });
        }
    }

    // 使用 FormBuilder 收集配置
    let form_result = build_jira_form(existing, has_jira_email, has_jira_address, has_jira_token)
        .run()
        .wrap_err("Failed to collect Jira configuration")?;

    // 处理结果
    let jira_email = if let Some(email) = form_result.get("jira_email") {
        if !email.is_empty() {
            Some(email.clone())
        } else if has_jira_email {
            existing.jira_email.clone()
        } else {
            return Err(color_eyre::eyre::eyre!("Jira email address is required"));
        }
    } else {
        return Err(color_eyre::eyre::eyre!("Jira email address is required"));
    };

    let jira_service_address = if let Some(address) = form_result.get("jira_service_address") {
        if !address.is_empty() {
            Some(address.clone())
        } else if has_jira_address {
            existing.jira_service_address.clone()
        } else {
            return Err(color_eyre::eyre::eyre!("Jira service address is required"));
        }
    } else {
        return Err(color_eyre::eyre::eyre!("Jira service address is required"));
    };

    let jira_api_token = if let Some(token) = form_result.get("jira_api_token") {
        if !token.is_empty() {
            Some(token.clone())
        } else if has_jira_token {
            existing.jira_api_token.clone()
        } else {
            return Err(color_eyre::eyre::eyre!("Jira API token is required"));
        }
    } else {
        return Err(color_eyre::eyre::eyre!("Jira API token is required"));
    };

    Ok(JiraConfig {
        email: jira_email,
        service_address: jira_service_address,
        api_token: jira_api_token,
    })
}

/// 构建 Jira 配置表单
fn build_jira_form(
    existing: &CollectedConfig,
    has_jira_email: bool,
    has_jira_address: bool,
    has_jira_token: bool,
) -> FormBuilder {
    FormBuilder::new().add_group(
        "jira",
        |g| {
            g.add_step(|s| {
                // Jira email
                let jira_email_prompt = if has_jira_email {
                    "Please enter your Jira email address (press Enter to keep)"
                } else {
                    "Please enter your Jira email address (required)"
                };
                let mut field = InputFormField::new("jira_email", jira_email_prompt)
                    .result_title("Your Jira email");
                if has_jira_email {
                    field = field.allow_empty(true);
                    if let Some(ref email) = existing.jira_email {
                        field = field.default(email.clone());
                    }
                } else {
                    field = field.required();
                }
                let has_jira_email_clone = has_jira_email;
                field = field.validator(Arc::new(move |input: &str| {
                    if input.is_empty() && !has_jira_email_clone {
                        Err("Jira email address is required".to_string())
                    } else if !input.is_empty() && !input.contains('@') {
                        Err("Please enter a valid email address".to_string())
                    } else {
                        Ok(())
                    }
                }));
                s.add_input(field)
            })
            .add_step(|s| {
                // Jira service address
                let jira_address_prompt = if has_jira_address {
                    "Please enter your Jira service address (press Enter to keep)"
                } else {
                    "Please enter your Jira service address (required)"
                };
                let mut field = InputFormField::new("jira_service_address", jira_address_prompt)
                    .result_title("Your Jira service address");
                if has_jira_address {
                    field = field.allow_empty(true);
                    if let Some(ref addr) = existing.jira_service_address {
                        field = field.default(addr.clone());
                    }
                } else {
                    field = field.required();
                }
                let has_jira_address_clone = has_jira_address;
                field = field.validator(Arc::new(move |input: &str| {
                    if input.is_empty() && !has_jira_address_clone {
                        Err("Jira service address is required".to_string())
                    } else if !input.is_empty()
                        && !input.starts_with("http://")
                        && !input.starts_with("https://")
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
            .add_step(|s| {
                // Jira API token
                let jira_token_prompt = if has_jira_token {
                    "Please enter your Jira API token [current: ***] (press Enter to keep)"
                } else {
                    "Please enter your Jira API token (required)"
                };
                let mut field = InputFormField::new("jira_api_token", jira_token_prompt)
                    .result_title("Your Jira API token");
                if has_jira_token {
                    field = field.allow_empty(true);
                } else {
                    field = field.required();
                }
                let has_jira_token_clone = has_jira_token;
                field = field.validator(Arc::new(move |input: &str| {
                    if input.is_empty() && !has_jira_token_clone {
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
