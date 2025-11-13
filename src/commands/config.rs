//! 配置查看命令
//! 显示当前的 TOML 配置文件

use crate::settings::{
    defaults::{default_llm_model, default_response_format},
    paths::ConfigPaths,
    settings::Settings,
};
use crate::{log_break, log_info, log_warning, mask_sensitive_value};
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

        // 从 TOML 文件加载配置
        let settings = Settings::get();

        // 检查是否有配置
        if Self::is_empty_config(settings) {
            log_warning!("  No configuration found!");
            log_info!("   Run 'workflow setup' to initialize configuration.");
            return Ok(());
        }

        // 显示所有配置
        log_break!();
        log_break!('-', 100, "Configuration");
        Self::print_all_config(settings)?;

        Ok(())
    }

    /// 检查配置是否为空
    fn is_empty_config(settings: &Settings) -> bool {
        settings.user.email.is_none()
            && settings.jira.api_token.is_none()
            && settings.github.api_token.is_none()
            && settings.codeup.project_id.is_none()
            && settings.llm.url.is_none()
            && settings.llm.key.is_none()
    }

    /// 打印所有配置
    fn print_all_config(settings: &Settings) -> Result<()> {
        // 用户配置
        if let Some(ref email) = settings.user.email {
            log_info!("  Email: {}", email);
        }

        // Jira 配置
        if let Some(ref address) = settings.jira.service_address {
            log_info!("  Jira Service Address: {}", address);
        }
        if let Some(ref token) = settings.jira.api_token {
            log_info!("  Jira API Token: {}", mask_sensitive_value(token));
        }

        // GitHub 配置
        if let Some(ref token) = settings.github.api_token {
            log_info!("  GitHub API Token: {}", mask_sensitive_value(token));
        }
        if let Some(ref prefix) = settings.github.branch_prefix {
            log_info!("  GitHub Branch Prefix: {}", prefix);
        }

        // 日志配置
        log_info!(
            "  Log Output Folder Name: {}",
            settings.log.output_folder_name
        );
        log_info!(
            "  Delete Logs When Completed: {}",
            if settings.log.delete_when_completed {
                "Yes"
            } else {
                "No"
            }
        );
        if let Some(ref dir) = settings.log.download_base_dir {
            log_info!("  Download Base Dir: {}", dir);
        }

        // LLM 配置
        log_info!("  LLM Provider: {}", settings.llm.provider);
        if let Some(ref url) = settings.llm.url {
            log_info!("  LLM URL: {}", url);
        }
        if let Some(ref key) = settings.llm.key {
            log_info!("  LLM Key: {}", mask_sensitive_value(key));
        }
        // 显示 model（如果有保存的值，否则显示默认值）
        if let Some(ref model) = settings.llm.model {
            log_info!("  LLM Model: {}", model);
        } else {
            let default_model = default_llm_model(&settings.llm.provider);
            log_info!("  LLM Model: {} (default)", default_model);
        }
        // 显示 response_format（如果有保存的值，否则显示默认值）
        if settings.llm.response_format.is_empty() {
            let default_format = default_response_format();
            log_info!("  LLM Response Format: {} (default)", default_format);
        } else {
            log_info!("  LLM Response Format: {}", settings.llm.response_format);
        }

        // Codeup 配置
        if let Some(id) = settings.codeup.project_id {
            log_info!("  Codeup Project ID: {}", id);
        }
        if let Some(ref token) = settings.codeup.csrf_token {
            log_info!("  Codeup CSRF Token: {}", mask_sensitive_value(token));
        }
        if let Some(ref cookie) = settings.codeup.cookie {
            log_info!("  Codeup Cookie: {}", mask_sensitive_value(cookie));
        }

        Ok(())
    }
}
