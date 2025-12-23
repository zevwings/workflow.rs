//! 字段构建器，用于在步骤中构建字段

use std::sync::Arc;

use crate::base::dialog::form::types::{FieldDefaultValue, FormField, FormFieldType};

/// 字段构建器
///
/// 用于在步骤中构建字段，提供 `add_text`、`add_selection` 等方法。
#[derive(Clone)]
pub struct FieldBuilder {
    /// 表单字段列表
    pub fields: Vec<FormField>,
    /// 当前正在构建的字段索引
    current_field: Option<usize>,
}

impl FieldBuilder {
    /// 创建新的字段构建器
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            current_field: None,
        }
    }

    /// 添加文本输入字段
    pub fn add_text(mut self, name: impl Into<String>, message: impl Into<String>) -> Self {
        let field = FormField {
            name: name.into(),
            field_type: FormFieldType::Text,
            message: message.into(),
            choices: Vec::new(),
            default_choice: None,
            default_value: None,
            required: false,
            allow_empty: false,
            validator: None,
            condition: None,
        };
        self.current_field = Some(self.fields.len());
        self.fields.push(field);
        self
    }

    /// 添加密码输入字段
    pub fn add_password(mut self, name: impl Into<String>, message: impl Into<String>) -> Self {
        let field = FormField {
            name: name.into(),
            field_type: FormFieldType::Password,
            message: message.into(),
            choices: Vec::new(),
            default_choice: None,
            default_value: None,
            required: false,
            allow_empty: false,
            validator: None,
            condition: None,
        };
        self.current_field = Some(self.fields.len());
        self.fields.push(field);
        self
    }

    /// 添加选择字段
    pub fn add_selection(
        mut self,
        name: impl Into<String>,
        message: impl Into<String>,
        choices: Vec<String>,
    ) -> Self {
        let field = FormField {
            name: name.into(),
            field_type: FormFieldType::Selection,
            message: message.into(),
            choices,
            default_choice: None,
            default_value: None,
            required: false,
            allow_empty: false,
            validator: None,
            condition: None,
        };
        self.current_field = Some(self.fields.len());
        self.fields.push(field);
        self
    }

    /// 添加确认字段
    pub fn add_confirmation(mut self, name: impl Into<String>, message: impl Into<String>) -> Self {
        let field = FormField {
            name: name.into(),
            field_type: FormFieldType::Confirmation,
            message: message.into(),
            choices: Vec::new(),
            default_choice: None,
            default_value: None,
            required: false,
            allow_empty: false,
            validator: None,
            condition: None,
        };
        self.current_field = Some(self.fields.len());
        self.fields.push(field);
        self
    }

    /// 标记当前字段为必填
    pub fn required(mut self) -> Self {
        if let Some(idx) = self.current_field {
            if let Some(field) = self.fields.get_mut(idx) {
                field.required = true;
            }
        }
        self
    }

    /// 设置当前字段的默认值
    pub fn default(mut self, value: impl Into<FieldDefaultValue>) -> Self {
        if let Some(idx) = self.current_field {
            if let Some(field) = self.fields.get_mut(idx) {
                let default_value = value.into();
                field.default_value = Some(default_value.clone());
                // 如果是选择字段，同时设置 default_choice
                if field.field_type == FormFieldType::Selection {
                    if let Some(s) = default_value.as_string() {
                        field.default_choice = Some(s);
                    }
                }
            }
        }
        self
    }

    /// 设置当前字段的验证器
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        if let Some(idx) = self.current_field {
            if let Some(field) = self.fields.get_mut(idx) {
                field.validator = Some(Arc::new(validator));
            }
        }
        self
    }

    /// 允许当前字段为空（仅对文本和密码字段有效）
    pub fn allow_empty(mut self, allow: bool) -> Self {
        if let Some(idx) = self.current_field {
            if let Some(field) = self.fields.get_mut(idx) {
                field.allow_empty = allow;
            }
        }
        self
    }
}

impl Default for FieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for FieldDefaultValue {
    fn from(s: String) -> Self {
        FieldDefaultValue::String(s)
    }
}

impl From<&str> for FieldDefaultValue {
    fn from(s: &str) -> Self {
        FieldDefaultValue::String(s.to_string())
    }
}

impl From<bool> for FieldDefaultValue {
    fn from(b: bool) -> Self {
        FieldDefaultValue::Bool(b)
    }
}
