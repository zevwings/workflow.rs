//! 初始化设置命令
//! 交互式配置应用，保存到 shell 配置文件（~/.zshrc, ~/.bash_profile 等）

use crate::{EnvFile, Shell, log_debug, log_info, log_break, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use std::collections::HashMap;

/// 初始化设置命令
pub struct SetupCommand;

impl SetupCommand {
    /// 运行初始化设置流程
    pub fn run() -> Result<()> {
        log_success!("Starting Workflow CLI initialization...\n");

        // 注意：在 setup 阶段，我们直接读取环境变量和 shell 配置文件即可

        // 加载现有配置（从 shell 配置文件和当前环境变量）
        // 优先从当前环境变量读取（如果已加载到 shell）
        let env_var_keys = EnvFile::get_workflow_env_keys();
        let merged_env = EnvFile::load_merged(&env_var_keys);

        if !merged_env.is_empty() {
            log_info!("  Found existing configuration. Press Enter to keep current values, or enter new values to override.\n");
        }

        // 收集配置信息（智能处理现有配置）
        let env_vars = Self::collect_config(&merged_env)?;

        // 保存配置（统一保存到环境变量）
        log_info!("💾 Saving configuration...");
        EnvFile::save(&env_vars).context("Failed to save environment variables")?;
        log_success!("  Environment variables saved to shell config file");

        let _shell_config_path = EnvFile::get_shell_config_path()
            .map_err(|_| anyhow::anyhow!("Failed to get shell config path"))?;
        log_debug!("   Shell config: {:?}", _shell_config_path);

        // 保存配置后，更新当前进程的环境变量
        // 这样后续对 Settings::load() 的调用会使用新值
        for (key, value) in &env_vars {
            std::env::set_var(key, value);
        }

        // 验证 Jira 配置（如果已配置）
        Self::verify_jira_config(&env_vars)?;

        // 验证 GitHub 配置（如果已配置）
        Self::verify_github_config(&env_vars)?;

        // 验证 Codeup 配置（如果已配置）
        Self::verify_codeup_config(&env_vars)?;

        log_break!();
        log_success!("Initialization completed successfully!");
        log_info!("   You can now use the Workflow CLI commands.");

        // 尝试重新加载 shell 配置
        log_break!();
        log_info!("Reloading shell configuration...");
        if let Ok(shell_info) = Shell::detect() {
            let _ = Shell::reload_config(&shell_info);
        } else {
            log_break!();
            log_info!("  Could not detect shell type.");
            log_info!("Please manually reload your shell configuration:");
            log_info!("  source ~/.zshrc  # for zsh");
            log_info!("  source ~/.bashrc  # for bash");
        }

        Ok(())
    }

    /// 收集配置信息（统一保存为环境变量）
    fn collect_config(existing_env: &HashMap<String, String>) -> Result<HashMap<String, String>> {
        let mut env_vars = existing_env.clone();

        // ==================== 必填项：用户配置 ====================
        log_break!();
        log_info!("  User Configuration (Required)");
        log_break!('─', 65);

        let current_email = existing_env.get("EMAIL").cloned();
        let has_email = current_email.is_some();
        let email_prompt = if let Some(ref email) = current_email {
            format!("Email address [current: {}]", email)
        } else {
            "Email address".to_string()
        };

        let default_email = current_email.clone().unwrap_or_default();

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

        if !email.is_empty() {
            env_vars.insert("EMAIL".to_string(), email);
        } else if current_email.is_none() {
            anyhow::bail!("Email is required");
        }

        // ==================== 必填项：GitHub 配置 ====================
        log_break!();
        log_info!("🐙 GitHub Configuration (Required)");
        log_break!('─', 65);

        let current_github_token = existing_env.get("GITHUB_API_TOKEN").cloned();
        let github_token_prompt = if current_github_token.is_some() {
            "GitHub API token [current: ***]".to_string()
        } else {
            "GitHub API token".to_string()
        };

        let github_api_token: String = Input::new()
            .with_prompt(&github_token_prompt)
            .allow_empty(current_github_token.is_some())
            .interact_text()
            .context("Failed to get GitHub API token")?;

        if !github_api_token.is_empty() {
            env_vars.insert("GITHUB_API_TOKEN".to_string(), github_api_token);
        } else if current_github_token.is_some() {
            // 保持原值
            env_vars.insert(
                "GITHUB_API_TOKEN".to_string(),
                current_github_token.unwrap(),
            );
        } else {
            anyhow::bail!("GitHub API token is required");
        }

        // ==================== 必填项：Jira 配置 ====================
        log_break!();
        log_info!("🎫 Jira Configuration (Required)");
        log_break!('─', 65);

        let current_jira_address = existing_env.get("JIRA_SERVICE_ADDRESS").cloned();
        let has_jira_address = current_jira_address.is_some();
        let jira_address_prompt = if let Some(ref addr) = current_jira_address {
            format!("Jira service address [current: {}]", addr)
        } else {
            "Jira service address".to_string()
        };

        let default_jira_address = current_jira_address
            .clone()
            .unwrap_or_else(|| "https://your-company.atlassian.net".to_string());

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

        if !jira_service_address.is_empty() {
            env_vars.insert("JIRA_SERVICE_ADDRESS".to_string(), jira_service_address);
        } else if current_jira_address.is_none() {
            anyhow::bail!("Jira service address is required");
        }

        let current_jira_token = existing_env.get("JIRA_API_TOKEN").cloned();
        let jira_token_prompt = if current_jira_token.is_some() {
            "Jira API token [current: ***]".to_string()
        } else {
            "Jira API token".to_string()
        };

        let jira_api_token: String = Input::new()
            .with_prompt(&jira_token_prompt)
            .allow_empty(current_jira_token.is_some())
            .interact_text()
            .context("Failed to get Jira API token")?;

        if !jira_api_token.is_empty() {
            env_vars.insert("JIRA_API_TOKEN".to_string(), jira_api_token);
        } else if current_jira_token.is_none() {
            anyhow::bail!("Jira API token is required");
        }

        // ==================== 可选：GitHub 配置 ====================
        log_break!();
        log_info!("🐙 GitHub Configuration (Optional)");
        log_break!('─', 65);

        let current_gh_prefix = existing_env.get("GITHUB_BRANCH_PREFIX").cloned();
        let gh_prefix_prompt = if let Some(ref prefix) = current_gh_prefix {
            format!(
                "GitHub branch prefix [current: {}] (press Enter to keep)",
                prefix
            )
        } else {
            "GitHub branch prefix (press Enter to skip)".to_string()
        };

        let default_gh_prefix = current_gh_prefix.clone().unwrap_or_default();

        let gh_prefix: String = Input::new()
            .with_prompt(&gh_prefix_prompt)
            .allow_empty(true)
            .default(default_gh_prefix)
            .interact_text()
            .context("Failed to get GitHub branch prefix")?;

        if !gh_prefix.is_empty() {
            env_vars.insert("GITHUB_BRANCH_PREFIX".to_string(), gh_prefix);
        } else if let Some(prefix) = current_gh_prefix {
            // 保持原值
            env_vars.insert("GITHUB_BRANCH_PREFIX".to_string(), prefix);
        }

        // ==================== 可选：日志配置 ====================
        log_break!();
        log_info!("📝 Log Configuration (Optional)");
        log_break!('─', 65);

        let current_log_folder = existing_env
            .get("LOG_OUTPUT_FOLDER_NAME")
            .cloned()
            .unwrap_or_else(|| "logs".to_string());
        let log_folder_prompt = format!("Log output folder name [current: {}]", current_log_folder);

        let log_folder: String = Input::new()
            .with_prompt(&log_folder_prompt)
            .default(current_log_folder.clone())
            .interact_text()
            .context("Failed to get log folder name")?;

        if !log_folder.is_empty() {
            env_vars.insert("LOG_OUTPUT_FOLDER_NAME".to_string(), log_folder);
        } else {
            // 保持原值或使用默认值
            env_vars.insert("LOG_OUTPUT_FOLDER_NAME".to_string(), current_log_folder);
        }

        let current_delete_logs = existing_env
            .get("LOG_DELETE_WHEN_OPERATION_COMPLETED")
            .map(|v| v == "1")
            .unwrap_or(false);

        let delete_logs_prompt = format!(
            "Delete logs when operation completed? [current: {}]",
            if current_delete_logs { "Yes" } else { "No" }
        );

        let delete_logs = Confirm::new()
            .with_prompt(&delete_logs_prompt)
            .default(current_delete_logs)
            .interact()
            .context("Failed to get delete logs confirmation")?;
        env_vars.insert(
            "LOG_DELETE_WHEN_OPERATION_COMPLETED".to_string(),
            if delete_logs {
                "1".to_string()
            } else {
                "0".to_string()
            },
        );

        // ==================== 可选：代理配置 ====================
        log_break!();
        log_info!("🌐 Proxy Configuration (Optional)");
        log_break!('─', 65);

        let current_disable_proxy = existing_env
            .get("DISABLE_CHECK_PROXY")
            .map(|v| v == "1")
            .unwrap_or(false);

        let disable_proxy_prompt = format!(
            "Disable proxy check? [current: {}]",
            if current_disable_proxy { "Yes" } else { "No" }
        );

        let disable_proxy_check = Confirm::new()
            .with_prompt(&disable_proxy_prompt)
            .default(current_disable_proxy)
            .interact()
            .context("Failed to get proxy check confirmation")?;
        env_vars.insert(
            "DISABLE_CHECK_PROXY".to_string(),
            if disable_proxy_check {
                "1".to_string()
            } else {
                "0".to_string()
            },
        );

        // ==================== 可选：LLM/AI 配置 ====================
        log_break!();
        log_info!("🤖 LLM/AI Configuration (Optional)");
        log_break!('─', 65);

        let llm_providers = vec!["openai", "deepseek", "proxy"];
        let current_provider = existing_env
            .get("LLM_PROVIDER")
            .cloned()
            .unwrap_or_else(|| "openai".to_string());
        let current_provider_idx = llm_providers
            .iter()
            .position(|&p| p == current_provider.as_str())
            .unwrap_or(0);

        let llm_provider_prompt = format!("Select LLM provider [current: {}]", current_provider);

        let llm_provider_idx = Select::new()
            .with_prompt(&llm_provider_prompt)
            .items(&llm_providers)
            .default(current_provider_idx)
            .interact()
            .context("Failed to select LLM provider")?;
        let selected_provider = llm_providers[llm_provider_idx].to_string();
        env_vars.insert("LLM_PROVIDER".to_string(), selected_provider.clone());

        match selected_provider.as_str() {
            "openai" => {
                let current_openai_key = existing_env.get("LLM_OPENAI_KEY").cloned();
                let openai_key_prompt = if current_openai_key.is_some() {
                    "OpenAI API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "OpenAI API key (optional, press Enter to skip)".to_string()
                };

                let openai_key: String = Input::new()
                    .with_prompt(&openai_key_prompt)
                    .allow_empty(true)
                    .interact_text()
                    .context("Failed to get OpenAI key")?;

                if !openai_key.is_empty() {
                    env_vars.insert("LLM_OPENAI_KEY".to_string(), openai_key);
                } else if current_openai_key.is_some() {
                    // 保持原值
                    env_vars.insert("LLM_OPENAI_KEY".to_string(), current_openai_key.unwrap());
                }
            }
            "deepseek" => {
                let current_deepseek_key = existing_env.get("LLM_DEEPSEEK_KEY").cloned();
                let deepseek_key_prompt = if current_deepseek_key.is_some() {
                    "DeepSeek API key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "DeepSeek API key (optional, press Enter to skip)".to_string()
                };

                let deepseek_key: String = Input::new()
                    .with_prompt(&deepseek_key_prompt)
                    .allow_empty(true)
                    .interact_text()
                    .context("Failed to get DeepSeek key")?;

                if !deepseek_key.is_empty() {
                    env_vars.insert("LLM_DEEPSEEK_KEY".to_string(), deepseek_key);
                } else if current_deepseek_key.is_some() {
                    env_vars.insert(
                        "LLM_DEEPSEEK_KEY".to_string(),
                        current_deepseek_key.unwrap(),
                    );
                }
            }
            "proxy" => {
                let current_llm_proxy_url = existing_env.get("LLM_PROXY_URL").cloned();
                let llm_proxy_url_prompt = if let Some(ref url) = current_llm_proxy_url {
                    format!("LLM proxy URL [current: {}] (press Enter to keep)", url)
                } else {
                    "LLM proxy URL (optional, press Enter to skip)".to_string()
                };

                let llm_proxy_url: String = Input::new()
                    .with_prompt(&llm_proxy_url_prompt)
                    .allow_empty(true)
                    .interact_text()
                    .context("Failed to get LLM proxy URL")?;

                if !llm_proxy_url.is_empty() {
                    env_vars.insert("LLM_PROXY_URL".to_string(), llm_proxy_url);
                } else if current_llm_proxy_url.is_some() {
                    env_vars.insert("LLM_PROXY_URL".to_string(), current_llm_proxy_url.unwrap());
                }

                let current_llm_proxy_key = existing_env.get("LLM_PROXY_KEY").cloned();
                let llm_proxy_key_prompt = if current_llm_proxy_key.is_some() {
                    "LLM proxy key [current: ***] (press Enter to keep)".to_string()
                } else {
                    "LLM proxy key (optional, press Enter to skip)".to_string()
                };

                let llm_proxy_key: String = Input::new()
                    .with_prompt(&llm_proxy_key_prompt)
                    .allow_empty(true)
                    .interact_text()
                    .context("Failed to get LLM proxy key")?;

                if !llm_proxy_key.is_empty() {
                    env_vars.insert("LLM_PROXY_KEY".to_string(), llm_proxy_key);
                } else if current_llm_proxy_key.is_some() {
                    env_vars.insert("LLM_PROXY_KEY".to_string(), current_llm_proxy_key.unwrap());
                }
            }
            _ => {}
        }

        // ==================== 可选：Codeup 配置 ====================
        log_break!();
        log_info!("📦 Codeup Configuration (Optional)");
        log_break!('─', 65);

        let has_codeup = existing_env.contains_key("CODEUP_PROJECT_ID")
            || existing_env.contains_key("CODEUP_CSRF_TOKEN")
            || existing_env.contains_key("CODEUP_COOKIE");

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

        if should_configure_codeup {
            let current_codeup_id = existing_env.get("CODEUP_PROJECT_ID").cloned();
            let codeup_id_prompt = if let Some(ref id) = current_codeup_id {
                format!("Codeup project ID [current: {}] (press Enter to keep)", id)
            } else {
                "Codeup project ID (optional, press Enter to skip)".to_string()
            };

            let default_codeup_id = current_codeup_id.clone().unwrap_or_default();

            let codeup_project_id: String = Input::new()
                .with_prompt(&codeup_id_prompt)
                .allow_empty(true)
                .default(default_codeup_id)
                .interact_text()
                .context("Failed to get Codeup project ID")?;

            if !codeup_project_id.is_empty() {
                if let Ok(id) = codeup_project_id.parse::<u64>() {
                    env_vars.insert("CODEUP_PROJECT_ID".to_string(), id.to_string());
                } else {
                    log_warning!("Invalid project ID, skipping...");
                    if let Some(id) = current_codeup_id {
                        env_vars.insert("CODEUP_PROJECT_ID".to_string(), id);
                    }
                }
            } else if let Some(id) = current_codeup_id {
                // 保持原值
                env_vars.insert("CODEUP_PROJECT_ID".to_string(), id);
            }

            let current_codeup_csrf = existing_env.get("CODEUP_CSRF_TOKEN").cloned();
            let codeup_csrf_prompt = if current_codeup_csrf.is_some() {
                "Codeup CSRF token [current: ***] (press Enter to keep)".to_string()
            } else {
                "Codeup CSRF token (optional, press Enter to skip)".to_string()
            };

            let codeup_csrf_token: String = Input::new()
                .with_prompt(&codeup_csrf_prompt)
                .allow_empty(true)
                .interact_text()
                .context("Failed to get Codeup CSRF token")?;

            if !codeup_csrf_token.is_empty() {
                env_vars.insert("CODEUP_CSRF_TOKEN".to_string(), codeup_csrf_token);
            } else if current_codeup_csrf.is_some() {
                env_vars.insert(
                    "CODEUP_CSRF_TOKEN".to_string(),
                    current_codeup_csrf.unwrap(),
                );
            }

            let current_codeup_cookie = existing_env.get("CODEUP_COOKIE").cloned();
            let codeup_cookie_prompt = if current_codeup_cookie.is_some() {
                "Codeup cookie [current: ***] (press Enter to keep)".to_string()
            } else {
                "Codeup cookie (optional, press Enter to skip)".to_string()
            };

            let codeup_cookie: String = Input::new()
                .with_prompt(&codeup_cookie_prompt)
                .allow_empty(true)
                .interact_text()
                .context("Failed to get Codeup cookie")?;

            if !codeup_cookie.is_empty() {
                env_vars.insert("CODEUP_COOKIE".to_string(), codeup_cookie);
            } else if current_codeup_cookie.is_some() {
                env_vars.insert("CODEUP_COOKIE".to_string(), current_codeup_cookie.unwrap());
            }
        } else if has_codeup {
            // 用户选择不配置，但已存在配置，保持原值
            if let Some(id) = existing_env.get("CODEUP_PROJECT_ID") {
                env_vars.insert("CODEUP_PROJECT_ID".to_string(), id.clone());
            }
            if let Some(token) = existing_env.get("CODEUP_CSRF_TOKEN") {
                env_vars.insert("CODEUP_CSRF_TOKEN".to_string(), token.clone());
            }
            if let Some(cookie) = existing_env.get("CODEUP_COOKIE") {
                env_vars.insert("CODEUP_COOKIE".to_string(), cookie.clone());
            }
        }

        Ok(env_vars)
    }

    /// 验证 Jira 配置
    ///
    /// 尝试获取 Jira 用户信息来验证配置是否正确。
    fn verify_jira_config(env_vars: &HashMap<String, String>) -> Result<()> {
        // 检查是否配置了 Jira 相关信息
        let has_jira_config = env_vars.contains_key("JIRA_SERVICE_ADDRESS")
            && env_vars.contains_key("JIRA_API_TOKEN")
            && env_vars.contains_key("EMAIL");

        if !has_jira_config {
            return Ok(());
        }

        log_break!();
        log_info!("🔍 Verifying Jira configuration...");

        // 尝试获取 Jira 用户信息
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

        Ok(())
    }

    /// 验证 GitHub 配置
    ///
    /// 尝试获取 GitHub 用户信息来验证配置是否正确。
    fn verify_github_config(env_vars: &HashMap<String, String>) -> Result<()> {
        // 检查是否配置了 GitHub API token
        let has_github_config = env_vars.contains_key("GITHUB_API_TOKEN");

        if !has_github_config {
            return Ok(());
        }

        log_break!();
        log_info!("🔍 Verifying GitHub configuration...");

        // 尝试获取 GitHub 用户信息
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

        Ok(())
    }

    /// 验证 Codeup 配置
    ///
    /// 尝试获取 Codeup 用户信息来验证配置是否正确。
    fn verify_codeup_config(env_vars: &HashMap<String, String>) -> Result<()> {
        // 检查是否配置了 Codeup 相关信息
        let has_codeup_config = env_vars.contains_key("CODEUP_PROJECT_ID")
            && env_vars.contains_key("CODEUP_COOKIE")
            && env_vars.contains_key("CODEUP_CSRF_TOKEN");

        if !has_codeup_config {
            return Ok(());
        }

        log_break!();
        log_info!("🔍 Verifying Codeup configuration...");

        // 尝试获取 Codeup 用户信息
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

        Ok(())
    }
}
