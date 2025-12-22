//! 表单执行器，负责执行表单并收集用户输入

use color_eyre::{eyre::eyre, Result};
use dialoguer::Password;
use std::collections::HashMap;

use crate::base::dialog::form::builder::FormBuilder;
use crate::base::dialog::form::types::{
    Condition, ConditionOperator, ConditionValue, ConditionalGroup, FormField, FormFieldType,
    FormResult,
};
use crate::base::dialog::{ConfirmDialog, InputDialog, SelectDialog};

impl FormBuilder {
    /// 执行表单并收集用户输入
    ///
    /// # 返回
    ///
    /// 返回 `FormResult`，包含所有字段的值
    ///
    /// # 错误
    ///
    /// 如果用户取消或验证失败，返回错误
    pub fn run(self) -> Result<FormResult> {
        // 存储所有字段的值
        let mut field_values: HashMap<String, String> = HashMap::new();

        // 处理所有常规字段
        for field in &self.fields {
            if self.should_ask_field(field, &field_values) {
                self.ask_field(field, &mut field_values)?;
            }
        }

        // 处理条件组
        for group in &self.conditional_groups {
            self.process_conditional_group(group, &mut field_values)?;
        }

        // 构建结果
        Ok(FormResult {
            values: field_values,
        })
    }

    /// 询问单个字段
    fn ask_field(&self, field: &FormField, field_values: &mut HashMap<String, String>) -> Result<()> {
        match field.field_type {
            FormFieldType::Text => {
                let mut dialog = InputDialog::new(&field.message);

                // 设置默认值
                if let Some(ref default_value) = field.default_value {
                    if let Some(default_str) = default_value.as_string() {
                        dialog = dialog.with_default(default_str);
                    }
                }

                // 设置验证器
                if let Some(ref validator) = field.validator {
                    let validator_clone = validator.clone();
                    dialog = dialog.with_validator(move |input: &str| {
                        // 如果必填且为空，返回错误
                        if field.required && input.trim().is_empty() {
                            return Err(format!("Field '{}' is required", field.name));
                        }
                        // 调用自定义验证器
                        validator_clone(input)
                    });
                } else if field.required {
                    // 如果没有验证器但必填，添加默认验证
                    dialog = dialog.with_validator(move |input: &str| {
                        if input.trim().is_empty() {
                            Err(format!("Field '{}' is required", field.name))
                        } else {
                            Ok(())
                        }
                    });
                } else {
                    // 允许空值
                    dialog = dialog.allow_empty(true);
                }

                let value = dialog.prompt()?;
                field_values.insert(field.name.clone(), value);
            }
            FormFieldType::Password => {
                let password = Password::new()
                    .with_prompt(&field.message)
                    .interact()
                    .map_err(|e| eyre!("Failed to get password: {}", e))?;

                // 验证必填
                if field.required && password.is_empty() {
                    color_eyre::eyre::bail!("Field '{}' is required", field.name);
                }

                // 验证器
                if let Some(ref validator) = field.validator {
                    validator(&password)?;
                }

                field_values.insert(field.name.clone(), password);
            }
            FormFieldType::Selection => {
                let mut dialog = SelectDialog::new(&field.message, field.choices.clone());

                // 设置默认选项
                if let Some(ref default_choice) = field.default_choice {
                    if let Some(idx) = field.choices.iter().position(|c| c == default_choice) {
                        dialog = dialog.with_default(idx);
                    }
                } else if let Some(ref default_value) = field.default_value {
                    if let Some(default_str) = default_value.as_string() {
                        if let Some(idx) = field.choices.iter().position(|c| c == &default_str) {
                            dialog = dialog.with_default(idx);
                        }
                    }
                }

                let value = dialog.prompt()?;
                field_values.insert(field.name.clone(), value);
            }
            FormFieldType::Confirmation => {
                let mut dialog = ConfirmDialog::new(&field.message);

                // 设置默认值
                if let Some(ref default_value) = field.default_value {
                    if let Some(default_bool) = default_value.as_bool() {
                        dialog = dialog.with_default(default_bool);
                    }
                }

                let confirmed = dialog.prompt()?;
                // 将布尔值转换为字符串（"yes" 或 "no"）
                let value = if confirmed { "yes".to_string() } else { "no".to_string() };
                field_values.insert(field.name.clone(), value);
            }
        }

        Ok(())
    }

    /// 处理条件组
    fn process_conditional_group(
        &self,
        group: &ConditionalGroup,
        field_values: &mut HashMap<String, String>,
    ) -> Result<()> {
        // 检查条件是否满足
        if !self.should_ask_field_by_condition(&group.condition, field_values) {
            return Ok(());
        }

        // 处理组内的字段
        for field in &group.fields {
            if self.should_ask_field(field, field_values) {
                self.ask_field(field, field_values)?;
            }
        }

        // 递归处理嵌套组
        for nested_group in &group.nested_groups {
            self.process_conditional_group(nested_group, field_values)?;
        }

        Ok(())
    }

    /// 判断是否应该询问字段（基于字段的条件）
    fn should_ask_field(&self, field: &FormField, field_values: &HashMap<String, String>) -> bool {
        if let Some(ref condition) = field.condition {
            self.should_ask_field_by_condition(condition, field_values)
        } else {
            true
        }
    }

    /// 判断是否应该询问字段（基于条件）
    fn should_ask_field_by_condition(
        &self,
        condition: &Condition,
        field_values: &HashMap<String, String>,
    ) -> bool {
        let field_value = match field_values.get(&condition.field_name) {
            Some(v) => v.clone(),
            None => return false, // 字段还未收集
        };

        self.evaluate_condition(condition, &field_value)
    }

    /// 评估条件
    fn evaluate_condition(
        &self,
        condition: &crate::base::dialog::form::types::Condition,
        field_value: &str,
    ) -> bool {
        match condition.operator {
            ConditionOperator::Equals => {
                match &condition.value {
                    ConditionValue::Single(expected) => {
                        field_value.eq_ignore_ascii_case(expected)
                    }
                    ConditionValue::Multiple(_) => false,
                }
            }
            ConditionOperator::NotEquals => {
                match &condition.value {
                    ConditionValue::Single(expected) => {
                        !field_value.eq_ignore_ascii_case(expected)
                    }
                    ConditionValue::Multiple(_) => false,
                }
            }
            ConditionOperator::In => {
                match &condition.value {
                    ConditionValue::Single(_) => false,
                    ConditionValue::Multiple(values) => {
                        values.iter().any(|v| field_value.eq_ignore_ascii_case(v))
                    }
                }
            }
            ConditionOperator::NotIn => {
                match &condition.value {
                    ConditionValue::Single(_) => true,
                    ConditionValue::Multiple(values) => {
                        !values.iter().any(|v| field_value.eq_ignore_ascii_case(v))
                    }
                }
            }
        }
    }
}
