//! 初始化设置命令
//! 交互式配置应用，保存到 TOML 配置文件（~/.workflow/config/workflow.toml）

use crate::base::settings::defaults::{
    default_download_base_dir, default_language, default_llm_model, default_log_folder,
};
use crate::base::settings::paths::Paths;
use crate::base::settings::settings::{GitHubAccount, Settings};
use crate::base::util::confirm;
use crate::commands::config::helpers::select_language;
use crate::commands::github::helpers::collect_github_account;
use crate::git::GitConfig;
use crate::jira::config::ConfigManager;
use crate::{log_break, log_info, log_message, log_success};
use anyhow::{Context, Result};
use dialoguer::{Input, Select};

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
    codeup_project_id: Option<u64>,
    codeup_csrf_token: Option<String>,
    codeup_cookie: Option<String>,
    // LLM 配置
    llm_provider: String,
    llm_url: Option<String>,
    llm_key: Option<String>,
    llm_model: Option<String>,
    llm_language: String, // LLM 输出语言
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
        log_success!("Configuration saved to ~/.workflow/config/workflow.toml");

        log_break!();
        log_info!("Verifying configuration...");
        log_break!();

        log_break!('-', 40, "Verifying Configuration");
        log_break!();

        // 验证配置（使用 load() 获取最新配置，避免 OnceLock 缓存问题）
        let settings = Settings::load();
        settings.verify()?;

        log_break!();
        log_success!("Initialization completed successfully!");
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
            codeup_project_id: settings.codeup.project_id,
            codeup_csrf_token: settings.codeup.csrf_token.clone(),
            codeup_cookie: settings.codeup.cookie.clone(),
            llm_provider: llm.provider.clone(),
            llm_url: llm.url.clone(),
            llm_key: llm.key.clone(),
            llm_model: llm.model.clone(),
            llm_language: if llm.language.is_empty() {
                default_language()
            } else {
                llm.language.clone()
            },
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
            let options = ["Add new account".to_string(), keep_option];
            let options_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
            let selection = Select::new()
                .with_prompt("GitHub account management")
                .items(&options_refs)
                .default(1)
                .interact()
                .context("Failed to get GitHub account management choice")?;

            let mut account_added = false;
            match selection {
                0 => {
                    // 添加新账号
                    let account = collect_github_account()?;
                    github_accounts.push(account);
                    account_added = true;
                    // 如果是第一个账号，自动设置为当前账号
                    if github_accounts.len() == 1 {
                        let first_account = github_accounts.first().unwrap();
                        github_current = Some(first_account.name.clone());
                        GitConfig::set_global_user(&first_account.email, &first_account.name)?;
                    }
                }
                _ => {
                    // 保持现有账号，但需要确保 Git 配置与当前账号一致
                    if let Some(ref current_name) = github_current {
                        if let Some(current_account) =
                            github_accounts.iter().find(|a| &a.name == current_name)
                        {
                            GitConfig::set_global_user(
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
                let account_names_display: Vec<&str> =
                    account_names.iter().map(|s| s.as_str()).collect();
                let default_index = github_current
                    .as_ref()
                    .and_then(|current| account_names.iter().position(|n| n == current))
                    .unwrap_or(0);

                let selection = Select::new()
                    .with_prompt("Select current GitHub account")
                    .items(&account_names_display)
                    .default(default_index)
                    .interact()
                    .context("Failed to select current account")?;

                github_current = Some(account_names[selection].clone());
                let current_account = &github_accounts[selection];
                GitConfig::set_global_user(&current_account.email, &current_account.name)?;
            } else if github_accounts.len() == 1 {
                // 如果只有一个账号，确保设置了 Git 配置
                let account = &github_accounts[0];
                if github_current
                    .as_ref()
                    .map(|c| c == &account.name)
                    .unwrap_or(false)
                {
                    GitConfig::set_global_user(&account.email, &account.name)?;
                }
            }
        } else {
            // 没有账号，添加第一个账号
            log_message!("No GitHub accounts configured. Let's add one:");
            let account = collect_github_account()?;
            github_accounts.push(account);
            let first_account = github_accounts.first().unwrap();
            github_current = Some(first_account.name.clone());
            GitConfig::set_global_user(&first_account.email, &first_account.name)?;
        }

        // ==================== 必填项：Jira 配置 ====================
        log_break!();
        log_message!("  Jira Configuration (Required)");
        log_break!('─', 65);

        let has_jira_email = existing.jira_email.is_some();
        let jira_email_prompt = if let Some(ref email) = existing.jira_email {
            format!("Jira email address [current: {}]", email)
        } else {
            "Jira email address".to_string()
        };

        let default_jira_email = existing.jira_email.clone().unwrap_or_default();

        let jira_email: String = Input::new()
            .with_prompt(&jira_email_prompt)
            .default(default_jira_email)
            .validate_with(move |input: &String| -> Result<(), &str> {
                if input.is_empty() && !has_jira_email {
                    Err("Jira email address is required")
                } else if !input.is_empty() && !input.contains('@') {
                    Err("Please enter a valid email address")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .context("Failed to get Jira email address")?;

        let jira_email = if !jira_email.is_empty() {
            Some(jira_email)
        } else if existing.jira_email.is_some() {
            existing.jira_email.clone()
        } else {
            anyhow::bail!("Jira email address is required");
        };

        let has_jira_address = existing.jira_service_address.is_some();
        let jira_address_prompt = if let Some(ref addr) = existing.jira_service_address {
            format!("Jira service address [current: {}]", addr)
        } else {
            "Jira service address".to_string()
        };

        let default_jira_address = existing
            .jira_service_address
            .clone()
            .unwrap_or_else(|| String::from(""));

        let jira_service_address: String = Input::new()
            .with_prompt(&jira_address_prompt)
            .default(default_jira_address)
            .validate_with(move |input: &String| -> Result<(), &str> {
                if input.is_empty() && !has_jira_address {
                    Err("Jira service address is required")
                } else if !input.is_empty()
                    && !input.starts_with("http://")
                    && !input.starts_with("https://")
                {
                    Err("Please enter a valid URL (must start with http:// or https://)")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .context("Failed to get Jira service address")?;

        let jira_service_address = if !jira_service_address.is_empty() {
            Some(jira_service_address)
        } else if existing.jira_service_address.is_some() {
            existing.jira_service_address.clone()
        } else {
            anyhow::bail!("Jira service address is required");
        };

        let jira_token_prompt = if existing.jira_api_token.is_some() {
            "Jira API token [current: ***]".to_string()
        } else {
            "Jira API token".to_string()
        };

        let jira_api_token: String = Input::new()
            .with_prompt(&jira_token_prompt)
            .allow_empty(existing.jira_api_token.is_some())
            .interact_text()
            .context("Failed to get Jira API token")?;

        let jira_api_token = if !jira_api_token.is_empty() {
            Some(jira_api_token)
        } else if existing.jira_api_token.is_some() {
            existing.jira_api_token.clone()
        } else {
            anyhow::bail!("Jira API token is required");
        };

        // ==================== 可选：文档基础路径配置 ====================
        log_break!();
        log_message!("  Document Base Directory (Optional)");
        log_break!('─', 65);

        // 设置文档基础目录
        let base_dir_prompt = if let Some(ref dir) = existing.log_download_base_dir {
            format!("Document base directory [current: {}]", dir)
        } else {
            format!(
                "Document base directory [default: {}] (press Enter to use default)",
                default_download_base_dir()
            )
        };

        let log_download_base_dir: String = Input::new()
            .with_prompt(&base_dir_prompt)
            .allow_empty(true)
            .default(
                existing
                    .log_download_base_dir
                    .clone()
                    .unwrap_or_else(default_download_base_dir),
            )
            .interact_text()
            .context("Failed to get document base directory")?;

        let log_download_base_dir = if log_download_base_dir.is_empty() {
            // 如果为空，使用默认值（但不在配置文件中保存，使用 None）
            None
        } else if log_download_base_dir == default_download_base_dir() {
            // 如果等于默认值，也不保存（使用 None）
            None
        } else {
            // 自定义路径，保存到配置文件
            Some(log_download_base_dir)
        };

        // ==================== 可选：LLM/AI 配置 ====================
        log_break!();
        log_message!("  LLM/AI Configuration (Optional)");
        log_break!('─', 65);

        let llm_providers = vec!["openai", "deepseek", "proxy"];
        let current_provider_idx = llm_providers
            .iter()
            .position(|&p| p == existing.llm_provider.as_str())
            .unwrap_or(0);

        let llm_provider_prompt =
            format!("Select LLM provider [current: {}]", existing.llm_provider);

        let llm_provider_idx = Select::new()
            .with_prompt(&llm_provider_prompt)
            .items(&llm_providers)
            .default(current_provider_idx)
            .interact()
            .context("Failed to select LLM provider")?;
        let llm_provider = llm_providers[llm_provider_idx].to_string();

        // 根据 provider 设置 URL（只有 proxy 需要输入和保存）
        // 对于 openai/deepseek，必须设置为 None，避免使用旧的 proxy URL 导致错误
        let llm_url = match llm_provider.as_str() {
            "openai" => None,   // openai 不使用 proxy URL，必须为 None
            "deepseek" => None, // deepseek 不使用 proxy URL，必须为 None
            "proxy" => {
                let llm_url_prompt = if let Some(ref url) = existing.llm_url {
                    format!("LLM proxy URL [current: {}] (press Enter to keep)", url)
                } else {
                    "LLM proxy URL (optional, press Enter to skip)".to_string()
                };

                let llm_url_input: String = Input::new()
                    .with_prompt(&llm_url_prompt)
                    .allow_empty(true)
                    .interact_text()
                    .context("Failed to get LLM proxy URL")?;

                if !llm_url_input.is_empty() {
                    Some(llm_url_input)
                } else {
                    existing.llm_url.clone()
                }
            }
            _ => None,
        };

        // 收集 API key
        let key_prompt = match llm_provider.as_str() {
            "openai" => {
                if existing.llm_key.is_some() {
                    "OpenAI API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "OpenAI API key (optional, press Enter to skip)".to_string()
                }
            }
            "deepseek" => {
                if existing.llm_key.is_some() {
                    "DeepSeek API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "DeepSeek API key (optional, press Enter to skip)".to_string()
                }
            }
            "proxy" => {
                if existing.llm_key.is_some() {
                    "LLM proxy key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "LLM proxy key (optional, press Enter to skip)".to_string()
                }
            }
            _ => "LLM API key (optional, press Enter to skip)".to_string(),
        };

        let llm_key_input: String = Input::new()
            .with_prompt(&key_prompt)
            .allow_empty(true)
            .interact_text()
            .context("Failed to get LLM API key")?;

        let llm_key = if !llm_key_input.is_empty() {
            Some(llm_key_input)
        } else {
            existing.llm_key.clone()
        };

        // 配置模型
        let default_model = existing
            .llm_model
            .clone()
            .unwrap_or_else(|| default_llm_model(&llm_provider));

        let model_prompt = match llm_provider.as_str() {
            "openai" => {
                if let Some(ref model) = existing.llm_model {
                    format!("OpenAI model [current: {}] (press Enter to keep)", model)
                } else {
                    "OpenAI model (optional, press Enter to skip)".to_string()
                }
            }
            "deepseek" => {
                if let Some(ref model) = existing.llm_model {
                    format!("DeepSeek model [current: {}] (press Enter to keep)", model)
                } else {
                    "DeepSeek model (optional, press Enter to skip)".to_string()
                }
            }
            "proxy" => {
                if let Some(ref model) = existing.llm_model {
                    format!("LLM model [current: {}] (required)", model)
                } else {
                    "LLM model (required)".to_string()
                }
            }
            _ => "LLM model".to_string(),
        };

        let is_proxy = llm_provider == "proxy";
        // 只有当之前有保存的值时，才设置默认值；否则不设置，让用户明确输入或留空使用默认值
        let has_existing_model = existing.llm_model.is_some();

        let llm_model_input: String = {
            let mut input = Input::new()
                .with_prompt(&model_prompt)
                .allow_empty(!is_proxy);

            // 只有之前有保存的值时，才设置默认值
            if has_existing_model {
                input = input.default(default_model.clone());
            }

            input
                .validate_with(move |input: &String| -> Result<(), &str> {
                    if input.is_empty() && is_proxy {
                        Err("Model is required for proxy provider")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()
                .context("Failed to get LLM model")?
        };

        let llm_model = if !llm_model_input.is_empty() {
            Some(llm_model_input)
        } else if is_proxy {
            anyhow::bail!("Model is required for proxy provider");
        } else {
            // 对于 openai 和 deepseek，如果为空则设置为 None
            // 这样不会保存到 TOML，运行时会在 build_model() 中使用默认值
            None
        };

        // 配置 LLM 输出语言
        let current_language = if !existing.llm_language.is_empty() {
            Some(existing.llm_language.as_str())
        } else {
            None
        };

        let llm_language =
            select_language(current_language).context("Failed to select LLM output language")?;

        // ==================== 可选：Codeup 配置 ====================
        log_break!();
        log_message!("📦 Codeup Configuration (Optional)");
        log_break!('─', 65);

        let has_codeup = existing.codeup_project_id.is_some()
            || existing.codeup_csrf_token.is_some()
            || existing.codeup_cookie.is_some();

        let codeup_confirm_prompt = if has_codeup {
            "Do you want to configure Codeup? [current: configured]".to_string()
        } else {
            "Do you use Codeup (Aliyun Code Repository)?".to_string()
        };

        let should_configure_codeup = confirm(&codeup_confirm_prompt, has_codeup, None)?;

        let (codeup_project_id, codeup_csrf_token, codeup_cookie) = if should_configure_codeup {
            let codeup_id_prompt = if let Some(ref id) = existing.codeup_project_id {
                format!("Codeup project ID [current: {}] (press Enter to keep)", id)
            } else {
                "Codeup project ID (optional, press Enter to skip)".to_string()
            };

            let default_codeup_id = existing
                .codeup_project_id
                .map(|id| id.to_string())
                .unwrap_or_default();

            let codeup_project_id: String = Input::new()
                .with_prompt(&codeup_id_prompt)
                .allow_empty(true)
                .default(default_codeup_id)
                .interact_text()
                .context("Failed to get Codeup project ID")?;

            let codeup_project_id = if !codeup_project_id.is_empty() {
                codeup_project_id.parse::<u64>().ok()
            } else {
                existing.codeup_project_id
            };

            let codeup_csrf_prompt = if existing.codeup_csrf_token.is_some() {
                "Codeup CSRF token [current: ***] (press Enter to keep)".to_string()
            } else {
                "Codeup CSRF token (optional, press Enter to skip)".to_string()
            };

            let codeup_csrf_token: String = Input::new()
                .with_prompt(&codeup_csrf_prompt)
                .allow_empty(true)
                .interact_text()
                .context("Failed to get Codeup CSRF token")?;

            let codeup_csrf_token = if !codeup_csrf_token.is_empty() {
                Some(codeup_csrf_token)
            } else {
                existing.codeup_csrf_token.clone()
            };

            let codeup_cookie_prompt = if existing.codeup_cookie.is_some() {
                "Codeup cookie [current: ***] (press Enter to keep)".to_string()
            } else {
                "Codeup cookie (optional, press Enter to skip)".to_string()
            };

            let codeup_cookie: String = Input::new()
                .with_prompt(&codeup_cookie_prompt)
                .allow_empty(true)
                .interact_text()
                .context("Failed to get Codeup cookie")?;

            let codeup_cookie = if !codeup_cookie.is_empty() {
                Some(codeup_cookie)
            } else {
                existing.codeup_cookie.clone()
            };

            (codeup_project_id, codeup_csrf_token, codeup_cookie)
        } else {
            (
                existing.codeup_project_id,
                existing.codeup_csrf_token.clone(),
                existing.codeup_cookie.clone(),
            )
        };

        Ok(CollectedConfig {
            jira_email,
            jira_api_token,
            jira_service_address,
            github_accounts,
            github_current,
            log_download_base_dir,
            codeup_project_id,
            codeup_csrf_token,
            codeup_cookie,
            llm_provider,
            llm_url,
            llm_key,
            llm_model,
            llm_language,
        })
    }

    /// 保存配置到 TOML 文件
    fn save_config(config: &CollectedConfig) -> Result<()> {
        use crate::base::settings::settings::{
            ApifoxSettings, CodeupSettings, GitHubSettings, JiraSettings, LogSettings, Settings,
        };

        // 构建 Settings 结构体
        let settings = Settings {
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
                output_folder_name: default_log_folder(), // 使用默认值，不再允许用户配置
                download_base_dir: config.log_download_base_dir.clone(),
                level: None, // 日志级别通过 workflow log set 命令设置
            },
            codeup: CodeupSettings {
                project_id: config.codeup_project_id,
                csrf_token: config.codeup_csrf_token.clone(),
                cookie: config.codeup_cookie.clone(),
            },
            llm: crate::base::settings::settings::LLMSettings {
                url: config.llm_url.clone(),
                key: config.llm_key.clone(),
                provider: config.llm_provider.clone(),
                model: config.llm_model.clone(),
                language: config.llm_language.clone(),
            },
            apifox: ApifoxSettings::default(), // Apifox 配置通过 mcp setup 命令管理
        };

        // 保存 workflow.toml
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);
        manager.write(&settings)?;

        Ok(())
    }
}
