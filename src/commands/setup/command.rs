//! Setup 命令主流程

use crate::base::constants::messages::log;
use crate::base::settings::paths::Paths;
use crate::base::settings::{
    GitHubSettings, JiraSettings, LLMProviderSettings, LLMSettings, LogSettings, Settings,
};
use crate::commands::setup::log as log_config;
use crate::commands::setup::types::CollectedConfig;
use crate::commands::setup::{github, jira, llm};
use crate::jira::config::ConfigManager;
use crate::{br, info, success, warning};
use color_eyre::Result;
use std::collections::HashMap;

/// 初始化设置命令
pub struct SetupCommand;

impl SetupCommand {
    /// 运行初始化设置流程
    pub fn run() -> Result<()> {
        success!("Starting Workflow CLI initialization...\n");

        // 加载现有配置（从 TOML 文件）
        let existing_config = Self::load_existing_config()?;

        // 收集配置信息（智能处理现有配置）
        let config = Self::collect_config(&existing_config)?;

        // 保存配置到 TOML 文件
        info!("Saving configuration...");
        Self::save_config(&config)?;
        if let Ok(config_path) = Paths::workflow_config() {
            success!("{} {}", log::CONFIG_SAVED_PREFIX, config_path.display());
        } else {
            success!(
                "{} ~/.workflow/config/workflow.toml",
                log::CONFIG_SAVED_PREFIX
            );
        }

        br!();
        info!("Verifying configuration...");
        br!();

        br!('-', 40, "Verifying Configuration");
        br!();

        // 检查配置文件权限
        if let Some(warning) = Settings::check_permissions() {
            warning!("{}", warning);
        }

        // 验证配置（使用 load() 获取最新配置，避免 OnceLock 缓存问题）
        let settings = Settings::load();

        // 逐个验证并展示结果
        crate::commands::check::check::CheckCommand::verify_and_display_all(&settings)?;

        br!();
        success!("Initialization completed successfully!");
        br!();
        info!("You can now use the Workflow CLI commands.");

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
        // 1. GitHub 配置（独立函数，处理复杂逻辑）
        let (github_accounts, github_current) = github::handle_github_config(existing)?;

        // 2. Jira 配置（使用 FormBuilder）
        let jira_config = jira::handle_jira_config(existing)?;

        // 3. LLM 配置（使用 FormBuilder）
        let llm_config = llm::handle_llm_config(existing)?;

        // 4. Log 配置（使用 FormBuilder）
        let log_config = log_config::handle_log_config(existing)?;

        Ok(CollectedConfig {
            jira_email: jira_config.email,
            jira_api_token: jira_config.api_token,
            jira_service_address: jira_config.service_address,
            github_accounts,
            github_current,
            log_output_folder_name: log_config.output_folder_name,
            log_download_base_dir: log_config.download_base_dir,
            enable_trace_console: log_config.enable_trace_console,
            llm_provider: llm_config.provider,
            llm_language: llm_config.language,
            llm_openai_key: llm_config.openai_key,
            llm_openai_model: llm_config.openai_model,
            llm_deepseek_key: llm_config.deepseek_key,
            llm_deepseek_model: llm_config.deepseek_model,
            llm_proxy_url: llm_config.proxy_url,
            llm_proxy_key: llm_config.proxy_key,
            llm_proxy_model: llm_config.proxy_model,
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
            llm: LLMSettings {
                provider: config.llm_provider.clone(),
                language: config.llm_language.clone(),
                openai: LLMProviderSettings {
                    url: None,
                    key: config.llm_openai_key.clone(),
                    model: config.llm_openai_model.clone(),
                },
                deepseek: LLMProviderSettings {
                    url: None,
                    key: config.llm_deepseek_key.clone(),
                    model: config.llm_deepseek_model.clone(),
                },
                proxy: LLMProviderSettings {
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
