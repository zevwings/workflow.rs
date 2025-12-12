//! LLM 配置查看命令
//! 显示当前的 LLM 配置信息

use crate::base::settings::settings::Settings;
use crate::base::settings::table::LLMConfigRow;
use crate::base::util::table::{TableBuilder, TableStyle};
use crate::{log_break, log_info, log_message, log_success, log_warning};
use color_eyre::Result;

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

        // 使用 get_llm_config() 获取配置信息
        let llm_config = settings.get_llm_config();

        // 使用表格格式显示（与 config show 保持一致）
        log_info!("LLM Configuration");
        let config_rows = vec![LLMConfigRow {
            provider: llm_config.provider.clone(),
            model: llm_config.model.clone(),
            key: llm_config.key.clone(),
            language: llm_config.language.clone(),
        }];
        log_message!(
            "{}",
            TableBuilder::new(config_rows).with_style(TableStyle::Modern).render()
        );

        log_break!();
        log_success!("LLM configuration displayed.");

        Ok(())
    }

    /// 检查 LLM 配置是否为空
    fn is_empty_config(llm: &crate::base::settings::settings::LLMSettings) -> bool {
        llm.openai.is_empty()
            && llm.deepseek.is_empty()
            && llm.proxy.is_empty()
            && llm.provider == crate::base::settings::defaults::default_llm_provider()
            && llm.language == crate::base::settings::defaults::default_language()
    }
}
