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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::form::{ConfirmFormField, InputFormField, SelectFormField};

    #[test]
    fn test_form_executor_new() {
        let executor = FormExecutor::new();
        // Just verify it can be created
        assert!(std::mem::size_of_val(&executor) >= 0);
    }

    #[test]
    fn test_form_executor_default() {
        let executor = FormExecutor::default();
        assert!(std::mem::size_of_val(&executor) >= 0);
    }

    #[test]
    fn test_execute_single_input_field() {
        let events = [
            MockBackend::type_string("test_value"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_input(InputFormField::new("name", "Enter name"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_string("name"), "test_value");
    }

    #[test]
    fn test_execute_single_confirm_field() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = vec![Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE))];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_confirm(ConfirmFormField::new("proceed", "Continue?"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert!(form_result.get_bool("proceed"));
    }

    #[test]
    fn test_execute_single_select_field() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_select(SelectFormField::new(
                "choice",
                "Select option",
                vec!["Option A".to_string(), "Option B".to_string(), "Option C".to_string()],
            ));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_int("choice"), 0); // First option selected
    }

    #[test]
    fn test_execute_multiple_fields() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = [
            // Input field: "Alice"
            MockBackend::type_string("Alice"),
            vec![MockBackend::press_enter()],
            // Confirm field: 'y'
            vec![Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE))],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_input(InputFormField::new("name", "Enter name"))
            .add_confirm(ConfirmFormField::new("agree", "Do you agree?"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_string("name"), "Alice");
        assert!(form_result.get_bool("agree"));
    }

    #[test]
    fn test_execute_with_default_value() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_input(InputFormField::new("name", "Enter name").default("DefaultName"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_string("name"), "DefaultName");
    }

    #[test]
    fn test_execute_conditional_field_executed() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = [
            // First confirm: 'y' (enable)
            vec![Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE))],
            // Second input: "conditional_value"
            MockBackend::type_string("conditional_value"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_confirm(ConfirmFormField::new("enable", "Enable feature?"))
            .add_input(
                InputFormField::new("feature_name", "Feature name")
                    .condition(Box::new(|result| result.get_bool("enable"))),
            );

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert!(form_result.get_bool("enable"));
        assert_eq!(form_result.get_string("feature_name"), "conditional_value");
    }

    #[test]
    fn test_execute_conditional_field_skipped() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = [
            // First confirm: 'n' (disable)
            vec![Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE))],
            // Second input should be skipped, so no events needed
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_confirm(ConfirmFormField::new("enable", "Enable feature?"))
            .add_input(
                InputFormField::new("feature_name", "Feature name")
                    .condition(Box::new(|result| result.get_bool("enable"))),
            );

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert!(!form_result.get_bool("enable"));
        // Conditional field was skipped, so its value should be empty/default
        assert_eq!(form_result.get_string("feature_name"), "");
    }

    #[test]
    fn test_step_type_unconditional() {
        let executor = FormExecutor::new();
        let result = FormResult::new();
        let step = FormStep {
            step_type: StepType::Unconditional,
            fields: vec![],
        };
        assert!(executor.should_execute_step(&step, &result));
    }

    #[test]
    fn test_step_type_conditional() {
        let executor = FormExecutor::new();

        let mut result = FormResult::new();
        result.set("enabled".to_string(), true);

        let step = FormStep {
            step_type: StepType::Conditional(Box::new(|r| r.get_bool("enabled"))),
            fields: vec![],
        };
        assert!(executor.should_execute_step(&step, &result));

        result.set("enabled".to_string(), false);
        let step2 = FormStep {
            step_type: StepType::Conditional(Box::new(|r| r.get_bool("enabled"))),
            fields: vec![],
        };
        assert!(!executor.should_execute_step(&step2, &result));
    }

    #[test]
    fn test_step_type_conditional_all() {
        let executor = FormExecutor::new();

        let mut result = FormResult::new();
        result.set("a".to_string(), true);
        result.set("b".to_string(), true);

        let step = FormStep {
            step_type: StepType::ConditionalAll(vec![
                Box::new(|r| r.get_bool("a")),
                Box::new(|r| r.get_bool("b")),
            ]),
            fields: vec![],
        };
        assert!(executor.should_execute_step(&step, &result));

        result.set("b".to_string(), false);
        let step2 = FormStep {
            step_type: StepType::ConditionalAll(vec![
                Box::new(|r| r.get_bool("a")),
                Box::new(|r| r.get_bool("b")),
            ]),
            fields: vec![],
        };
        assert!(!executor.should_execute_step(&step2, &result));
    }

    #[test]
    fn test_step_type_conditional_any() {
        let executor = FormExecutor::new();

        let mut result = FormResult::new();
        result.set("a".to_string(), false);
        result.set("b".to_string(), true);

        let step = FormStep {
            step_type: StepType::ConditionalAny(vec![
                Box::new(|r| r.get_bool("a")),
                Box::new(|r| r.get_bool("b")),
            ]),
            fields: vec![],
        };
        assert!(executor.should_execute_step(&step, &result));

        result.set("a".to_string(), false);
        result.set("b".to_string(), false);
        let step2 = FormStep {
            step_type: StepType::ConditionalAny(vec![
                Box::new(|r| r.get_bool("a")),
                Box::new(|r| r.get_bool("b")),
            ]),
            fields: vec![],
        };
        assert!(!executor.should_execute_step(&step2, &result));
    }

    #[test]
    fn test_step_type_dynamic_condition() {
        let executor = FormExecutor::new();

        let mut result = FormResult::new();
        result.set("count".to_string(), 5usize);

        let step = FormStep {
            step_type: StepType::DynamicCondition(Box::new(|r| r.get_int("count") > 3)),
            fields: vec![],
        };
        assert!(executor.should_execute_step(&step, &result));

        result.set("count".to_string(), 2usize);
        let step2 = FormStep {
            step_type: StepType::DynamicCondition(Box::new(|r| r.get_int("count") > 3)),
            fields: vec![],
        };
        assert!(!executor.should_execute_step(&step2, &result));
    }
}
