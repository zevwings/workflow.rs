//! 字段执行器

use std::sync::Arc;

use crate::{backend::Backend, dialog::Result, form::field::FormField};

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

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::{
        backend::MockBackend,
        dialog::{ValidationResult, Validator},
        form::field::FieldType,
    };

    /// 创建测试用的 FormField
    fn create_test_field(field_type: FieldType, key: &str, prompt: &str) -> FormField {
        FormField {
            key: key.to_string(),
            field_type,
            prompt: prompt.to_string(),
            default_value: None,
            validator: None,
            condition: None,
            result_title: None,
            nested_form: None,
            options: vec![],
            default_index: None,
            default_selected: vec![],
        }
    }

    // ========================================================================
    // Confirm 字段测试
    // ========================================================================

    #[test]
    fn test_execute_confirm_field_with_backend_yes() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let field = create_test_field(FieldType::Confirm, "agree", "Do you agree?");
        let executors = FieldExecutors;

        let result = executors.execute_confirm_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let confirmed = value.downcast_ref::<bool>().unwrap();
        assert!(*confirmed);
    }

    #[test]
    fn test_execute_confirm_field_with_backend_no() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let field = create_test_field(FieldType::Confirm, "agree", "Do you agree?");
        let executors = FieldExecutors;

        let result = executors.execute_confirm_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let confirmed = value.downcast_ref::<bool>().unwrap();
        assert!(!*confirmed);
    }

    #[test]
    fn test_execute_confirm_field_with_default_true() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Confirm, "agree", "Do you agree?");
        field.default_value = Some(Box::new(true));
        let executors = FieldExecutors;

        let result = executors.execute_confirm_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let confirmed = value.downcast_ref::<bool>().unwrap();
        assert!(*confirmed);
    }

    #[test]
    fn test_execute_confirm_field_with_default_false() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Confirm, "agree", "Do you agree?");
        field.default_value = Some(Box::new(false));
        let executors = FieldExecutors;

        let result = executors.execute_confirm_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let confirmed = value.downcast_ref::<bool>().unwrap();
        assert!(!*confirmed);
    }

    #[test]
    fn test_execute_confirm_field_with_result_title() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Confirm, "agree", "Do you agree?");
        field.result_title = Some("Agreement".to_string());
        let executors = FieldExecutors;

        let result = executors.execute_confirm_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Input 字段测试
    // ========================================================================

    #[test]
    fn test_execute_input_field_with_backend() {
        let events = [
            MockBackend::type_string("test_input"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let field = create_test_field(FieldType::Input, "name", "Enter your name");
        let executors = FieldExecutors;

        let result = executors.execute_input_field_with_backend(&field, false, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let input = value.downcast_ref::<String>().unwrap();
        assert_eq!(input, "test_input");
    }

    #[test]
    fn test_execute_input_field_with_default() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Input, "name", "Enter your name");
        field.default_value = Some(Box::new("DefaultName".to_string()));
        let executors = FieldExecutors;

        let result = executors.execute_input_field_with_backend(&field, false, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let input = value.downcast_ref::<String>().unwrap();
        assert_eq!(input, "DefaultName");
    }

    #[test]
    fn test_execute_input_field_with_result_title() {
        let events = [
            MockBackend::type_string("value"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Input, "name", "Enter your name");
        field.result_title = Some("Name".to_string());
        let executors = FieldExecutors;

        let result = executors.execute_input_field_with_backend(&field, false, &mut backend);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_password_field_with_backend() {
        let events = [
            MockBackend::type_string("secret123"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let field = create_test_field(FieldType::Password, "password", "Enter password");
        let executors = FieldExecutors;

        let result = executors.execute_input_field_with_backend(&field, true, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let password = value.downcast_ref::<String>().unwrap();
        assert_eq!(password, "secret123");
    }

    #[test]
    fn test_execute_input_field_with_validator() {
        // 自定义验证器
        struct MinLengthValidator(usize);
        impl Validator for MinLengthValidator {
            fn validate(&self, input: &str) -> ValidationResult {
                if input.len() >= self.0 {
                    Ok(())
                } else {
                    Err(format!("Minimum {} characters required", self.0))
                }
            }
        }

        let events = [
            MockBackend::type_string("ab"),   // 太短
            vec![MockBackend::press_enter()], // 尝试提交
            MockBackend::type_string("cde"),  // 添加更多字符
            vec![MockBackend::press_enter()], // 再次提交
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Input, "name", "Enter name (min 5 chars)");
        field.validator = Some(Arc::new(MinLengthValidator(5)));
        let executors = FieldExecutors;

        let result = executors.execute_input_field_with_backend(&field, false, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let input = value.downcast_ref::<String>().unwrap();
        assert_eq!(input, "abcde");
    }

    // ========================================================================
    // Select 字段测试
    // ========================================================================

    #[test]
    fn test_execute_select_field_with_backend_first_option() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Select, "choice", "Select an option");
        field.options = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ];
        let executors = FieldExecutors;

        let result = executors.execute_select_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let index = value.downcast_ref::<usize>().unwrap();
        assert_eq!(*index, 0);
    }

    #[test]
    fn test_execute_select_field_with_backend_navigate_down() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Select, "choice", "Select an option");
        field.options = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ];
        let executors = FieldExecutors;

        let result = executors.execute_select_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let index = value.downcast_ref::<usize>().unwrap();
        assert_eq!(*index, 1);
    }

    #[test]
    fn test_execute_select_field_with_default_index() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Select, "choice", "Select an option");
        field.options = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ];
        field.default_index = Some(2);
        let executors = FieldExecutors;

        let result = executors.execute_select_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let index = value.downcast_ref::<usize>().unwrap();
        assert_eq!(*index, 2);
    }

    #[test]
    fn test_execute_select_field_with_result_title() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Select, "choice", "Select an option");
        field.options = vec!["Option A".to_string(), "Option B".to_string()];
        field.result_title = Some("Choice".to_string());
        let executors = FieldExecutors;

        let result = executors.execute_select_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());
    }

    // ========================================================================
    // MultiSelect 字段测试
    // ========================================================================

    #[test]
    fn test_execute_multiselect_field_with_backend_select_first() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)), // Toggle first
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::MultiSelect, "choices", "Select options");
        field.options = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ];
        let executors = FieldExecutors;

        let result = executors.execute_multiselect_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let indices = value.downcast_ref::<Vec<usize>>().unwrap();
        assert_eq!(indices, &[0]);
    }

    #[test]
    fn test_execute_multiselect_field_with_backend_select_multiple() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)), // Toggle first
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)), // Toggle third
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::MultiSelect, "choices", "Select options");
        field.options = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ];
        let executors = FieldExecutors;

        let result = executors.execute_multiselect_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let indices = value.downcast_ref::<Vec<usize>>().unwrap();
        assert_eq!(indices, &[0, 2]);
    }

    #[test]
    fn test_execute_multiselect_field_with_default_selected() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::MultiSelect, "choices", "Select options");
        field.options = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ];
        field.default_selected = vec![0, 2];
        let executors = FieldExecutors;

        let result = executors.execute_multiselect_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let indices = value.downcast_ref::<Vec<usize>>().unwrap();
        assert_eq!(indices, &[0, 2]);
    }

    #[test]
    fn test_execute_multiselect_field_with_result_title() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::MultiSelect, "choices", "Select options");
        field.options = vec!["Option A".to_string(), "Option B".to_string()];
        field.result_title = Some("Choices".to_string());
        let executors = FieldExecutors;

        let result = executors.execute_multiselect_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_multiselect_field_select_none() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::MultiSelect, "choices", "Select options");
        field.options = vec!["Option A".to_string(), "Option B".to_string()];
        let executors = FieldExecutors;

        let result = executors.execute_multiselect_field_with_backend(&field, &mut backend);
        assert!(result.is_ok());

        let value = result.unwrap();
        let indices = value.downcast_ref::<Vec<usize>>().unwrap();
        assert!(indices.is_empty());
    }

    // ========================================================================
    // ArcValidatorAdapter 测试
    // ========================================================================

    #[test]
    fn test_arc_validator_adapter() {
        struct TestValidator;
        impl Validator for TestValidator {
            fn validate(&self, input: &str) -> ValidationResult {
                if input == "valid" {
                    Ok(())
                } else {
                    Err("Invalid input".to_string())
                }
            }
        }

        let validator: Arc<dyn Validator + Send + Sync> = Arc::new(TestValidator);
        let adapter = ArcValidatorAdapter(validator);

        assert!(adapter.validate("valid").is_ok());
        assert!(adapter.validate("invalid").is_err());
    }

    // ========================================================================
    // 取消操作测试
    // ========================================================================

    #[test]
    fn test_execute_confirm_field_cancel() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let field = create_test_field(FieldType::Confirm, "agree", "Do you agree?");
        let executors = FieldExecutors;

        let result = executors.execute_confirm_field_with_backend(&field, &mut backend);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_input_field_cancel() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let field = create_test_field(FieldType::Input, "name", "Enter name");
        let executors = FieldExecutors;

        let result = executors.execute_input_field_with_backend(&field, false, &mut backend);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_select_field_cancel() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::Select, "choice", "Select");
        field.options = vec!["A".to_string(), "B".to_string()];
        let executors = FieldExecutors;

        let result = executors.execute_select_field_with_backend(&field, &mut backend);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_multiselect_field_cancel() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let mut field = create_test_field(FieldType::MultiSelect, "choices", "Select");
        field.options = vec!["A".to_string(), "B".to_string()];
        let executors = FieldExecutors;

        let result = executors.execute_multiselect_field_with_backend(&field, &mut backend);
        assert!(result.is_err());
    }
}
