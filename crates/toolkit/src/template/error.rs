//! 模板引擎错误类型

use thiserror::Error;

/// 模板引擎错误
#[derive(Debug, Error)]
pub enum TemplateError {
    /// 模板注册错误
    #[error("Template registration error: {0}")]
    Registration(#[from] handlebars::TemplateError),

    /// 模板渲染错误
    #[error("Template rendering error: {0}")]
    Rendering(#[from] handlebars::RenderError),

    /// 系统时间错误
    #[error("System time error: {0}")]
    SystemTime(String),
}
