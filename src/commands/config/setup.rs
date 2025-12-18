//! 初始化设置命令
//! 交互式配置应用，保存到 TOML 配置文件（~/.workflow/config/workflow.toml）

use crate::base::constants::messages::log;
use crate::base::dialog::{InputDialog, SelectDialog};
use crate::base::indicator::Spinner;
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::{
    default_download_base_dir, // CodeupSettings,  // Codeup support has been removed
    GitHubAccount,
    GitHubSettings,
    JiraSettings,
    LLMSettings,
    LogSettings,
    Settings,
};
// use crate::base::dialog::ConfirmDialog;  // Unused after Codeup removal
use crate::commands::config::helpers::select_language;
use crate::commands::github::helpers::collect_github_account;
use crate::git::GitConfig;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use std::collections::HashMap;

/// 初始化设置命令
pub struct SetupCommand;

/// 收集的配置数据
#[derive(Debug, Clone)]
struct CollectedConfig {
    // Workflow 配置
    jira_email: Option<String>,
    jira_api_token: Option<String>,
    jira_service_address: Option<String>,
    github_accounts: Vec<GitHubAccount>,
    github_current: Option<String>,
    log_download_base_dir: Option<String>,
    enable_trace_console: Option<bool>,
    // codeup_project_id: Option<u64>,  // Codeup support has been removed
    // codeup_csrf_token: Option<String>,  // Codeup support has been removed
    // codeup_cookie: Option<String>,  // Codeup support has been removed
    // LLM 配置
    llm_provider: String,
    llm_language: String, // LLM 输出语言（所有 provider 共享）
    // 各 provider 的配置
    llm_openai_key: Option<String>,
    llm_openai_model: Option<String>,
    llm_deepseek_key: Option<String>,
    llm_deepseek_model: Option<String>,
    llm_proxy_url: Option<String>,
    llm_proxy_key: Option<String>,
    llm_proxy_model: Option<String>,
}

impl SetupCommand {
    /// 运行初始化设置流程
    pub fn run() -> Result<()> {
        log_success!("Starting Workflow CLI initialization...\n");

        // 加载现有配置（从 TOML 文件）
        let existing_config = Self::load_existing_config()?;

        // 收集配置信息（智能处理现有配置）
        let config = Self::collect_config(&existing_config)?;

        // 保存配置到 TOML 文件
        log_message!("Saving configuration...");
        Self::save_config(&config)?;
        if let Ok(config_path) = crate::base::Paths::workflow_config() {
            log_success!("{} {}", log::CONFIG_SAVED_PREFIX, config_path.display());
        } else {
            log_success!(
                "{} ~/.workflow/config/workflow.toml",
                log::CONFIG_SAVED_PREFIX
            );
        }

        log_break!();
        log_info!("Verifying configuration...");
        log_break!();

        log_break!('-', 40, "Verifying Configuration");
        log_break!();

        // 检查配置文件权限
        if let Some(warning) = Settings::check_permissions() {
            log_warning!("{}", warning);
        }

        // 创建 spinner 显示验证进度
        let spinner = Spinner::new("Verifying configurations...");

        // 验证配置（使用 load() 获取最新配置，避免 OnceLock 缓存问题）
        let settings = Settings::load();
        let result = settings.verify()?;

        // 完成 spinner
        spinner.finish();

        crate::commands::config::show::ConfigCommand::print_verification_result(&result);

        log_break!();
        log_success!("Initialization completed successfully!");
        log_break!();
        log_message!("You can now use the Workflow CLI commands.");

        Ok(())
    }

