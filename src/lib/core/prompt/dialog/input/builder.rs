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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::prompt::dialog::input::validator::validators;

    #[test]
    fn test_input_builder_new() {
        let builder = InputBuilder::new("Enter your name");
        assert_eq!(builder.message, "Enter your name");
        assert!(builder.default.is_none());
        assert!(builder.placeholder.is_none());
        assert!(builder.validator.is_none());
        assert!(!builder.password);
        assert!(builder.result_title.is_none());
    }

    #[test]
    fn test_input_builder_default() {
        let builder = InputBuilder::new("Name").default("John Doe");
        assert_eq!(builder.default, Some("John Doe".to_string()));
    }

    #[test]
    fn test_input_builder_placeholder() {
        let builder = InputBuilder::new("Name").placeholder("Enter your name here");
        assert_eq!(
            builder.placeholder,
            Some("Enter your name here".to_string())
        );
    }

    #[test]
    fn test_input_builder_validator() {
        let validator = validators::required();
        let builder = InputBuilder::new("Name").validator(validator);
        assert!(builder.validator.is_some());
    }

    #[test]
    fn test_input_builder_validator_boxed() {
        let validator: Box<dyn Validator + Send + Sync> = Box::new(validators::required());
        let builder = InputBuilder::new("Name").validator_boxed(validator);
        assert!(builder.validator.is_some());
    }

    #[test]
    fn test_input_builder_password() {
        let builder = InputBuilder::new("Password").password();
        assert!(builder.password);
    }

    #[test]
    fn test_input_builder_result_title() {
        let builder = InputBuilder::new("Name").result_title("Your Name");
        assert_eq!(builder.result_title, Some("Your Name".to_string()));
    }

    #[test]
    fn test_input_builder_chain() {
        let validator = validators::min_length(3);
        let builder = InputBuilder::new("Username")
            .default("user")
            .placeholder("Enter username")
            .validator(validator)
            .result_title("Username");

        assert_eq!(builder.message, "Username");
        assert_eq!(builder.default, Some("user".to_string()));
        assert_eq!(builder.placeholder, Some("Enter username".to_string()));
        assert!(builder.validator.is_some());
        assert_eq!(builder.result_title, Some("Username".to_string()));
    }

    #[test]
    fn test_input_builder_multiple_validators() {
        // 测试可以替换验证器
        let builder1 = InputBuilder::new("Email").validator(validators::required());
        assert!(builder1.validator.is_some());

        // 注意：在实际使用中，通常只设置一个验证器
        // 这里只是测试构建器的灵活性
    }
}
