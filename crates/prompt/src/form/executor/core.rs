//! 表单执行器核心逻辑

use crate::backend::Backend;
use crate::dialog::Result;
use crate::form::builder::FormBuilder;
use crate::form::executor::field_executors::FieldExecutors;
use crate::form::executor::separator::{print_nested_form_separator_simple, print_separator};
use crate::form::field::{FieldType, FormField};
use crate::form::types::{FormResult, FormStep, StepType};
use crate::PromptError;

/// 表单执行器
pub struct FormExecutor;

impl Default for FormExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl FormExecutor {
    /// 创建新的表单执行器
    pub fn new() -> Self {
        Self
    }

    /// 执行表单字段序列（使用默认终端后端）
    pub fn execute(&self, builder: &FormBuilder) -> Result<FormResult> {
        if builder.has_groups() {
            self.execute_groups(builder)
        } else {
            self.execute_with_level(builder, 0)
        }
    }

    /// 使用指定后端执行表单字段序列
    pub fn execute_with_backend<B: Backend>(
        &self,
        builder: &FormBuilder,
        backend: &mut B,
    ) -> Result<FormResult> {
        if builder.has_groups() {
            self.execute_groups_with_backend(builder, backend)
        } else {
            self.execute_with_level_and_backend(builder, 0, backend)
        }
    }

