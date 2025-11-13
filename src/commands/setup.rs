//! 初始化设置命令
//! 交互式配置应用，保存到 TOML 配置文件（~/.workflow/config/workflow.toml 和 llm.toml）

use crate::settings::{paths::ConfigPaths, settings::Settings};
use crate::{log_break, log_info, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use std::fs;
use toml;

/// 初始化设置命令
pub struct SetupCommand;

/// 收集的配置数据
#[derive(Debug, Clone)]
struct CollectedConfig {
    // Workflow 配置
    email: Option<String>,
    jira_api_token: Option<String>,
    jira_service_address: Option<String>,
    github_api_token: Option<String>,
    github_branch_prefix: Option<String>,
    log_output_folder_name: String,
    log_delete_when_completed: bool,
    disable_check_proxy: bool,
    codeup_project_id: Option<u64>,
    codeup_csrf_token: Option<String>,
    codeup_cookie: Option<String>,
    // LLM 配置
    llm_provider: String,
    llm_openai_key: Option<String>,
    llm_deepseek_key: Option<String>,
    llm_proxy_url: Option<String>,
    llm_proxy_key: Option<String>,
}

impl CollectedConfig {
    fn has_llm_config(&self) -> bool {
        self.llm_openai_key.is_some()
            || self.llm_deepseek_key.is_some()
            || self.llm_proxy_url.is_some()
    }
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
        log_info!("💾 Saving configuration...");
        Self::save_config(&config)?;
        log_success!("  Configuration saved to ~/.workflow/config/workflow.toml");
        if config.has_llm_config() {
            log_success!("  LLM configuration saved to ~/.workflow/config/llm.toml");
        }

        // 验证配置
        Self::verify_config(&config)?;

        log_break!();
        log_success!("Initialization completed successfully!");
        log_info!("   You can now use the Workflow CLI commands.");

        Ok(())
    }

    /// 加载现有配置（从 TOML 文件）
    fn load_existing_config() -> Result<CollectedConfig> {
        let settings = Settings::get();
        let llm_settings = settings.llm.as_ref();

        Ok(CollectedConfig {
            email: settings.user.email.clone(),
            jira_api_token: settings.jira.api_token.clone(),
            jira_service_address: settings.jira.service_address.clone(),
            github_api_token: settings.github.api_token.clone(),
            github_branch_prefix: settings.github.branch_prefix.clone(),
            log_output_folder_name: settings.log.output_folder_name.clone(),
            log_delete_when_completed: settings.log.delete_when_completed,
            disable_check_proxy: settings.proxy.disable_check,
            codeup_project_id: settings.codeup.project_id,
            codeup_csrf_token: settings.codeup.csrf_token.clone(),
            codeup_cookie: settings.codeup.cookie.clone(),
            llm_provider: llm_settings
                .map(|s| s.llm_provider.clone())
                .unwrap_or_else(|| "openai".to_string()),
            llm_openai_key: llm_settings.and_then(|s| s.openai_key.clone()),
            llm_deepseek_key: llm_settings.and_then(|s| s.deepseek_key.clone()),
            llm_proxy_url: llm_settings.and_then(|s| s.llm_proxy_url.clone()),
            llm_proxy_key: llm_settings.and_then(|s| s.llm_proxy_key.clone()),
        })
    }

    /// 收集配置信息
    fn collect_config(existing: &CollectedConfig) -> Result<CollectedConfig> {

        // ==================== 必填项：用户配置 ====================
        log_break!();
        log_info!("  User Configuration (Required)");
        log_break!('─', 65);

        let has_email = existing.email.is_some();
        let email_prompt = if let Some(ref email) = existing.email {
            format!("Email address [current: {}]", email)
        } else {
            "Email address".to_string()
        };

        let default_email = existing.email.clone().unwrap_or_default();

        let email: String = Input::new()
            .with_prompt(&email_prompt)
            .default(default_email)
            .validate_with(move |input: &String| -> Result<(), &str> {
                if input.is_empty() && !has_email {
                    Err("Email is required")
                } else if !input.is_empty() && !input.contains('@') {
                    Err("Please enter a valid email address")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .context("Failed to get email")?;

        let email = if !email.is_empty() {
            Some(email)
        } else if existing.email.is_some() {
            existing.email.clone()
        } else {
            anyhow::bail!("Email is required");
        };

        // ==================== 必填项：GitHub 配置 ====================
        log_break!();
        log_info!("🐙 GitHub Configuration (Required)");
        log_break!('─', 65);

        let github_token_prompt = if existing.github_api_token.is_some() {
            "GitHub API token [current: ***]".to_string()
        } else {
            "GitHub API token".to_string()
        };

        let github_api_token: String = Input::new()
            .with_prompt(&github_token_prompt)
            .allow_empty(existing.github_api_token.is_some())
            .interact_text()
            .context("Failed to get GitHub API token")?;

        let github_api_token = if !github_api_token.is_empty() {
            Some(github_api_token)
        } else if existing.github_api_token.is_some() {
            existing.github_api_token.clone()
        } else {
            anyhow::bail!("GitHub API token is required");
        };

        // ==================== 必填项：Jira 配置 ====================
        log_break!();
        log_info!("🎫 Jira Configuration (Required)");
        log_break!('─', 65);

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

        // ==================== 可选：GitHub 配置 ====================
        log_break!();
        log_info!("🐙 GitHub Configuration (Optional)");
        log_break!('─', 65);

        let gh_prefix_prompt = if let Some(ref prefix) = existing.github_branch_prefix {
            format!(
                "GitHub branch prefix [current: {}] (press Enter to keep)",
                prefix
            )
        } else {
            "GitHub branch prefix (press Enter to skip)".to_string()
        };

        let default_gh_prefix = existing.github_branch_prefix.clone().unwrap_or_default();

        let gh_prefix: String = Input::new()
            .with_prompt(&gh_prefix_prompt)
            .allow_empty(true)
            .default(default_gh_prefix)
            .interact_text()
            .context("Failed to get GitHub branch prefix")?;

        let github_branch_prefix = if !gh_prefix.is_empty() {
            Some(gh_prefix)
        } else {
            existing.github_branch_prefix.clone()
        };

        // ==================== 可选：日志配置 ====================
        log_break!();
        log_info!("📝 Log Configuration (Optional)");
        log_break!('─', 65);

        let log_folder_prompt = format!(
            "Log output folder name [current: {}]",
            existing.log_output_folder_name
        );

        let log_output_folder_name: String = Input::new()
            .with_prompt(&log_folder_prompt)
            .default(existing.log_output_folder_name.clone())
            .interact_text()
            .context("Failed to get log folder name")?;

        let log_output_folder_name = if !log_output_folder_name.is_empty() {
            log_output_folder_name
        } else {
            existing.log_output_folder_name.clone()
        };

        let delete_logs_prompt = format!(
            "Delete logs when operation completed? [current: {}]",
            if existing.log_delete_when_completed {
                "Yes"
            } else {
                "No"
            }
        );

        let log_delete_when_completed = Confirm::new()
            .with_prompt(&delete_logs_prompt)
            .default(existing.log_delete_when_completed)
            .interact()
            .context("Failed to get delete logs confirmation")?;

        // ==================== 可选：代理配置 ====================
        log_break!();
        log_info!("🌐 Proxy Configuration (Optional)");
        log_break!('─', 65);

        let disable_proxy_prompt = format!(
            "Disable proxy check? [current: {}]",
            if existing.disable_check_proxy {
                "Yes"
            } else {
                "No"
            }
        );

        let disable_check_proxy = Confirm::new()
            .with_prompt(&disable_proxy_prompt)
            .default(existing.disable_check_proxy)
            .interact()
            .context("Failed to get proxy check confirmation")?;

        // ==================== 可选：LLM/AI 配置 ====================
        log_break!();
        log_info!("🤖 LLM/AI Configuration (Optional)");
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

        let (llm_openai_key, llm_deepseek_key, llm_proxy_url, llm_proxy_key) =
            match llm_provider.as_str() {
                "openai" => {
                    let openai_key_prompt = if existing.llm_openai_key.is_some() {
                        "OpenAI API key [current: ***] (press Enter to keep)".to_string()
                    } else {
                        "OpenAI API key (optional, press Enter to skip)".to_string()
                    };

                    let openai_key: String = Input::new()
                        .with_prompt(&openai_key_prompt)
                        .allow_empty(true)
                        .interact_text()
                        .context("Failed to get OpenAI key")?;

                    let llm_openai_key = if !openai_key.is_empty() {
                        Some(openai_key)
                    } else {
                        existing.llm_openai_key.clone()
                    };
                    (llm_openai_key, None, None, None)
                }
                "deepseek" => {
                    let deepseek_key_prompt = if existing.llm_deepseek_key.is_some() {
                        "DeepSeek API key [current: ***] (press Enter to keep)".to_string()
                    } else {
                        "DeepSeek API key (optional, press Enter to skip)".to_string()
                    };

                    let deepseek_key: String = Input::new()
                        .with_prompt(&deepseek_key_prompt)
                        .allow_empty(true)
                        .interact_text()
                        .context("Failed to get DeepSeek key")?;

                    let llm_deepseek_key = if !deepseek_key.is_empty() {
                        Some(deepseek_key)
                    } else {
                        existing.llm_deepseek_key.clone()
                    };
                    (None, llm_deepseek_key, None, None)
                }
                "proxy" => {
                    let llm_proxy_url_prompt = if let Some(ref url) = existing.llm_proxy_url {
                        format!("LLM proxy URL [current: {}] (press Enter to keep)", url)
                    } else {
                        "LLM proxy URL (optional, press Enter to skip)".to_string()
                    };

                    let llm_proxy_url: String = Input::new()
                        .with_prompt(&llm_proxy_url_prompt)
                        .allow_empty(true)
                        .interact_text()
                        .context("Failed to get LLM proxy URL")?;

                    let llm_proxy_url = if !llm_proxy_url.is_empty() {
                        Some(llm_proxy_url)
                    } else {
                        existing.llm_proxy_url.clone()
                    };

                    let llm_proxy_key_prompt = if existing.llm_proxy_key.is_some() {
                        "LLM proxy key [current: ***] (press Enter to keep)".to_string()
                    } else {
                        "LLM proxy key (optional, press Enter to skip)".to_string()
                    };

                    let llm_proxy_key: String = Input::new()
                        .with_prompt(&llm_proxy_key_prompt)
                        .allow_empty(true)
                        .interact_text()
                        .context("Failed to get LLM proxy key")?;

                    let llm_proxy_key = if !llm_proxy_key.is_empty() {
                        Some(llm_proxy_key)
                    } else {
                        existing.llm_proxy_key.clone()
                    };
                    (None, None, llm_proxy_url, llm_proxy_key)
                }
                _ => (None, None, None, None),
            };

        // ==================== 可选：Codeup 配置 ====================
        log_break!();
        log_info!("📦 Codeup Configuration (Optional)");
        log_break!('─', 65);

        let has_codeup = existing.codeup_project_id.is_some()
            || existing.codeup_csrf_token.is_some()
            || existing.codeup_cookie.is_some();

        let codeup_confirm_prompt = if has_codeup {
            "Do you want to configure Codeup? [current: configured]".to_string()
        } else {
            "Do you use Codeup (Aliyun Code Repository)?".to_string()
        };

        let should_configure_codeup = Confirm::new()
            .with_prompt(&codeup_confirm_prompt)
            .default(has_codeup)
            .interact()
            .context("Failed to get Codeup confirmation")?;

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
            email,
            jira_api_token,
            jira_service_address,
            github_api_token,
            github_branch_prefix,
            log_output_folder_name,
            log_delete_when_completed,
            disable_check_proxy,
            codeup_project_id,
            codeup_csrf_token,
            codeup_cookie,
            llm_provider,
            llm_openai_key,
            llm_deepseek_key,
            llm_proxy_url,
            llm_proxy_key,
        })
    }

    /// 保存配置到 TOML 文件
    fn save_config(config: &CollectedConfig) -> Result<()> {
        use crate::settings::settings::{
            CodeupSettings, GitHubSettings, JiraSettings, LogSettings, ProxySettings,
            Settings, UserSettings, LLMSettingsToml,
        };

        // 构建 Settings 结构体
        let settings = Settings {
            user: UserSettings {
                email: config.email.clone(),
            },
            jira: JiraSettings {
                api_token: config.jira_api_token.clone(),
                service_address: config.jira_service_address.clone(),
            },
            github: GitHubSettings {
                api_token: config.github_api_token.clone(),
                branch_prefix: config.github_branch_prefix.clone(),
            },
            log: LogSettings {
                delete_when_completed: config.log_delete_when_completed,
                output_folder_name: config.log_output_folder_name.clone(),
                download_base_dir: None, // 使用默认值
            },
            proxy: ProxySettings {
                disable_check: config.disable_check_proxy,
            },
            codeup: CodeupSettings {
                project_id: config.codeup_project_id,
                csrf_token: config.codeup_csrf_token.clone(),
                cookie: config.codeup_cookie.clone(),
            },
            llm: None, // LLM 配置单独保存
        };

        // 保存 workflow.toml
        let workflow_config_path = ConfigPaths::workflow_config()?;
        let toml_content = toml::to_string_pretty(&settings)
            .context("Failed to serialize settings to TOML")?;
        fs::write(&workflow_config_path, toml_content)
            .context("Failed to write workflow.toml")?;

        // 保存 llm.toml（如果有 LLM 配置）
        if config.has_llm_config() {
            let llm_settings = LLMSettingsToml {
                openai_key: config.llm_openai_key.clone(),
                llm_proxy_url: config.llm_proxy_url.clone(),
                llm_proxy_key: config.llm_proxy_key.clone(),
                deepseek_key: config.llm_deepseek_key.clone(),
                llm_provider: config.llm_provider.clone(),
            };

            let llm_config_path = ConfigPaths::llm_config()?;
            let llm_toml_content = toml::to_string_pretty(&llm_settings)
                .context("Failed to serialize LLM settings to TOML")?;
            fs::write(&llm_config_path, llm_toml_content)
                .context("Failed to write llm.toml")?;
        }

        Ok(())
    }

    /// 验证配置
    fn verify_config(config: &CollectedConfig) -> Result<()> {
        // 验证 Jira 配置
        if config.jira_api_token.is_some()
            && config.jira_service_address.is_some()
            && config.email.is_some()
        {
            log_break!();
            log_info!("🔍 Verifying Jira configuration...");

            match crate::jira::users::get_user_info() {
                Ok(user) => {
                    log_break!();
                    log_success!("Jira configuration verified successfully!");
                    log_info!("   User: {}", user.display_name);
                    if let Some(email) = &user.email_address {
                        log_info!("   Email: {}", email);
                    }
                    log_info!("   Account ID: {}", user.account_id);
                }
                Err(e) => {
                    log_warning!("  Failed to verify Jira configuration");
                    log_info!("   Error: {}", e);
                    log_info!("   Please check your Jira service address and API token.");
                    log_info!("   You can run 'workflow setup' again to update the configuration.");
                }
            }
        }

        // 验证 GitHub 配置
        if config.github_api_token.is_some() {
            log_break!();
            log_info!("🔍 Verifying GitHub configuration...");

            match crate::pr::GitHub::get_user_info() {
                Ok(user) => {
                    log_break!();
                    log_success!("GitHub configuration verified successfully!");
                    log_info!("   User: {}", user.login);
                    if let Some(name) = &user.name {
                        log_info!("   Name: {}", name);
                    }
                    if let Some(email) = &user.email {
                        log_info!("   Email: {}", email);
                    }
                }
                Err(e) => {
                    log_warning!("  Failed to verify GitHub configuration");
                    log_info!("   Error: {}", e);
                    log_info!("   Please check your GitHub API token.");
                    log_info!("   You can run 'workflow setup' again to update the configuration.");
                }
            }
        }

        // 验证 Codeup 配置
        if config.codeup_project_id.is_some()
            && config.codeup_cookie.is_some()
            && config.codeup_csrf_token.is_some()
        {
            log_break!();
            log_info!("🔍 Verifying Codeup configuration...");

            match crate::pr::Codeup::get_user_info() {
                Ok(user) => {
                    log_break!();
                    log_success!("Codeup configuration verified successfully!");
                    if let Some(name) = &user.name {
                        log_info!("   Name: {}", name);
                    }
                    if let Some(username) = &user.username {
                        log_info!("   Username: {}", username);
                    }
                    if let Some(email) = &user.email {
                        log_info!("   Email: {}", email);
                    }
                    if let Some(id) = user.id {
                        log_info!("   ID: {}", id);
                    }
                }
                Err(e) => {
                    log_warning!("  Failed to verify Codeup configuration");
                    log_info!("   Error: {}", e);
                    log_info!("   Please check your Codeup project ID, cookie, and CSRF token.");
                    log_info!("   You can run 'workflow setup' again to update the configuration.");
                }
            }
        }

        Ok(())
    }
}
