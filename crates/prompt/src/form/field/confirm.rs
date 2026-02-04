//! 确认字段配置

use crate::form::field::types::Condition;

/// 确认字段配置
pub struct ConfirmFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值
    pub default_value: bool,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl ConfirmFormField {
    /// 创建新的确认字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            default_value: false,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认值
    pub fn default(mut self, value: bool) -> Self {
        self.default_value = value;
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
    fn test_confirm_form_field_builder() {
        // 基本创建和默认值
        let field = ConfirmFormField::new("agree", "Do you agree?");
        assert_eq!(field.key, "agree");
        assert_eq!(field.prompt, "Do you agree?");
        assert!(!field.default_value);
        assert!(field.result_title.is_none());
        assert!(field.condition.is_none());

        // 链式调用
        let field = ConfirmFormField::new("terms", "Accept terms?")
            .default(true)
            .result_title("Terms Accepted");
        assert!(field.default_value);
        assert_eq!(field.result_title, Some("Terms Accepted".to_string()));

        // 条件函数
        let condition: Condition = Box::new(|_result| true);
        let field = ConfirmFormField::new("optional", "Optional?").condition(condition);
        assert!(field.condition.is_some());
    }
}
