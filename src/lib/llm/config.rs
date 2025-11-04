use anyhow::Result;
use std::env;

use crate::log_info;
use crate::EnvFile;

/// 获取当前 LLM provider（优先从环境变量读取，然后从 shell 配置文件读取，默认 openai）
pub fn get_llm_provider() -> Result<String> {
    // 1. 优先从当前进程的环境变量读取
    if let Ok(provider) = env::var("LLM_PROVIDER") {
        if !provider.is_empty() {
            log_info!("LLM_PROVIDER: {} (from environment variable)", provider);
            return Ok(provider);
        }
    }

    // 2. 从 shell 配置文件读取
    if let Ok(shell_config_env) = EnvFile::load() {
        if let Some(provider) = shell_config_env.get("LLM_PROVIDER") {
            if !provider.is_empty() {
                log_info!("LLM_PROVIDER: {} (from shell config file)", provider);
                return Ok(provider.clone());
            }
        }
    }

    // 3. 默认使用 openai
    log_info!("LLM_PROVIDER: openai (default)");
    Ok("openai".to_string())
}

