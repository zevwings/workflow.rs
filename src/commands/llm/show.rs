//! LLM 配置查看命令
//! 显示当前的 LLM 配置信息

use crate::base::settings::settings::Settings;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::Result;

/// LLM 配置查看命令
pub struct LLMShowCommand;

impl LLMShowCommand {
    /// 显示当前 LLM 配置
    pub fn show() -> Result<()> {
        log_break!('=', 40, "LLM Configuration");
        log_break!();

        let settings = Settings::load();
        let llm = &settings.llm;

        // 检查是否有 LLM 配置
        if Self::is_empty_config(llm) {
            log_warning!("No LLM configuration found.");
            log_message!("Run 'workflow llm setup' to configure LLM settings.");
            return Ok(());
        }

        // 显示 Provider
        log_info!("Provider: {}", llm.provider);

        // 显示 URL（仅 proxy 模式）
        if llm.provider == "proxy" {
            if let Some(ref url) = llm.url {
                log_info!("Proxy URL: {}", url);
            } else {
                log_warning!("Proxy URL: Not configured");
            }
        }

        // 显示 API Key（掩码显示）
        if let Some(ref key) = llm.key {
            let masked = if key.len() > 8 {
                format!("{}...{}", &key[..4], &key[key.len() - 4..])
            } else {
                "***".to_string()
            };
            log_info!("API Key: {}", masked);
        } else {
            log_warning!("API Key: Not configured");
        }

        // 显示 Model
        if let Some(ref model) = llm.model {
            log_info!("Model: {}", model);
        } else {
            log_info!("Model: (using default)");
        }

        // 显示 Language
        if !llm.language.is_empty() {
            log_info!("Output Language: {}", llm.language);
        } else {
            log_info!("Output Language: en (default)");
        }

        log_break!();
        log_success!("LLM configuration displayed.");

        Ok(())
    }

    /// 检查 LLM 配置是否为空
    fn is_empty_config(llm: &crate::base::settings::settings::LLMSettings) -> bool {
        llm.url.is_none()
            && llm.key.is_none()
            && llm.model.is_none()
            && llm.provider == crate::base::settings::defaults::default_llm_provider()
            && llm.language == crate::base::settings::defaults::default_language()
    }
}
