//! 字段执行器

use crate::backend::Backend;
use crate::dialog::Result;
use crate::form::field::FormField;
use std::sync::Arc;

/// 字段执行器
pub(super) struct FieldExecutors;

impl FieldExecutors {
    /// 执行确认字段
    pub(super) fn execute_confirm_field(
        &self,
        field: &FormField,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<bool>())
            .copied()
            .unwrap_or(false);

        let mut builder = crate::dialog::ConfirmBuilder::new(&field.prompt).default(default_value);

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        let confirmed = builder.prompt()?;
        Ok(Box::new(confirmed))
    }

    /// 使用指定后端执行确认字段
    pub(super) fn execute_confirm_field_with_backend<B: Backend>(
        &self,
        field: &FormField,
        backend: &mut B,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<bool>())
            .copied()
            .unwrap_or(false);

        let mut builder = crate::dialog::ConfirmBuilder::new(&field.prompt).default(default_value);

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        let confirmed = builder.prompt_with_backend(backend)?;
        Ok(Box::new(confirmed))
    }

    /// 执行输入字段（Input 或 Password）
    pub(super) fn execute_input_field(
        &self,
        field: &FormField,
        is_password: bool,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<String>())
            .cloned()
            .unwrap_or_default();

        let mut builder = if is_password {
            crate::dialog::InputBuilder::new(&field.prompt).password()
        } else {
            crate::dialog::InputBuilder::new(&field.prompt)
        };

        if !default_value.is_empty() {
            builder = builder.default(default_value);
        }

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        if let Some(validator) = &field.validator {
            builder = builder.validator(ArcValidatorAdapter(Arc::clone(validator)));
        }

        let value = builder.prompt()?;
        Ok(Box::new(value))
    }

    /// 使用指定后端执行输入字段
    pub(super) fn execute_input_field_with_backend<B: Backend>(
        &self,
        field: &FormField,
        is_password: bool,
        backend: &mut B,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_value = field
            .default_value
            .as_ref()
            .and_then(|v| v.downcast_ref::<String>())
            .cloned()
            .unwrap_or_default();

        let mut builder = if is_password {
            crate::dialog::InputBuilder::new(&field.prompt).password()
        } else {
            crate::dialog::InputBuilder::new(&field.prompt)
        };

        if !default_value.is_empty() {
            builder = builder.default(default_value);
        }

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        if let Some(validator) = &field.validator {
            builder = builder.validator(ArcValidatorAdapter(Arc::clone(validator)));
        }

        let value = builder.prompt_with_backend(backend)?;
        Ok(Box::new(value))
    }

    /// 执行选择字段
    pub(super) fn execute_select_field(
        &self,
        field: &FormField,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_index = field.default_index.unwrap_or(0);
        let mut builder =
            crate::dialog::SelectBuilder::new(field.prompt.clone(), field.options.clone())
                .default(default_index);

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        let selected = builder.prompt()?;

        let index = field
            .options
            .iter()
            .enumerate()
            .find(|(_, opt)| *opt == &selected)
            .map(|(idx, _)| idx)
            .ok_or_else(|| {
                crate::error::PromptError::InvalidInput(
                    "Selected option not found in options list".to_string(),
                )
            })?;

        Ok(Box::new(index))
    }

    /// 使用指定后端执行选择字段
    pub(super) fn execute_select_field_with_backend<B: Backend>(
        &self,
        field: &FormField,
        backend: &mut B,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let default_index = field.default_index.unwrap_or(0);
        let mut builder =
            crate::dialog::SelectBuilder::new(field.prompt.clone(), field.options.clone())
                .default(default_index);

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        let selected = builder.prompt_with_backend(backend)?;

        let index = field
            .options
            .iter()
            .enumerate()
            .find(|(_, opt)| *opt == &selected)
            .map(|(idx, _)| idx)
            .ok_or_else(|| {
                crate::error::PromptError::InvalidInput(
                    "Selected option not found in options list".to_string(),
                )
            })?;

        Ok(Box::new(index))
    }

    /// 执行多选字段
    pub(super) fn execute_multiselect_field(
        &self,
        field: &FormField,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let mut builder =
            crate::dialog::MultiSelectBuilder::new(&field.prompt, field.options.clone())
                .default(field.default_selected.clone());

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        let selected = builder.prompt()?;

        let indices: Vec<usize> = selected
            .iter()
            .filter_map(|item| {
                field
                    .options
                    .iter()
                    .enumerate()
                    .find(|(_, opt)| *opt == item)
                    .map(|(idx, _)| idx)
            })
            .collect();

        Ok(Box::new(indices))
    }

    /// 使用指定后端执行多选字段
    pub(super) fn execute_multiselect_field_with_backend<B: Backend>(
        &self,
        field: &FormField,
        backend: &mut B,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let mut builder =
            crate::dialog::MultiSelectBuilder::new(&field.prompt, field.options.clone())
                .default(field.default_selected.clone());

        if let Some(ref result_title) = field.result_title {
            builder = builder.result_title(result_title);
        }

        let selected = builder.prompt_with_backend(backend)?;

        let indices: Vec<usize> = selected
            .iter()
            .filter_map(|item| {
                field
                    .options
                    .iter()
                    .enumerate()
                    .find(|(_, opt)| *opt == item)
                    .map(|(idx, _)| idx)
            })
            .collect();

        Ok(Box::new(indices))
    }
}

/// Arc 验证器适配器
struct ArcValidatorAdapter(Arc<dyn crate::dialog::Validator + Send + Sync>);

impl crate::dialog::Validator for ArcValidatorAdapter {
    fn validate(&self, input: &str) -> crate::dialog::ValidationResult {
        self.0.validate(input)
    }
}
