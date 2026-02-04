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

                        let value = self
                            .execute_field_with_backend(field, &result, 0, backend)
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
        // 验证 executor 是零大小类型
        assert_eq!(std::mem::size_of_val(&executor), 0);
    }

    #[test]
    fn test_form_executor_default() {
        // 验证 Default trait 实现正确
        let _: FormExecutor = Default::default();
        // FormExecutor 是单元结构体，大小为 0
        assert_eq!(std::mem::size_of::<FormExecutor>(), 0);
    }

    #[test]
    fn test_execute_single_input_field() {
        let events = [
            MockBackend::type_string("test_value"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_input(InputFormField::new("name", "Enter name"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_string("name"), "test_value");
    }

    #[test]
    fn test_execute_single_confirm_field() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_confirm(ConfirmFormField::new("proceed", "Continue?"));

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

        let builder = FormBuilder::new().add_select(SelectFormField::new(
            "choice",
            "Select option",
            vec![
                "Option A".to_string(),
                "Option B".to_string(),
                "Option C".to_string(),
            ],
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
            vec![Event::Key(KeyEvent::new(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            ))],
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
            vec![Event::Key(KeyEvent::new(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            ))],
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
            vec![Event::Key(KeyEvent::new(
                KeyCode::Char('n'),
                KeyModifiers::NONE,
            ))],
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

    // ========================================================================
    // 嵌套表单测试
    // ========================================================================

    #[test]
    fn test_execute_nested_form() {
        use crate::form::NestedFormField;

        let events = [
            // 嵌套表单的第一个字段
            MockBackend::type_string("nested_value"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let nested_builder =
            FormBuilder::new().add_input(InputFormField::new("nested_name", "Nested name"));

        let builder = FormBuilder::new().add_form(NestedFormField::new(
            "nested",
            "Nested form",
            nested_builder,
        ));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        let nested_result = form_result.get_form("nested").unwrap();
        assert_eq!(nested_result.get_string("nested_name"), "nested_value");
    }

    #[test]
    fn test_execute_deeply_nested_form() {
        use crate::form::NestedFormField;

        let events = [
            // 第一层嵌套表单的字段
            MockBackend::type_string("level1_value"),
            vec![MockBackend::press_enter()],
            // 第二层嵌套表单的字段
            MockBackend::type_string("level2_value"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let level2_builder =
            FormBuilder::new().add_input(InputFormField::new("level2_name", "Level 2 name"));

        let level1_builder = FormBuilder::new()
            .add_input(InputFormField::new("level1_name", "Level 1 name"))
            .add_form(NestedFormField::new("level2", "Level 2", level2_builder));

        let builder =
            FormBuilder::new().add_form(NestedFormField::new("level1", "Level 1", level1_builder));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        let level1 = form_result.get_form("level1").unwrap();
        assert_eq!(level1.get_string("level1_name"), "level1_value");
        let level2 = level1.get_form("level2").unwrap();
        assert_eq!(level2.get_string("level2_name"), "level2_value");
    }

    // ========================================================================
    // Password 字段测试
    // ========================================================================

    #[test]
    fn test_execute_password_field() {
        use crate::form::PasswordFormField;

        let events = [
            MockBackend::type_string("secret123"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder =
            FormBuilder::new().add_password(PasswordFormField::new("password", "Enter password"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_string("password"), "secret123");
    }

    // ========================================================================
    // MultiSelect 字段测试
    // ========================================================================

    #[test]
    fn test_execute_multiselect_field() {
        use crate::form::MultiSelectFormField;
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)), // Toggle first
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)), // Toggle second
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_multiselect(MultiSelectFormField::new(
            "choices",
            "Select options",
            vec![
                "Option A".to_string(),
                "Option B".to_string(),
                "Option C".to_string(),
            ],
        ));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        let indices = form_result.get_int_slice("choices");
        assert_eq!(indices, vec![0, 1]);
    }

    // ========================================================================
    // 取消操作测试
    // ========================================================================

    #[test]
    fn test_execute_cancel_input_field() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_input(InputFormField::new("name", "Enter name"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_cancel_confirm_field() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_confirm(ConfirmFormField::new("agree", "Agree?"));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_cancel_select_field() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_select(SelectFormField::new(
            "choice",
            "Select",
            vec!["A".to_string(), "B".to_string()],
        ));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_err());
    }

    // ========================================================================
    // 多字段组合测试
    // ========================================================================

    #[test]
    fn test_execute_complex_form() {
        use crate::form::PasswordFormField;
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = [
            // Input: username
            MockBackend::type_string("john_doe"),
            vec![MockBackend::press_enter()],
            // Password: secret
            MockBackend::type_string("mypassword"),
            vec![MockBackend::press_enter()],
            // Confirm: yes
            vec![Event::Key(KeyEvent::new(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            ))],
            // Select: second option (index 1)
            vec![
                Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
                MockBackend::press_enter(),
            ],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_input(InputFormField::new("username", "Username"))
            .add_password(PasswordFormField::new("password", "Password"))
            .add_confirm(ConfirmFormField::new("remember", "Remember me?"))
            .add_select(SelectFormField::new(
                "role",
                "Select role",
                vec!["Admin".to_string(), "User".to_string(), "Guest".to_string()],
            ));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_string("username"), "john_doe");
        assert_eq!(form_result.get_string("password"), "mypassword");
        assert!(form_result.get_bool("remember"));
        assert_eq!(form_result.get_int("role"), 1);
    }

    // ========================================================================
    // 边界条件测试
    // ========================================================================

    #[test]
    fn test_execute_empty_form() {
        let mut backend = MockBackend::new();

        let builder = FormBuilder::new();
        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert!(form_result.values.is_empty());
    }

    #[test]
    fn test_execute_all_conditions_false() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = vec![
            // First confirm: 'n' (disable all)
            Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
        ];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new()
            .add_confirm(ConfirmFormField::new("enable", "Enable?"))
            .add_input(
                InputFormField::new("field1", "Field 1")
                    .condition(Box::new(|r| r.get_bool("enable"))),
            )
            .add_input(
                InputFormField::new("field2", "Field 2")
                    .condition(Box::new(|r| r.get_bool("enable"))),
            );

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert!(!form_result.get_bool("enable"));
        // Conditional fields should be skipped
        assert_eq!(form_result.get_string("field1"), "");
        assert_eq!(form_result.get_string("field2"), "");
    }

    #[test]
    fn test_execute_with_select_navigation() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let events = vec![
            // Navigate: down, down, up, enter (select second option)
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_select(SelectFormField::new(
            "option",
            "Select",
            vec![
                "First".to_string(),
                "Second".to_string(),
                "Third".to_string(),
            ],
        ));

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_int("option"), 1); // Second option
    }

    #[test]
    fn test_execute_with_default_select() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let builder = FormBuilder::new().add_select(
            SelectFormField::new(
                "option",
                "Select",
                vec![
                    "First".to_string(),
                    "Second".to_string(),
                    "Third".to_string(),
                ],
            )
            .default(2),
        );

        let executor = FormExecutor::new();
        let result = executor.execute_with_backend(&builder, &mut backend);

        assert!(result.is_ok());
        let form_result = result.unwrap();
        assert_eq!(form_result.get_int("option"), 2); // Third option (default)
    }
}
