use std::sync::Arc;

use client::LLMConfigContext;
use domain::{GlobalConfigRepository, LLMSettings};

/// LLM 配置上下文实现
///
/// 实现 `LLMConfigContext` trait，提供基于配置适配器的配置获取逻辑。
pub struct LLMConfigContextImpl {
    config: Arc<dyn GlobalConfigRepository>,
}

impl LLMConfigContextImpl {
    pub fn new(config: Arc<dyn GlobalConfigRepository>) -> Self {
        Self { config }
    }
}

impl LLMConfigContext for LLMConfigContextImpl {
    fn get_provider(&self) -> String {
        self.config.load().map(|c| c.llm.provider.clone()).unwrap_or_default()
    }

    fn get_current_provider_url(&self) -> String {
        self.config
            .load()
            .ok()
            .and_then(|c| c.llm.current_provider().url.clone())
            .unwrap_or_default()
    }

    fn get_current_provider_key(&self) -> String {
        self.config
            .load()
            .ok()
            .and_then(|c| c.llm.current_provider().key.clone())
            .unwrap_or_default()
    }

    fn get_current_provider_model(&self) -> String {
        self.config
            .load()
            .ok()
            .and_then(|c| {
                c.llm
                    .current_provider()
                    .model
                    .clone()
                    .or_else(|| Some(LLMSettings::default_model(&c.llm.provider)))
            })
            .unwrap_or_default()
    }

    fn get_language(&self) -> String {
        self.config
            .load()
            .map(|c| c.llm.language.clone())
            .unwrap_or_else(|_| "en".to_string())
    }
}