    /// 执行 Group 模式
    fn execute_groups(&self, builder: &FormBuilder) -> Result<FormResult> {
        let mut result = FormResult::new();
        let groups = builder.get_groups();

        for group in groups {
            if group.optional {
                let should_configure = if let Some(title) = &group.title {
                    crate::br!();
                    crate::separator!('─', 80, title);
                    if let Some(description) = &group.description {
                        crate::debug!("{}", description);
                        crate::br!();
                    }
                    crate::confirm!("Configure {}?", title)
                        .default(group.default_enabled)
                        .prompt()?
                } else {
                    group.default_enabled
                };

                if !should_configure {
                    continue;
                }
            } else {
                if let Some(title) = &group.title {
                    crate::br!();
                    crate::separator!('─', 80, title);
                }
                if let Some(description) = &group.description {
                    crate::debug!("{}", description);
                    crate::br!();
                }
            }

            for step in &group.steps {
                if self.should_execute_step(step, &result) {
                    for field in &step.fields {
                        if let Some(condition) = &field.condition {
                            if !condition(&result) {
                                continue;
                            }
                        }

                        let value = self.execute_field(field, &result, 0).map_err(|e| {
                            let collected_fields: Vec<String> =
                                result.values.keys().cloned().collect();
                            crate::error::PromptError::Terminal(format!(
                                "Failed to execute field '{}': {}. \
                                Already collected fields: {:?}.",
                                field.key, e, collected_fields
                            ))
                        })?;

                        result.set_boxed(field.key.clone(), value);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 使用指定后端执行 Group 模式
    fn execute_groups_with_backend<B: Backend>(
        &self,
        builder: &FormBuilder,
        backend: &mut B,
    ) -> Result<FormResult> {
        let mut result = FormResult::new();
        let groups = builder.get_groups();

        for group in groups {
            if group.optional {
                let should_configure = if let Some(title) = &group.title {
                    // 注意：这里的 separator 等输出暂时跳过，因为它们也需要 backend 支持
                    crate::dialog::ConfirmBuilder::new(format!("Configure {}?", title))
                        .default(group.default_enabled)
                        .prompt_with_backend(backend)?
                } else {
                    group.default_enabled
                };

                if !should_configure {
                    continue;
                }
            }

            for step in &group.steps {
                if self.should_execute_step(step, &result) {
                    for field in &step.fields {
                        if let Some(condition) = &field.condition {
                            if !condition(&result) {
                                continue;
                            }
                        }

                        let value =
                            self.execute_field_with_backend(field, &result, 0, backend)
                                .map_err(|e| {
                                    let collected_fields: Vec<String> =
                                        result.values.keys().cloned().collect();
                                    crate::error::PromptError::Terminal(format!(
                                        "Failed to execute field '{}': {}. \
                                        Already collected fields: {:?}.",
                                        field.key, e, collected_fields
                                    ))
                                })?;

                        result.set_boxed(field.key.clone(), value);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 判断步骤是否应该执行
    fn should_execute_step(&self, step: &FormStep, result: &FormResult) -> bool {
        match &step.step_type {
            StepType::Unconditional => true,
            StepType::Conditional(condition) => condition(result),
            StepType::ConditionalAll(conditions) => conditions.iter().all(|c| c(result)),
            StepType::ConditionalAny(conditions) => conditions.iter().any(|c| c(result)),
            StepType::DynamicCondition(f) => f(result),
        }
    }

    /// 执行单个字段
    fn execute_field(
        &self,
        field: &FormField,
        _current_result: &FormResult,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let executors = FieldExecutors;
        match field.field_type {
            FieldType::Confirm => executors.execute_confirm_field(field),
            FieldType::Input => executors.execute_input_field(field, false),
            FieldType::Password => executors.execute_input_field(field, true),
            FieldType::Select => executors.execute_select_field(field),
            FieldType::MultiSelect => executors.execute_multiselect_field(field),
            FieldType::Form => self.execute_nested_form(field, level),
        }
    }

    /// 使用指定后端执行单个字段
    fn execute_field_with_backend<B: Backend>(
        &self,
        field: &FormField,
        _current_result: &FormResult,
        level: usize,
        backend: &mut B,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let executors = FieldExecutors;
        match field.field_type {
            FieldType::Confirm => executors.execute_confirm_field_with_backend(field, backend),
            FieldType::Input => executors.execute_input_field_with_backend(field, false, backend),
            FieldType::Password => executors.execute_input_field_with_backend(field, true, backend),
            FieldType::Select => executors.execute_select_field_with_backend(field, backend),
            FieldType::MultiSelect => {
                executors.execute_multiselect_field_with_backend(field, backend)
            }
            FieldType::Form => self.execute_nested_form_with_backend(field, level, backend),
        }
    }

    /// 执行嵌套表单
    pub(super) fn execute_with_level(
        &self,
        builder: &FormBuilder,
        level: usize,
    ) -> Result<FormResult> {
        let title = builder.get_title();
        let is_main_form = level == 0;

        if let Some(title) = title {
            if is_main_form {
                print_separator(title, "start", is_main_form)?;
            } else {
                print_nested_form_separator_simple(title)?;
            }
        }

        let mut result = FormResult::new();
        let fields = builder.get_fields();

        for field in fields {
            if let Some(condition) = &field.condition {
                if !condition(&result) {
                    continue;
                }
            }

            let value = self.execute_field(field, &result, level)?;
            result.set_boxed(field.key.clone(), value);
        }

        if let Some(title) = title {
            if is_main_form {
                print_separator(title, "end", is_main_form)?;
            }
        }

        Ok(result)
    }

    /// 使用指定后端执行嵌套表单
    pub(super) fn execute_with_level_and_backend<B: Backend>(
        &self,
        builder: &FormBuilder,
        level: usize,
        backend: &mut B,
    ) -> Result<FormResult> {
        // 注意：separator 输出暂时跳过，需要后续支持

        let mut result = FormResult::new();
        let fields = builder.get_fields();

        for field in fields {
            if let Some(condition) = &field.condition {
                if !condition(&result) {
                    continue;
                }
            }

            let value = self.execute_field_with_backend(field, &result, level, backend)?;
            result.set_boxed(field.key.clone(), value);
        }

        Ok(result)
    }

    /// 执行嵌套表单
    fn execute_nested_form(
        &self,
        field: &FormField,
        level: usize,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let nested_form = field
            .nested_form
            .as_ref()
            .ok_or_else(|| PromptError::InvalidInput("Nested form cannot be empty".to_string()))?;

        let nested_result = self.execute_with_level(nested_form, level + 1)?;
        Ok(Box::new(nested_result))
    }

    /// 使用指定后端执行嵌套表单
    fn execute_nested_form_with_backend<B: Backend>(
        &self,
        field: &FormField,
        level: usize,
        backend: &mut B,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let nested_form = field
            .nested_form
            .as_ref()
            .ok_or_else(|| PromptError::InvalidInput("Nested form cannot be empty".to_string()))?;

        let nested_result = self.execute_with_level_and_backend(nested_form, level + 1, backend)?;
        Ok(Box::new(nested_result))
    }
}
