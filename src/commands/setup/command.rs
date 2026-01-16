//! Setup 命令主流程

use crate::commands::setup::log as log_config;
use crate::commands::setup::{github, jira, llm};
use crate::config::settings::paths::Paths;
use crate::config::settings::Settings;
use crate::core::constants::messages;
use crate::services::jira::config::ConfigManager;
use crate::{br, info, success, warning};
use color_eyre::Result;

/// 初始化设置命令
pub struct SetupCommand;

impl SetupCommand {
    /// 运行初始化设置流程
    pub fn run() -> Result<()> {
        info!("Starting Workflow CLI initialization...\n");

        // 加载现有配置（从 TOML 文件）
        let existing_settings = Settings::get();

        // 收集配置信息（智能处理现有配置）
        let config = Self::collect_config(existing_settings)?;

        // 保存配置到 TOML 文件
        info!("Saving configuration...");
        Self::save_config(&config)?;
        if let Ok(config_path) = Paths::workflow_config() {
            success!(
                "{} {}",
                messages::LOG_CONFIG_SAVED_PREFIX,
                config_path.display()
            );
        } else {
            success!(
                "{} ~/.workflow/config/workflow.toml",
                messages::LOG_CONFIG_SAVED_PREFIX
            );
        }

        br!();
        info!("Verifying configuration...");

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

    /// 收集配置信息
    fn collect_config(settings: &Settings) -> Result<Settings> {
        // 1. GitHub 配置（独立函数，处理复杂逻辑）
        let github = github::handle_github_config(settings.github.clone())?;

        // 2. Jira 配置（使用 FormBuilder）
        let jira = jira::handle_jira_config(settings.jira.clone())?;

        // 3. LLM 配置（使用 FormBuilder）
        let llm = llm::handle_llm_config(settings.llm.clone())?;

        // 4. Log 配置（使用 FormBuilder）
        let log = log_config::handle_log_config(settings.log.clone())?;

        Ok(Settings {
            jira,
            github,
            log,
            llm,
            aliases: settings.aliases.clone(), // 保留现有别名
        })
    }

    /// 保存配置到 TOML 文件
    fn save_config(settings: &Settings) -> Result<()> {
        // 保存 workflow.toml
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<Settings>::new(config_path);
        manager.write(settings)?;

        Ok(())
    }
}