    /// 加载现有配置（从 TOML 文件）
    fn load_existing_config() -> Result<CollectedConfig> {
        let settings = Settings::get();
        let llm = &settings.llm;

        Ok(CollectedConfig {
            jira_email: settings.jira.email.clone(),
            jira_api_token: settings.jira.api_token.clone(),
            jira_service_address: settings.jira.service_address.clone(),
            github_accounts: settings.github.accounts.clone(),
            github_current: settings.github.current.clone(),
            log_download_base_dir: settings.log.download_base_dir.clone(),
            enable_trace_console: settings.log.enable_trace_console,
            // codeup_project_id: settings.codeup.project_id,  // Codeup support has been removed
            // codeup_csrf_token: settings.codeup.csrf_token.clone(),  // Codeup support has been removed
            // codeup_cookie: settings.codeup.cookie.clone(),  // Codeup support has been removed
            llm_provider: llm.provider.clone(),
            llm_language: if llm.language.is_empty() {
                LLMSettings::default_language()
            } else {
                llm.language.clone()
            },
            llm_openai_key: llm.openai.key.clone(),
            llm_openai_model: llm.openai.model.clone(),
            llm_deepseek_key: llm.deepseek.key.clone(),
            llm_deepseek_model: llm.deepseek.model.clone(),
            llm_proxy_url: llm.proxy.url.clone(),
            llm_proxy_key: llm.proxy.key.clone(),
            llm_proxy_model: llm.proxy.model.clone(),
        })
    }

