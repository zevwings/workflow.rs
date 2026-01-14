//! 输入提示构建器
//!
//! 提供文本输入功能，支持密码模式、验证器、占位符等

use super::prompt::prompt;
use super::validator::Validator;
use crate::core::prompt::dialog::Result;

/// 输入提示构建器
pub struct InputBuilder {
    pub(crate) message: String,
    pub(crate) default: Option<String>,
    pub(crate) placeholder: Option<String>,
    pub(crate) validator: Option<Box<dyn Validator>>,
    pub(crate) password: bool,
    pub(crate) result_title: Option<String>,
}

impl InputBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            placeholder: None,
            validator: None,
            password: false,
            result_title: None,
        }
    }

    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        self.validator = Some(Box::new(validator));
        self
    }

    /// 设置验证器（接受已装箱的验证器）
    /// 用于从 FormField 传递验证器
    pub fn validator_boxed(mut self, validator: Box<dyn Validator + Send + Sync>) -> Self {
        // 转换类型：Box<dyn Validator + Send + Sync> -> Box<dyn Validator>
        // 这是安全的，因为 Validator trait 已经要求 Send + Sync
        self.validator = Some(validator);
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }

    /// 执行提示
    pub fn prompt(self) -> Result<String> {
        prompt(self)
    }
}
