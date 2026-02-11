//! 密码字段配置

use std::sync::Arc;

use crate::{dialog::Validator, form::field::types::Condition};

/// 密码字段配置
pub struct PasswordFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值（可选，空字符串表示无默认值）
    pub default_value: String,
    /// 验证器（可选）
    /// 使用 Arc 以便可以克隆并在多个地方使用
    pub validator: Option<Arc<dyn Validator + Send + Sync>>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl PasswordFormField {
    /// 创建新的密码字段
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

    /// 标记字段为必填
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

    /// 设置验证器
    pub fn validator(mut self, validator: Arc<dyn Validator + Send + Sync>) -> Self {
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
    fn test_password_form_field_new() {
        let field = PasswordFormField::new("password", "Enter password");
        assert_eq!(field.key, "password");
        assert_eq!(field.prompt, "Enter password");
        assert_eq!(field.default_value, "");
        assert!(field.validator.is_none());
        assert!(field.result_title.is_none());
        assert!(field.condition.is_none());
    }

    #[test]
    fn test_password_form_field_with_default() {
        let field = PasswordFormField::new("password", "Enter password").default("secret123");
        assert_eq!(field.default_value, "secret123");
    }

    #[test]
    fn test_password_form_field_with_result_title() {
        let field = PasswordFormField::new("password", "Enter password").result_title("Password");
        assert_eq!(field.result_title, Some("Password".to_string()));
    }

    #[test]
    fn test_password_form_field_required_validator() {
        let field = PasswordFormField::new("password", "Enter password").required();
        assert!(field.validator.is_some());

        let validator = field.validator.unwrap();
        // 空字符串应该验证失败
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
        // 非空字符串应该验证成功
        assert!(validator.validate("secret").is_ok());
    }

    #[test]
    fn test_password_form_field_custom_validator() {
        let validator = Arc::new(|input: &str| {
            if input.len() >= 8 {
                Ok(())
            } else {
                Err("Password must be at least 8 characters".to_string())
            }
        });
        let field = PasswordFormField::new("password", "Enter password").validator(validator);
        assert!(field.validator.is_some());
    }

    #[test]
    fn test_password_form_field_with_condition() {
        let condition: Condition = Box::new(|_result| true);
        let field = PasswordFormField::new("password", "Enter password").condition(condition);
        assert!(field.condition.is_some());
    }

    #[test]
    fn test_password_form_field_builder_chain() {
        let field = PasswordFormField::new("password", "Enter password")
            .default("default_pass")
            .result_title("User Password")
            .required();

        assert_eq!(field.key, "password");
        assert_eq!(field.prompt, "Enter password");
        assert_eq!(field.default_value, "default_pass");
        assert_eq!(field.result_title, Some("User Password".to_string()));
        assert!(field.validator.is_some());
    }
}
