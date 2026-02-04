//! 输入字段配置

use crate::dialog::Validator;
use crate::form::field::types::Condition;
use std::sync::Arc;

/// 输入字段配置
pub struct InputFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值（可选）
    pub default_value: String,
    /// 验证器（可选）
    /// 使用 Arc 以便可以克隆并在多个地方使用
    pub validator: Option<Arc<dyn Validator + Send + Sync>>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl InputFormField {
    /// 创建新的输入字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            default_value: String::new(),
            validator: None,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认值
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_value = value.into();
        self
    }

    /// 设置验证器
    pub fn validator(mut self, validator: Arc<dyn Validator + Send + Sync>) -> Self {
        self.validator = Some(validator);
        self
    }

    /// 标记字段为必填或非必填
    ///
    /// # Arguments
    ///
    /// * `required` - 是否必填，`true` 表示必填，`false` 表示可选
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prompt::InputFormField;
    ///
    /// // 必填字段
    /// InputFormField::new("name", "Enter name").required();
    ///
    /// // 非必填字段
    /// InputFormField::new("email", "Enter email");
    /// ```
    pub fn required(mut self) -> Self {
        let key = self.key.clone();
        let validator = Arc::new(move |input: &str| {
            if input.trim().is_empty() {
                Err(format!("Field '{}' is required", key))
            } else {
                Ok(())
            }
        });
        self.validator = Some(validator);
        self
    }

    /// 设置条件函数
    pub fn condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_form_field_new() {
        let field = InputFormField::new("username", "Enter username");
        assert_eq!(field.key, "username");
        assert_eq!(field.prompt, "Enter username");
        assert_eq!(field.default_value, "");
        assert!(field.validator.is_none());
        assert!(field.result_title.is_none());
        assert!(field.condition.is_none());
    }

    #[test]
    fn test_input_form_field_with_default() {
        let field = InputFormField::new("name", "Enter name").default("John");
        assert_eq!(field.default_value, "John");
    }

    #[test]
    fn test_input_form_field_with_result_title() {
        let field = InputFormField::new("email", "Enter email").result_title("Email Address");
        assert_eq!(field.result_title, Some("Email Address".to_string()));
    }

    #[test]
    fn test_input_form_field_required_validator() {
        let field = InputFormField::new("name", "Enter name").required();
        assert!(field.validator.is_some());

        let validator = field.validator.unwrap();
        // 空字符串应该验证失败
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
        // 非空字符串应该验证成功
        assert!(validator.validate("test").is_ok());
    }

    #[test]
    fn test_input_form_field_custom_validator() {
        let validator = Arc::new(|input: &str| {
            if input.len() >= 3 {
                Ok(())
            } else {
                Err("Input must be at least 3 characters".to_string())
            }
        });
        let field = InputFormField::new("code", "Enter code").validator(validator);
        assert!(field.validator.is_some());
    }

    #[test]
    fn test_input_form_field_with_condition() {
        let condition: Condition = Box::new(|_result| true);
        let field = InputFormField::new("optional", "Enter optional").condition(condition);
        assert!(field.condition.is_some());
    }

    #[test]
    fn test_input_form_field_builder_chain() {
        let field = InputFormField::new("username", "Enter username")
            .default("admin")
            .result_title("Username")
            .required();

        assert_eq!(field.key, "username");
        assert_eq!(field.prompt, "Enter username");
        assert_eq!(field.default_value, "admin");
        assert_eq!(field.result_title, Some("Username".to_string()));
        assert!(field.validator.is_some());
    }
}
