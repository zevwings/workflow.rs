//! 初始化设置命令
//! 交互式配置应用，保存到 TOML 配置文件（~/.workflow/config/workflow.toml）

use crate::base::constants::messages::log;
use crate::base::dialog::{FormBuilder, GroupConfig, SelectDialog};
use crate::base::indicator::Spinner;
use crate::base::llm::{get_supported_language_display_names, SUPPORTED_LANGUAGES};
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::{
    default_download_base_dir, GitHubAccount, GitHubSettings, JiraSettings, LLMSettings,
    LogSettings, Settings,
};
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
    log_output_folder_name: Option<String>,
    log_download_base_dir: Option<String>,
    enable_trace_console: Option<bool>,
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
            log_output_folder_name: settings.log.output_folder_name.clone(),
            log_download_base_dir: settings.log.download_base_dir.clone(),
            enable_trace_console: settings.log.enable_trace_console,
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

        // 准备现有值
        let has_jira_email = existing.jira_email.is_some();
        let has_jira_address = existing.jira_service_address.is_some();
        let has_jira_token = existing.jira_api_token.is_some();
        let default_folder_name = LogSettings::default_log_folder();
        let is_custom_folder_name = existing
            .log_output_folder_name
            .as_ref()
            .map(|name| name != &default_folder_name)
            .unwrap_or(false);
        let default_dir = default_download_base_dir();
        let is_custom_dir = existing
            .log_download_base_dir
            .as_ref()
            .map(|dir| dir != &default_dir)
            .unwrap_or(false);
        let current_trace_console = existing.enable_trace_console.unwrap_or(false);

        // 使用 FormBuilder 收集所有配置
        let form_result = FormBuilder::new()
            // Group 1: Jira Configuration (必填组)
            .add_group(
                "jira",
                |g| {
                    g.step(|f| {
                        // Jira email
                        let jira_email_prompt = if has_jira_email {
                            "Jira email address (press Enter to keep)"
                        } else {
                            "Jira email address (required)"
                        };
                        let mut field = f.add_text("jira_email", jira_email_prompt);
                        if has_jira_email {
                            field = field.allow_empty(true);
                            if let Some(ref email) = existing.jira_email {
                                field = field.default(email.clone());
                            }
                        } else {
                            field = field.required();
                        }
                        field.validate(move |input: &str| {
                            if input.is_empty() && !has_jira_email {
                                Err("Jira email address is required".to_string())
                            } else if !input.is_empty() && !input.contains('@') {
                                Err("Please enter a valid email address".to_string())
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .step(|f| {
                        // Jira service address
                        let jira_address_prompt = if has_jira_address {
                            "Jira service address (press Enter to keep)"
                        } else {
                            "Jira service address (required)"
                        };
                        let mut field = f.add_text("jira_service_address", jira_address_prompt);
                        if has_jira_address {
                            field = field.allow_empty(true);
                            if let Some(ref addr) = existing.jira_service_address {
                                field = field.default(addr.clone());
                            }
                        } else {
                            field = field.required();
                        }
                        field.validate(move |input: &str| {
                            if input.is_empty() && !has_jira_address {
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
                        })
                    })
                    .step(|f| {
                        // Jira API token
                        let jira_token_prompt = if has_jira_token {
                            "Jira API token [current: ***] (press Enter to keep)"
                        } else {
                            "Jira API token (required)"
                        };
                        let mut field = f.add_text("jira_api_token", jira_token_prompt);
                        if has_jira_token {
                            field = field.allow_empty(true);
                        } else {
                            field = field.required();
                        }
                        field.validate(move |input: &str| {
                            if input.is_empty() && !has_jira_token {
                                Err("Jira API token is required".to_string())
                            } else {
                                Ok(())
                            }
                        })
                    })
                },
                GroupConfig::required().with_title("Jira Configuration (Required)"),
            )
            // Group 2: Log Configuration (可选组)
            .add_group(
                "log",
                |g| {
                    g.step(|f| {
                        f.add_confirmation(
                            "should_configure_log_folder",
                            "Do you want to configure log output folder name?",
                        )
                        .default(is_custom_folder_name)
                    })
                    .step_if("should_configure_log_folder", "yes", |f| {
                        let folder_name_prompt = if is_custom_folder_name {
                            "Log output folder name (press Enter to keep)".to_string()
                        } else {
                            format!(
                                "Log output folder name (press Enter to use default: {})",
                                default_folder_name
                            )
                        };
                        let mut field = f
                            .add_text("log_output_folder_name", &folder_name_prompt)
                            .allow_empty(true);
                        if is_custom_folder_name {
                            if let Some(ref existing_name) = existing.log_output_folder_name {
                                field = field.default(existing_name.clone());
                            }
                        } else {
                            field = field.default(default_folder_name.clone());
                        }
                        field
                    })
                    .step(|f| {
                        f.add_confirmation(
                            "should_configure_doc_dir",
                            "Do you want to configure document base directory?",
                        )
                        .default(is_custom_dir)
                    })
                    .step_if("should_configure_doc_dir", "yes", |f| {
                        let base_dir_prompt = if is_custom_dir {
                            "Document base directory (press Enter to keep)".to_string()
                        } else {
                            format!(
                                "Document base directory (press Enter to use default: {})",
                                default_dir
                            )
                        };
                        let mut field =
                            f.add_text("log_download_base_dir", &base_dir_prompt).allow_empty(true);
                        if is_custom_dir {
                            if let Some(ref existing_dir) = existing.log_download_base_dir {
                                field = field.default(existing_dir.clone());
                            }
                        } else {
                            field = field.default(default_dir.clone());
                        }
                        field
                    })
                    .step(|f| {
                        // Tracing Console Output
                        let trace_console_options = vec![
                            "Enable (output to both file and console)".to_string(),
                            "Disable (output to file only)".to_string(),
                        ];
                        let default_option = if current_trace_console {
                            trace_console_options[0].clone()
                        } else {
                            trace_console_options[1].clone()
                        };
                        f.add_selection(
                            "trace_console_mode",
                            "Select trace console output mode",
                            trace_console_options,
                        )
                        .default(default_option)
                    })
                },
                GroupConfig::optional().with_title("Log Configuration (Optional)"),
            )
            // Group 3: LLM Configuration (可选组)
            .add_group(
                "llm",
                |g| {
                    let llm_providers = vec![
                        "openai".to_string(),
                        "deepseek".to_string(),
                        "proxy".to_string(),
                    ];
                    let llm_provider_prompt =
                        format!("Select LLM provider [current: {}]", existing.llm_provider);

                    // OpenAI 配置字段
                    let openai_key_prompt = if existing.llm_openai_key.is_some() {
                        "OpenAI API key [current: ***] (press Enter to keep)"
                    } else {
                        "OpenAI API key (optional, press Enter to skip)"
                    };
                    let openai_model_default = existing
                        .llm_openai_model
                        .clone()
                        .unwrap_or_else(|| LLMSettings::default_model("openai"));
                    let openai_model_prompt = if existing.llm_openai_model.is_some() {
                        "OpenAI model (press Enter to keep)"
                    } else {
                        "OpenAI model (optional, press Enter to skip)"
                    };

                    // DeepSeek 配置字段
                    let deepseek_key_prompt = if existing.llm_deepseek_key.is_some() {
                        "DeepSeek API key [current: ***] (press Enter to keep)"
                    } else {
                        "DeepSeek API key (optional, press Enter to skip)"
                    };
                    let deepseek_model_default = existing
                        .llm_deepseek_model
                        .clone()
                        .unwrap_or_else(|| LLMSettings::default_model("deepseek"));
                    let deepseek_model_prompt = if existing.llm_deepseek_model.is_some() {
                        "DeepSeek model (press Enter to keep)"
                    } else {
                        "DeepSeek model (optional, press Enter to skip)"
                    };

                    // Proxy 配置字段
                    let proxy_url_prompt = if existing.llm_proxy_url.is_some() {
                        "LLM proxy URL (required) (press Enter to keep)"
                    } else {
                        "LLM proxy URL (required)"
                    };
                    let proxy_key_prompt = if existing.llm_proxy_key.is_some() {
                        "LLM proxy key [current: ***] (press Enter to keep)"
                    } else {
                        "LLM proxy key (required)"
                    };
                    let proxy_model_prompt = if existing.llm_proxy_model.is_some() {
                        "LLM model (press Enter to keep)"
                    } else {
                        "LLM model (required)"
                    };

                    let has_existing_proxy_url = existing.llm_proxy_url.is_some();
                    let has_existing_proxy_key = existing.llm_proxy_key.is_some();
                    let has_existing_proxy_model = existing.llm_proxy_model.is_some();

                    g.step(|f| {
                        f.add_selection("llm_provider", &llm_provider_prompt, llm_providers)
                            .default(existing.llm_provider.clone())
                    })
                    .step_if("llm_provider", "openai", |f| {
                        let mut form =
                            f.add_text("llm_openai_key", openai_key_prompt).allow_empty(true);
                        if let Some(ref key) = existing.llm_openai_key {
                            form = form.default(key.clone());
                        }
                        form.add_text("llm_openai_model", openai_model_prompt)
                            .allow_empty(true)
                            .default(openai_model_default)
                    })
                    .step_if("llm_provider", "deepseek", |f| {
                        let mut form =
                            f.add_text("llm_deepseek_key", deepseek_key_prompt).allow_empty(true);
                        if let Some(ref key) = existing.llm_deepseek_key {
                            form = form.default(key.clone());
                        }
                        form.add_text("llm_deepseek_model", deepseek_model_prompt)
                            .allow_empty(true)
                            .default(deepseek_model_default)
                    })
                    .step_if("llm_provider", "proxy", |f| {
                        let mut form = f.add_text("llm_proxy_url", proxy_url_prompt);
                        if has_existing_proxy_url {
                            form = form.allow_empty(true);
                            if let Some(ref url) = existing.llm_proxy_url {
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

                        let mut form = form.add_text("llm_proxy_key", proxy_key_prompt);
                        if has_existing_proxy_key {
                            form = form.allow_empty(true);
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

                        let mut form = form.add_text("llm_proxy_model", proxy_model_prompt);
                        if has_existing_proxy_model {
                            form = form.allow_empty(true);
                            if let Some(ref model) = existing.llm_proxy_model {
                                form = form.default(model.clone());
                            }
                        } else {
                            form = form.required();
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
                GroupConfig::optional().with_title("LLM/AI Configuration (Optional)"),
            )
            .run()
            .wrap_err("Failed to collect configuration")?;

        // 处理结果：Jira 配置
        let jira_email = if let Some(email) = form_result.get("jira_email") {
            if !email.is_empty() {
                Some(email.clone())
            } else if has_jira_email {
                existing.jira_email.clone()
            } else {
                color_eyre::eyre::bail!("Jira email address is required");
            }
        } else {
            color_eyre::eyre::bail!("Jira email address is required");
        };

        let jira_service_address = if let Some(address) = form_result.get("jira_service_address") {
            if !address.is_empty() {
                Some(address.clone())
            } else if has_jira_address {
                existing.jira_service_address.clone()
            } else {
                color_eyre::eyre::bail!("Jira service address is required");
            }
        } else {
            color_eyre::eyre::bail!("Jira service address is required");
        };

        let jira_api_token = if let Some(token) = form_result.get("jira_api_token") {
            if !token.is_empty() {
                Some(token.clone())
            } else if has_jira_token {
                existing.jira_api_token.clone()
            } else {
                color_eyre::eyre::bail!("Jira API token is required");
            }
        } else {
            color_eyre::eyre::bail!("Jira API token is required");
        };

        // 处理结果：Log 配置（包含 Tracing）
        // 如果用户选择不配置 Log 组，使用现有值
        let (log_output_folder_name, log_download_base_dir, enable_trace_console) = if form_result
            .has("should_configure_log_folder")
            || form_result.has("should_configure_doc_dir")
            || form_result.has("trace_console_mode")
        {
            // 用户配置了 Log 组，处理配置
            let log_output_folder_name =
                if form_result.get("should_configure_log_folder") == Some(&"yes".to_string()) {
                    if let Some(input_value) = form_result.get("log_output_folder_name") {
                        if input_value.is_empty() || input_value == &default_folder_name {
                            None
                        } else {
                            Some(input_value.clone())
                        }
                    } else {
                        None
                    }
                } else if is_custom_folder_name {
                    existing.log_output_folder_name.clone()
                } else {
                    None
                };

            let log_download_base_dir =
                if form_result.get("should_configure_doc_dir") == Some(&"yes".to_string()) {
                    if let Some(input_value) = form_result.get("log_download_base_dir") {
                        if input_value.is_empty() || input_value == &default_dir {
                            None
                        } else {
                            Some(input_value.clone())
                        }
                    } else {
                        None
                    }
                } else if is_custom_dir {
                    existing.log_download_base_dir.clone()
                } else {
                    None
                };

            // Tracing 配置
            let enable_trace_console = if let Some(mode) = form_result.get("trace_console_mode") {
                if mode == "Enable (output to both file and console)" {
                    Some(true)
                } else {
                    None
                }
            } else {
                existing.enable_trace_console
            };

            (
                log_output_folder_name,
                log_download_base_dir,
                enable_trace_console,
            )
        } else {
            // 用户选择不配置 Log 组，使用现有值
            (
                existing.log_output_folder_name.clone(),
                existing.log_download_base_dir.clone(),
                existing.enable_trace_console,
            )
        };

        // 处理结果：LLM 配置
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

        Ok(CollectedConfig {
            jira_email,
            jira_api_token,
            jira_service_address,
            github_accounts,
            github_current,
            log_output_folder_name,
            log_download_base_dir,
            enable_trace_console,
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
                output_folder_name: config.log_output_folder_name.clone(),
                download_base_dir: config.log_download_base_dir.clone(),
                level: None, // 日志级别通过 workflow log set 命令设置
                enable_trace_console: config.enable_trace_console,
            },
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