    /// 收集配置信息
    fn collect_config(existing: &CollectedConfig) -> Result<CollectedConfig> {
        // ==================== 必填项：GitHub 配置 ====================
        log_break!();
        log_message!("  GitHub Configuration (Required)");
        log_break!('─', 65);

        let mut github_accounts = existing.github_accounts.clone();
        let mut github_current = existing.github_current.clone();

        // 如果已有账号，询问是否要管理账号
        if !github_accounts.is_empty() {
            // 获取当前账号的 email，用于显示
            let current_email = github_current
                .as_ref()
                .and_then(|current_name| {
                    github_accounts
                        .iter()
                        .find(|a| &a.name == current_name)
                        .map(|a| a.email.clone())
                })
                .unwrap_or_else(|| "unknown".to_string());

            let keep_option = format!("Keep current accounts ({})", current_email);
            let options = vec!["Add new account".to_string(), keep_option];
            let selected_option = SelectDialog::new("GitHub account management", options.clone())
                .with_default(1)
                .prompt()
                .wrap_err("Failed to get GitHub account management choice")?;
            let selection = options.iter().position(|opt| opt == &selected_option).unwrap_or(1);

            let mut account_added = false;
            match selection {
                0 => {
                    // 添加新账号
                    let account = collect_github_account()?;
                    github_accounts.push(account);
                    account_added = true;
                    // 如果是第一个账号，自动设置为当前账号
                    if github_accounts.len() == 1 {
                        let first_account = github_accounts
                            .first()
                            .ok_or_else(|| eyre!("Expected at least one GitHub account"))?;
                        github_current = Some(first_account.name.clone());
                        let _ =
                            GitConfig::set_global_user(&first_account.email, &first_account.name)?;
                    }
                }
                _ => {
                    // 保持现有账号，但需要确保 Git 配置与当前账号一致
                    if let Some(ref current_name) = github_current {
                        if let Some(current_account) =
                            github_accounts.iter().find(|a| &a.name == current_name)
                        {
                            let _ = GitConfig::set_global_user(
                                &current_account.email,
                                &current_account.name,
                            )?;
                        }
                    }
                }
            }

            // 只有在添加了新账号后，如果有多个账号，才询问选择当前账号
            if account_added && github_accounts.len() > 1 {
                let account_names: Vec<String> =
                    github_accounts.iter().map(|a| a.name.clone()).collect();
                let default_index = github_current
                    .as_ref()
                    .and_then(|current| account_names.iter().position(|n| n == current))
                    .unwrap_or(0);

                let account_names_vec: Vec<String> = account_names.to_vec();
                let selected_account =
                    SelectDialog::new("Select current GitHub account", account_names_vec.clone())
                        .with_default(default_index)
                        .prompt()
                        .wrap_err("Failed to select current account")?;
                let selection = account_names_vec
                    .iter()
                    .position(|name| name == &selected_account)
                    .unwrap_or(default_index);

                github_current = Some(account_names[selection].clone());
                let current_account = &github_accounts[selection];
                let _ = GitConfig::set_global_user(&current_account.email, &current_account.name)?;
            } else if github_accounts.len() == 1 {
                // 如果只有一个账号，确保设置了 Git 配置
                let account = &github_accounts[0];
                if github_current.as_ref().map(|c| c == &account.name).unwrap_or(false) {
                    let _ = GitConfig::set_global_user(&account.email, &account.name)?;
                }
            }
        } else {
            // 没有账号，添加第一个账号
            log_message!("No GitHub accounts configured. Let's add one:");
            let account = collect_github_account()?;
            github_accounts.push(account);
            let first_account = github_accounts
                .first()
                .ok_or_else(|| eyre!("Expected at least one GitHub account"))?;
            github_current = Some(first_account.name.clone());
            let _ = GitConfig::set_global_user(&first_account.email, &first_account.name)?;
        }

        // ==================== 必填项：Jira 配置 ====================
        log_break!();
        log_message!("  Jira Configuration (Required)");
        log_break!('─', 65);

        let has_jira_email = existing.jira_email.is_some();
        let jira_email_prompt = if existing.jira_email.is_some() {
            "Jira email address (press Enter to keep)".to_string()
        } else {
            "Jira email address (required)".to_string()
        };

        let jira_email = if let Some(email) = &existing.jira_email {
            InputDialog::new(&jira_email_prompt).with_default(email.clone())
        } else {
            InputDialog::new(&jira_email_prompt)
        }
        .with_validator(move |input: &str| {
            if input.is_empty() && !has_jira_email {
                Err("Jira email address is required".to_string())
            } else if !input.is_empty() && !input.contains('@') {
                Err("Please enter a valid email address".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get Jira email address")?;

        let jira_email = if !jira_email.is_empty() {
            Some(jira_email)
        } else if existing.jira_email.is_some() {
            existing.jira_email.clone()
        } else {
            color_eyre::eyre::bail!("Jira email address is required");
        };

        let has_jira_address = existing.jira_service_address.is_some();
        let jira_address_prompt = if existing.jira_service_address.is_some() {
            "Jira service address (press Enter to keep)".to_string()
        } else {
            "Jira service address (required)".to_string()
        };

        let jira_service_address = if let Some(addr) = &existing.jira_service_address {
            InputDialog::new(&jira_address_prompt).with_default(addr.clone())
        } else {
            InputDialog::new(&jira_address_prompt)
        }
        .with_validator(move |input: &str| {
            if input.is_empty() && !has_jira_address {
                Err("Jira service address is required".to_string())
            } else if !input.is_empty()
                && !input.starts_with("http://")
                && !input.starts_with("https://")
            {
                Err("Please enter a valid URL (must start with http:// or https://)".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .wrap_err("Failed to get Jira service address")?;

        let jira_service_address = if !jira_service_address.is_empty() {
            Some(jira_service_address)
        } else if existing.jira_service_address.is_some() {
            existing.jira_service_address.clone()
        } else {
            color_eyre::eyre::bail!("Jira service address is required");
        };

        let jira_token_prompt = if existing.jira_api_token.is_some() {
            "Jira API token [current: ***] (press Enter to keep)".to_string()
        } else {
            "Jira API token (required)".to_string()
        };

        let jira_api_token = InputDialog::new(&jira_token_prompt)
            .allow_empty(existing.jira_api_token.is_some())
            .prompt()
            .wrap_err("Failed to get Jira API token")?;

        let jira_api_token = if !jira_api_token.is_empty() {
            Some(jira_api_token)
        } else if existing.jira_api_token.is_some() {
            existing.jira_api_token.clone()
        } else {
            color_eyre::eyre::bail!("Jira API token is required");
        };

        // ==================== 可选：文档基础路径配置 ====================
        log_break!();
        log_message!("  Document Base Directory (Optional)");
        log_break!('─', 65);

        // 检查是否是用户自定义的值（不等于默认值）
        // 由于 serde default，即使配置文件中没有这个字段，也会使用默认值
        // 所以需要检查值是否等于默认值来判断是否是用户配置的
        let default_dir = default_download_base_dir();
        let is_custom_dir = existing
            .log_download_base_dir
            .as_ref()
            .map(|dir| dir != &default_dir)
            .unwrap_or(false);

        // 设置文档基础目录
        let base_dir_prompt = if is_custom_dir {
            "Document base directory (press Enter to keep)".to_string()
        } else {
            "Document base directory (press Enter to use default)".to_string()
        };

        // 只有当用户之前设置过自定义值时，才显示该值；否则显示默认值作为提示
        let mut dialog = InputDialog::new(&base_dir_prompt).allow_empty(true);
        if is_custom_dir {
            // 用户自定义的值，作为默认值显示
            if let Some(ref existing_dir) = existing.log_download_base_dir {
                dialog = dialog.with_default(existing_dir.clone());
            }
        }

        let log_download_base_dir =
            dialog.prompt().wrap_err("Failed to get document base directory")?;

        let log_download_base_dir = if log_download_base_dir.is_empty() {
            // 如果为空，使用默认值（但不在配置文件中保存，使用 None）
            None
        } else if log_download_base_dir == default_dir {
            // 如果等于默认值，也不保存（使用 None）
            None
        } else {
            // 自定义路径，保存到配置文件
            Some(log_download_base_dir)
        };

        // ==================== 可选：Tracing 控制台输出配置 ====================
        log_break!();
        log_message!("  Tracing Console Output (Optional)");
        log_break!('─', 65);

        let current_trace_console = existing.enable_trace_console.unwrap_or(false);
        let current_status = if current_trace_console {
            "enabled (output to both file and console)"
        } else {
            "disabled (output to file only)"
        };

        log_message!("Current: {}", current_status);
        log_message!(
            "Enable tracing console output? (tracing logs will be output to both file and console)"
        );

        let trace_console_options = vec![
            "Enable (output to both file and console)",
            "Disable (output to file only)",
        ];

        let current_trace_console_idx = if current_trace_console { 0 } else { 1 };

        let selected_trace_console = SelectDialog::new(
            "Select trace console output mode",
            trace_console_options.clone(),
        )
        .with_default(current_trace_console_idx)
        .prompt()
        .wrap_err("Failed to select trace console option")?;

        // true 时写入配置文件，false 时从配置文件中删除（设置为 None）
        let enable_trace_console = trace_console_options
            .iter()
            .position(|&opt| opt == selected_trace_console)
            .map(|idx| if idx == 0 { Some(true) } else { None })
            .unwrap_or(None);

        // ==================== 可选：LLM/AI 配置 ====================
        log_break!();
        log_message!("  LLM/AI Configuration (Optional)");
        log_break!('─', 65);

        let llm_providers = ["openai", "deepseek", "proxy"];
        let current_provider_idx = llm_providers
            .iter()
            .position(|&p| p == existing.llm_provider.as_str())
            .unwrap_or(0);

        let llm_provider_prompt =
            format!("Select LLM provider [current: {}]", existing.llm_provider);

        let llm_providers_vec: Vec<String> = llm_providers.iter().map(|s| s.to_string()).collect();
        let llm_provider = SelectDialog::new(&llm_provider_prompt, llm_providers_vec)
            .with_default(current_provider_idx)
            .prompt()
            .wrap_err("Failed to select LLM provider")?;

        // 初始化各 provider 的配置（从 existing 加载，保持其他 provider 的配置不变）
        let mut llm_openai_key = existing.llm_openai_key.clone();
        let mut llm_openai_model = existing.llm_openai_model.clone();
        let mut llm_deepseek_key = existing.llm_deepseek_key.clone();
        let mut llm_deepseek_model = existing.llm_deepseek_model.clone();
        let mut llm_proxy_url = existing.llm_proxy_url.clone();
        let mut llm_proxy_key = existing.llm_proxy_key.clone();
        let mut llm_proxy_model = existing.llm_proxy_model.clone();

        // 根据选择的 provider 配置对应的设置
        match llm_provider.as_str() {
            "openai" => {
                // 配置 OpenAI API key
                let key_prompt = if llm_openai_key.is_some() {
                    "OpenAI API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "OpenAI API key (optional, press Enter to skip)".to_string()
                };

                let llm_key_input = InputDialog::new(&key_prompt)
                    .allow_empty(true)
                    .prompt()
                    .wrap_err("Failed to get OpenAI API key")?;

                if !llm_key_input.is_empty() {
                    llm_openai_key = Some(llm_key_input);
                }

                // 配置 OpenAI model
                let default_model = llm_openai_model
                    .clone()
                    .unwrap_or_else(|| LLMSettings::default_model("openai"));

                let model_prompt = if llm_openai_model.is_some() {
                    "OpenAI model (press Enter to keep)".to_string()
                } else {
                    "OpenAI model (optional, press Enter to skip)".to_string()
                };

                let llm_model_input = InputDialog::new(&model_prompt)
                    .allow_empty(true)
                    .with_default(default_model.clone())
                    .prompt()
                    .wrap_err("Failed to get OpenAI model")?;

                if !llm_model_input.is_empty() {
                    llm_openai_model = Some(llm_model_input);
                } else if llm_openai_model.is_none() {
                    // 如果用户没有输入且之前也没有值，设置为 None（使用默认值）
                    llm_openai_model = None;
                }
            }
            "deepseek" => {
                // 配置 DeepSeek API key
                let key_prompt = if llm_deepseek_key.is_some() {
                    "DeepSeek API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "DeepSeek API key (optional, press Enter to skip)".to_string()
                };

                let llm_key_input = InputDialog::new(&key_prompt)
                    .allow_empty(true)
                    .prompt()
                    .wrap_err("Failed to get DeepSeek API key")?;

                if !llm_key_input.is_empty() {
                    llm_deepseek_key = Some(llm_key_input);
                }

                // 配置 DeepSeek model
                let default_model = llm_deepseek_model
                    .clone()
                    .unwrap_or_else(|| LLMSettings::default_model("deepseek"));

                let model_prompt = if llm_deepseek_model.is_some() {
                    "DeepSeek model (press Enter to keep)".to_string()
                } else {
                    "DeepSeek model (optional, press Enter to skip)".to_string()
                };

                let llm_model_input = InputDialog::new(&model_prompt)
                    .allow_empty(true)
                    .with_default(default_model.clone())
                    .prompt()
                    .wrap_err("Failed to get DeepSeek model")?;

                if !llm_model_input.is_empty() {
                    llm_deepseek_model = Some(llm_model_input);
                } else if llm_deepseek_model.is_none() {
                    llm_deepseek_model = None;
                }
            }
            "proxy" => {
                // 配置 Proxy URL（必填）
                let llm_url_prompt = if llm_proxy_url.is_some() {
                    "LLM proxy URL (required) (press Enter to keep)".to_string()
                } else {
                    "LLM proxy URL (required)".to_string()
                };

                let has_existing_url = llm_proxy_url.is_some();
                let existing_url = llm_proxy_url.clone();

                let llm_url_input = {
                    let mut dialog = InputDialog::new(&llm_url_prompt);

                    // 如果存在现有值，允许空输入（表示保留现有值）
                    // 如果不存在现有值，不允许空输入（必须输入）
                    dialog = dialog.allow_empty(has_existing_url);

                    // 如果存在现有值，设置为默认值
                    if let Some(ref url) = llm_proxy_url {
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

                if !llm_url_input.is_empty() {
                    llm_proxy_url = Some(llm_url_input);
                } else if has_existing_url {
                    // 用户按 Enter 保留现有值
                    llm_proxy_url = existing_url;
                } else {
                    color_eyre::eyre::bail!("LLM proxy URL is required");
                }

                // 配置 Proxy API key（必填）
                let key_prompt = if llm_proxy_key.is_some() {
                    "LLM proxy key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "LLM proxy key (required)".to_string()
                };

                let has_existing_key = llm_proxy_key.is_some();
                let existing_key = llm_proxy_key.clone();

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

                if !llm_key_input.is_empty() {
                    llm_proxy_key = Some(llm_key_input);
                } else if has_existing_key {
                    // 用户按 Enter 保留现有值
                    llm_proxy_key = existing_key;
                } else {
                    color_eyre::eyre::bail!("LLM proxy key is required");
                }

                // 配置 Proxy model（必填）
                let model_prompt = if llm_proxy_model.is_some() {
                    "LLM model (press Enter to keep)".to_string()
                } else {
                    "LLM model (required)".to_string()
                };

                let llm_model_input = if let Some(model) = &llm_proxy_model {
                    InputDialog::new(&model_prompt).allow_empty(false).with_default(model.clone())
                } else {
                    InputDialog::new(&model_prompt).allow_empty(false)
                }
                .with_validator(|input: &str| {
                    if input.is_empty() {
                        Err("Model is required for proxy provider".to_string())
                    } else {
                        Ok(())
                    }
                })
                .prompt()
                .wrap_err("Failed to get LLM model")?;

                if !llm_model_input.is_empty() {
                    llm_proxy_model = Some(llm_model_input);
                } else {
                    color_eyre::eyre::bail!("Model is required for proxy provider");
                }
            }
            _ => {}
        }

        // 配置 LLM 输出语言（所有 provider 共享）
        let current_language = if !existing.llm_language.is_empty() {
            Some(existing.llm_language.as_str())
        } else {
            None
        };

        let llm_language =
            select_language(current_language).wrap_err("Failed to select LLM output language")?;

        // Codeup 配置已移除（Codeup support has been removed）
        // ==================== 可选：Codeup 配置 ====================
        // ... (removed)

        Ok(CollectedConfig {
            jira_email,
            jira_api_token,
            jira_service_address,
            github_accounts,
            github_current,
            log_download_base_dir,
            enable_trace_console,
            // codeup_project_id,  // Codeup support has been removed
            // codeup_csrf_token,  // Codeup support has been removed
            // codeup_cookie,  // Codeup support has been removed
            llm_provider,
            llm_language,
            llm_openai_key,
            llm_openai_model,
            llm_deepseek_key,
            llm_deepseek_model,
            llm_proxy_url,
            llm_proxy_key,
            llm_proxy_model,
        })
    }

    /// 保存配置到 TOML 文件
    fn save_config(config: &CollectedConfig) -> Result<()> {
        // 构建 Settings 结构体
        let settings = Settings {
            aliases: HashMap::new(),
            jira: JiraSettings {
                email: config.jira_email.clone(),
                api_token: config.jira_api_token.clone(),
                service_address: config.jira_service_address.clone(),
            },
            github: GitHubSettings {
                accounts: config.github_accounts.clone(),
                current: config.github_current.clone(),
            },
            log: LogSettings {
                output_folder_name: LogSettings::default_log_folder(), // 使用默认值，不再允许用户配置
                download_base_dir: config.log_download_base_dir.clone(),
                level: None, // 日志级别通过 workflow log set 命令设置
                enable_trace_console: config.enable_trace_console,
            },
            // codeup: CodeupSettings {  // Codeup support has been removed
            //     project_id: config.codeup_project_id,
            //     csrf_token: config.codeup_csrf_token.clone(),
            //     cookie: config.codeup_cookie.clone(),
            // },
            llm: crate::base::settings::settings::LLMSettings {
                provider: config.llm_provider.clone(),
                language: config.llm_language.clone(),
                openai: crate::base::settings::settings::LLMProviderSettings {
                    url: None,
                    key: config.llm_openai_key.clone(),
                    model: config.llm_openai_model.clone(),
                },
                deepseek: crate::base::settings::settings::LLMProviderSettings {
                    url: None,
                    key: config.llm_deepseek_key.clone(),
                    model: config.llm_deepseek_model.clone(),
                },
                proxy: crate::base::settings::settings::LLMProviderSettings {
                    url: config.llm_proxy_url.clone(),
                    key: config.llm_proxy_key.clone(),
                    model: config.llm_proxy_model.clone(),
                },
            },
        };

        // 保存 workflow.toml
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);
        manager.write(&settings)?;

        Ok(())
    }
}
