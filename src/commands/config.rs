//! 配置查看命令
//! 显示当前的 TOML 配置文件

use crate::settings::{paths::ConfigPaths, settings::Settings};
use crate::{log_break, log_info, log_warning};
use anyhow::Result;

/// 配置查看命令
pub struct ConfigCommand;

impl ConfigCommand {
    /// 显示当前配置（从 TOML 文件读取）
    pub fn show() -> Result<()> {
        log_break!('=', 40, "Current Configuration");
        log_break!();

        // 显示配置文件路径
        let workflow_config_path = ConfigPaths::workflow_config()
            .map_err(|_| anyhow::anyhow!("Failed to get workflow config path"))?;

        log_info!("Workflow config: {:?}\n", workflow_config_path);

        // 从 TOML 文件加载配置（使用 load() 获取最新配置，避免 OnceLock 缓存问题）
        let settings = Settings::load();

        // 检查是否有配置
        if Self::is_empty_config(&settings) {
            log_warning!("  No configuration found!");
            log_info!("   Run 'workflow setup' to initialize configuration.");
            return Ok(());
        }

        // 显示所有配置
        log_break!();
        log_break!('-', 100, "Configuration");
        settings.verify()?;

        Ok(())
    }

    /// 检查配置是否为空
    fn is_empty_config(settings: &Settings) -> bool {
        settings.jira.email.is_none()
            && settings.jira.api_token.is_none()
            && settings.github.accounts.is_empty()
            && settings.codeup.project_id.is_none()
            && settings.llm.url.is_none()
            && settings.llm.key.is_none()
    }
}
