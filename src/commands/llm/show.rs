//! LLM 配置查看命令
//! 显示当前的 LLM 配置信息

use crate::base::interactive::{TableBuilder, TableStyle};
use crate::base::settings::LLMConfigRow;
use crate::base::settings::{LLMSettings, Settings};
use crate::{br, info, success, warning};
use color_eyre::Result;

/// LLM 配置查看命令
pub struct LLMShowCommand;

impl LLMShowCommand {
    /// 显示当前 LLM 配置
    pub fn show() -> Result<()> {
        br!('=', 40, "LLM Configuration");
        br!();

        let settings = Settings::load();
        let llm = &settings.llm;

        // 检查是否有 LLM 配置
        if Self::is_empty_config(llm) {
            warning!("No LLM configuration found.");
            info!("Run 'workflow llm setup' to configure LLM settings.");
            return Ok(());
        }

        // 使用 get_llm_config() 获取配置信息
        let llm_config = settings.get_llm_config();

        // 使用表格格式显示（与 config show 保持一致）
        info!("LLM Configuration");
        let config_rows = vec![LLMConfigRow {
            provider: llm_config.provider.clone(),
            model: llm_config.model.clone(),
            key: llm_config.key.clone(),
            language: llm_config.language.clone(),
        }];
        TableBuilder::from_tabled(config_rows).with_style(TableStyle::Modern).print()?;

        br!();
        success!("LLM configuration displayed.");

        Ok(())
    }

    /// 检查 LLM 配置是否为空
    fn is_empty_config(llm: &LLMSettings) -> bool {
        llm.openai.is_empty()
            && llm.deepseek.is_empty()
            && llm.proxy.is_empty()
            && llm.provider == LLMSettings::default_provider()
            && llm.language == LLMSettings::default_language()
    }
}
