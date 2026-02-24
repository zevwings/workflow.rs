//! Codeup 配置上下文实现
//!
//! 实现 `client::codeup::context::CodeupConfigContext` trait，
//! 提供配置获取逻辑。

use std::sync::Arc;

use client::{CodeupClientError, CodeupConfigContext};
use domain::GlobalConfigRepository;

/// Codeup 配置上下文实现
///
/// 实现 `CodeupConfigContext` trait，提供基于配置适配器的配置获取逻辑。
pub struct CodeupContextImpl {
    config: Arc<dyn GlobalConfigRepository>,
}

impl CodeupContextImpl {
    pub fn new(config: Arc<dyn GlobalConfigRepository>) -> Self {
        Self { config }
    }

    /// 获取 Codeup 配置
    fn get_codeup_settings(&self) -> Result<domain::CodeupSettings, CodeupClientError> {
        let config = self
            .config
            .load()
            .map_err(|e| CodeupClientError::ConfigError(format!("加载配置失败: {}", e)))?;
        Ok(config.codeup)
    }
}

// 实现 client 层的 CodeupConfigContext
impl CodeupConfigContext for CodeupContextImpl {
    fn get_project_id(&self) -> Result<String, CodeupClientError> {
        let settings = self.get_codeup_settings()?;
        if settings.project_id.is_empty() {
            return Err(CodeupClientError::ConfigError(
                "Codeup project_id 未配置".to_string(),
            ));
        }
        Ok(settings.project_id)
    }

    fn get_csrf_token(&self) -> Result<String, CodeupClientError> {
        let settings = self.get_codeup_settings()?;
        if settings.csrf_token.is_empty() {
            return Err(CodeupClientError::ConfigError(
                "Codeup csrf_token 未配置".to_string(),
            ));
        }
        Ok(settings.csrf_token)
    }

    fn get_cookie(&self) -> Result<String, CodeupClientError> {
        let settings = self.get_codeup_settings()?;
        if settings.cookie.is_empty() {
            return Err(CodeupClientError::ConfigError(
                "Codeup cookie 未配置".to_string(),
            ));
        }
        Ok(settings.cookie)
    }
}
