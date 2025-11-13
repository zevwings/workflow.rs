//! 配置查看命令
//! 显示当前的 TOML 配置文件

use crate::settings::{paths::ConfigPaths, settings::Settings};
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
        let llm_config_path = ConfigPaths::llm_config()
            .map_err(|_| anyhow::anyhow!("Failed to get llm config path"))?;

        log_info!("Workflow config: {:?}", workflow_config_path);
        log_info!("LLM config: {:?}\n", llm_config_path);

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
            && settings.llm.is_none()
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

        // 代理配置
        log_info!(
            "  Disable Proxy Check: {}",
            if settings.proxy.disable_check {
                "Yes"
            } else {
                "No"
            }
        );

        // LLM 配置
        if let Some(ref llm) = settings.llm {
            log_info!("  LLM Provider: {}", llm.llm_provider);
            if let Some(ref key) = llm.openai_key {
                log_info!("  LLM OpenAI Key: {}", mask_sensitive_value(key));
            }
            if let Some(ref key) = llm.deepseek_key {
                log_info!("  LLM DeepSeek Key: {}", mask_sensitive_value(key));
            }
            if let Some(ref url) = llm.llm_proxy_url {
                log_info!("  LLM Proxy URL: {}", url);
            }
            if let Some(ref key) = llm.llm_proxy_key {
                log_info!("  LLM Proxy Key: {}", mask_sensitive_value(key));
            }
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
