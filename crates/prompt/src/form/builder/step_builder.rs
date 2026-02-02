//! Step 构建器，用于在 Step 内构建字段

use crate::form::field::{
    ConfirmFormField, FieldType, FormField, InputFormField, MultiSelectFormField, NestedFormField,
    PasswordFormField, SelectFormField,
};

/// Step 构建器
///
/// 用于在 Step 内构建字段，提供便捷的方法来添加各种类型的字段。
pub struct StepBuilder {
    fields: Vec<FormField>,
}

impl StepBuilder {
    /// 创建新的 Step 构建器
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// 添加确认字段
    pub fn add_confirm(mut self, config: ConfirmFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Confirm,
            prompt: config.prompt,
            default_value: Some(Box::new(config.default_value)),
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加输入字段
    pub fn add_input(mut self, config: InputFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Input,
            prompt: config.prompt,
            default_value: Some(Box::new(config.default_value)),
            validator: config.validator,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加密码字段
    pub fn add_password(mut self, config: PasswordFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Password,
            prompt: config.prompt,
            default_value: Some(Box::new(config.default_value)),
            validator: config.validator,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加单选字段
    pub fn add_select(mut self, config: SelectFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Select,
            prompt: config.prompt,
            default_value: None,
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: config.options,
            default_index: Some(config.default_index),
            default_selected: Vec::new(),
        });
        self
    }

    /// 添加多选字段
    pub fn add_multiselect(mut self, config: MultiSelectFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::MultiSelect,
            prompt: config.prompt,
            default_value: None,
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: None,
            options: config.options,
            default_index: None,
            default_selected: config.default_selected,
        });
        self
    }

    /// 添加嵌套表单字段
    pub fn add_form(mut self, config: NestedFormField) -> Self {
        self.fields.push(FormField {
            key: config.key,
            field_type: FieldType::Form,
            prompt: config.prompt,
            default_value: None,
            validator: None,
            condition: config.condition,
            result_title: config.result_title,
            nested_form: Some(config.nested_form),
            options: Vec::new(),
            default_index: None,
            default_selected: Vec::new(),
        });
        self
    }

    /// 获取字段列表（内部使用）
    pub(crate) fn into_fields(self) -> Vec<FormField> {
        self.fields
    }
}

impl Default for StepBuilder {
    fn default() -> Self {
        Self::new()
    }
}
