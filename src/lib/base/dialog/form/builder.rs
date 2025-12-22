//! FormBuilder 用于构建交互式表单

use crate::base::dialog::form::types::{
    Condition, ConditionOperator, ConditionValue, ConditionalGroup, FieldDefaultValue, FormField,
    FormFieldType,
};

/// 表单构建器
#[derive(Debug, Clone)]
pub struct FormBuilder {
    /// 表单字段列表
    pub fields: Vec<FormField>,
    /// 条件组列表
    pub conditional_groups: Vec<ConditionalGroup>,
    /// 当前正在构建的字段
    pub current_field: Option<usize>,
}

impl FormBuilder {
    /// 创建新的表单构建器
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::dialog::form::FormBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = FormBuilder::new()
    ///     .add_text("name", "Enter your name")
    ///         .required()
    ///         .default("John Doe")
    ///     .add_password("password", "Enter password")
    ///         .required()
    ///     .run()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            conditional_groups: Vec::new(),
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
        use crate::base::dialog::types::ValidatorFn;
        use std::sync::Arc;

        if let Some(idx) = self.current_field {
            if let Some(field) = self.fields.get_mut(idx) {
                field.validator = Some(Arc::new(validator));
            }
        }
        self
    }

    /// 添加条件组：当指定字段满足条件时，显示条件组中的字段
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::dialog::form::FormBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = FormBuilder::new()
    ///     .add_selection("provider", "Select provider", vec!["openai".to_string(), "custom".to_string()])
    ///     .when("provider", "custom", |f| {
    ///         f.add_text("url", "Enter URL").required()
    ///     })
    ///     .run()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn when<F>(mut self, field_name: impl Into<String>, value: impl Into<String>, builder: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let condition = Condition {
            field_name: field_name.into(),
            operator: ConditionOperator::Equals,
            value: ConditionValue::single(value.into()),
        };

        // 创建临时构建器来收集条件字段
        let temp_builder = Self {
            fields: Vec::new(),
            conditional_groups: Vec::new(),
            current_field: None,
        };

        // 使用 builder 构建条件字段
        let built = builder(temp_builder);

        // 创建条件组
        let group = ConditionalGroup {
            condition,
            fields: built.fields,
            nested_groups: built.conditional_groups,
        };

        self.conditional_groups.push(group);
        self
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// 实现 From trait 以便于设置默认值
impl From<String> for FieldDefaultValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for FieldDefaultValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for FieldDefaultValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
