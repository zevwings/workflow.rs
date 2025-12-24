//! Base/Dialog/Form Types 模块测试
//!
//! 测试表单类型定义的核心功能。

use color_eyre::Result;
use workflow::base::dialog::{
    Condition, ConditionOperator, ConditionValue, FieldDefaultValue, FormField, FormFieldType,
    FormGroup, FormResult, FormStep, GroupConfig, StepType,
};

// ==================== FormFieldType 枚举测试 ====================

#[test]
fn test_form_field_type_variants() {
    // 测试所有 FormFieldType 变体的创建和比较
    let text = FormFieldType::Text;
    let password = FormFieldType::Password;
    let selection = FormFieldType::Selection;
    let confirmation = FormFieldType::Confirmation;

    // 验证变体可以创建
    assert_eq!(text, FormFieldType::Text);
    assert_eq!(password, FormFieldType::Password);
    assert_eq!(selection, FormFieldType::Selection);
    assert_eq!(confirmation, FormFieldType::Confirmation);

    // 验证变体不相等
    assert_ne!(text, password);
    assert_ne!(selection, confirmation);
}

#[test]
fn test_form_field_type_clone() {
    // 测试 FormFieldType 的 Clone trait
    let field_type = FormFieldType::Text;
    let cloned = field_type;
    assert_eq!(field_type, cloned);
}

#[test]
fn test_form_field_type_debug() {
    // 测试 FormFieldType 的 Debug trait
    let field_type = FormFieldType::Text;
    let debug_str = format!("{:?}", field_type);
    assert!(debug_str.contains("Text"));
}

// ==================== ConditionOperator 枚举测试 ====================

#[test]
fn test_condition_operator_variants() {
    // 测试所有 ConditionOperator 变体的创建和比较
    let equals = ConditionOperator::Equals;
    let not_equals = ConditionOperator::NotEquals;
    let in_op = ConditionOperator::In;
    let not_in = ConditionOperator::NotIn;

    // 验证变体可以创建
    assert_eq!(equals, ConditionOperator::Equals);
    assert_eq!(not_equals, ConditionOperator::NotEquals);
    assert_eq!(in_op, ConditionOperator::In);
    assert_eq!(not_in, ConditionOperator::NotIn);

    // 验证变体不相等
    assert_ne!(equals, not_equals);
    assert_ne!(in_op, not_in);
}

#[test]
fn test_condition_operator_clone() {
    // 测试 ConditionOperator 的 Clone trait
    let op = ConditionOperator::Equals;
    let cloned = op;
    assert_eq!(op, cloned);
}

// ==================== ConditionValue 枚举测试 ====================

#[test]
fn test_condition_value_single() {
    // 测试创建单个值（覆盖 types.rs:58-60）
    let value = ConditionValue::single("test");
    match value {
        ConditionValue::Single(s) => assert_eq!(s, "test"),
        ConditionValue::Multiple(_) => panic!("Expected Single variant"),
    }
}

#[test]
fn test_condition_value_single_string() {
    // 测试 single() 方法接受 String
    let value = ConditionValue::single("test".to_string());
    match value {
        ConditionValue::Single(s) => assert_eq!(s, "test"),
        ConditionValue::Multiple(_) => panic!("Expected Single variant"),
    }
}

#[test]
fn test_condition_value_multiple() {
    // 测试创建多个值（覆盖 types.rs:63-65）
    let values = vec!["value1".to_string(), "value2".to_string()];
    let value = ConditionValue::multiple(values.clone());
    match value {
        ConditionValue::Single(_) => panic!("Expected Multiple variant"),
        ConditionValue::Multiple(v) => assert_eq!(v, values),
    }
}

#[test]
fn test_condition_value_clone() {
    // 测试 ConditionValue 的 Clone trait
    let value1 = ConditionValue::single("test");
    let value2 = value1.clone();
    match (value1, value2) {
        (ConditionValue::Single(s1), ConditionValue::Single(s2)) => assert_eq!(s1, s2),
        _ => panic!("Expected Single variants"),
    }
}

// ==================== Condition 结构体测试 ====================

#[test]
fn test_condition_creation() {
    // 测试创建 Condition（覆盖 types.rs:36-45）
    let condition = Condition {
        field_name: "test_field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("test_value"),
    };

    assert_eq!(condition.field_name, "test_field");
    assert_eq!(condition.operator, ConditionOperator::Equals);
    match condition.value {
        ConditionValue::Single(s) => assert_eq!(s, "test_value"),
        ConditionValue::Multiple(_) => panic!("Expected Single variant"),
    }
}

#[test]
fn test_condition_clone() {
    // 测试 Condition 的 Clone trait
    let condition1 = Condition {
        field_name: "field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };
    let condition2 = condition1.clone();
    assert_eq!(condition1.field_name, condition2.field_name);
    assert_eq!(condition1.operator, condition2.operator);
}

#[test]
fn test_condition_debug() {
    // 测试 Condition 的 Debug trait
    let condition = Condition {
        field_name: "field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };
    let debug_str = format!("{:?}", condition);
    assert!(debug_str.contains("field"));
}

// ==================== FieldDefaultValue 枚举测试 ====================

#[test]
fn test_field_default_value_string() {
    // 测试创建字符串默认值
    let value = FieldDefaultValue::String("test".to_string());
    match value {
        FieldDefaultValue::String(s) => assert_eq!(s, "test"),
        FieldDefaultValue::Bool(_) => panic!("Expected String variant"),
    }
}

#[test]
fn test_field_default_value_bool() {
    // 测试创建布尔默认值
    let value = FieldDefaultValue::Bool(true);
    match value {
        FieldDefaultValue::String(_) => panic!("Expected Bool variant"),
        FieldDefaultValue::Bool(b) => assert!(b),
    }
}

#[test]
fn test_field_default_value_as_string() {
    // 测试 as_string() 方法（覆盖 types.rs:79-83）
    let string_value = FieldDefaultValue::String("test".to_string());
    assert_eq!(string_value.as_string(), Some("test".to_string()));

    let bool_value = FieldDefaultValue::Bool(true);
    assert_eq!(bool_value.as_string(), None);
}

#[test]
fn test_field_default_value_as_bool() {
    // 测试 as_bool() 方法（覆盖 types.rs:87-91）
    let bool_value = FieldDefaultValue::Bool(true);
    assert_eq!(bool_value.as_bool(), Some(true));

    let string_value = FieldDefaultValue::String("test".to_string());
    assert_eq!(string_value.as_bool(), None);
}

#[test]
fn test_field_default_value_clone() {
    // 测试 FieldDefaultValue 的 Clone trait
    let value1 = FieldDefaultValue::String("test".to_string());
    let value2 = value1.clone();
    match (value1, value2) {
        (FieldDefaultValue::String(s1), FieldDefaultValue::String(s2)) => assert_eq!(s1, s2),
        _ => panic!("Expected String variants"),
    }
}

// ==================== FormResult 结构体测试 ====================

#[test]
fn test_form_result_new() {
    // 测试创建新的 FormResult（覆盖 types.rs:132-136）
    let result = FormResult::new();
    assert!(result.values.is_empty());
}

#[test]
fn test_form_result_default() {
    // 测试 FormResult 的 Default trait（覆盖 types.rs:173-176）
    let result = FormResult::default();
    assert!(result.values.is_empty());
}

#[test]
fn test_form_result_get() {
    // 测试 get() 方法（覆盖 types.rs:139-141）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert_eq!(result.get("field1"), Some(&"value1".to_string()));
    assert_eq!(result.get("field2"), None);
}

#[test]
fn test_form_result_get_required() -> Result<()> {
    // 测试 get_required() 方法（覆盖 types.rs:144-149）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert_eq!(result.get_required("field1")?, "value1");
    assert!(result.get_required("field2").is_err());

    Ok(())
}

#[test]
fn test_form_result_get_or_default() {
    // 测试 get_or_default() 方法（覆盖 types.rs:152-154）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert_eq!(result.get_or_default("field1", "default"), "value1");
    assert_eq!(result.get_or_default("field2", "default"), "default");
}

#[test]
fn test_form_result_has() {
    // 测试 has() 方法（覆盖 types.rs:157-159）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert!(result.has("field1"));
    assert!(!result.has("field2"));
}

#[test]
fn test_form_result_get_bool() {
    // 测试 get_bool() 方法（覆盖 types.rs:162-164）
    let mut result = FormResult::new();
    result.values.insert("yes_field".to_string(), "yes".to_string());
    result.values.insert("no_field".to_string(), "no".to_string());
    result.values.insert("other_field".to_string(), "other".to_string());

    assert_eq!(result.get_bool("yes_field"), Some(true));
    assert_eq!(result.get_bool("no_field"), Some(false));
    assert_eq!(result.get_bool("other_field"), Some(false)); // "other" != "yes"
    assert_eq!(result.get_bool("nonexistent"), None);
}

#[test]
fn test_form_result_get_required_bool() -> Result<()> {
    // 测试 get_required_bool() 方法（覆盖 types.rs:167-170）
    let mut result = FormResult::new();
    result.values.insert("yes_field".to_string(), "yes".to_string());

    assert_eq!(result.get_required_bool("yes_field")?, true);
    assert!(result.get_required_bool("nonexistent").is_err());

    Ok(())
}

#[test]
fn test_form_result_clone() {
    // 测试 FormResult 的 Clone trait
    let mut result1 = FormResult::new();
    result1.values.insert("field1".to_string(), "value1".to_string());
    let result2 = result1.clone();

    assert_eq!(result1.get("field1"), result2.get("field1"));
}

#[test]
fn test_form_result_debug() {
    // 测试 FormResult 的 Debug trait
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("field1") || debug_str.contains("value1"));
}

// ==================== StepType 枚举测试 ====================

#[test]
fn test_step_type_unconditional() {
    // 测试无条件步骤类型
    let step_type = StepType::Unconditional;
    match step_type {
        StepType::Unconditional => {}
        _ => panic!("Expected Unconditional variant"),
    }
}

#[test]
fn test_step_type_conditional() {
    // 测试单条件步骤类型
    let condition = Condition {
        field_name: "field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };
    let step_type = StepType::Conditional(condition.clone());
    match step_type {
        StepType::Conditional(c) => {
            assert_eq!(c.field_name, condition.field_name);
        }
        _ => panic!("Expected Conditional variant"),
    }
}

#[test]
fn test_step_type_conditional_all() {
    // 测试多条件步骤类型（AND 逻辑）
    let conditions = vec![
        Condition {
            field_name: "field1".to_string(),
            operator: ConditionOperator::Equals,
            value: ConditionValue::single("value1"),
        },
        Condition {
            field_name: "field2".to_string(),
            operator: ConditionOperator::Equals,
            value: ConditionValue::single("value2"),
        },
    ];
    let step_type = StepType::ConditionalAll(conditions.clone());
    match step_type {
        StepType::ConditionalAll(cs) => {
            assert_eq!(cs.len(), 2);
        }
        _ => panic!("Expected ConditionalAll variant"),
    }
}

#[test]
fn test_step_type_conditional_any() {
    // 测试多条件步骤类型（OR 逻辑）
    let conditions = vec![
        Condition {
            field_name: "field1".to_string(),
            operator: ConditionOperator::Equals,
            value: ConditionValue::single("value1"),
        },
    ];
    let step_type = StepType::ConditionalAny(conditions.clone());
    match step_type {
        StepType::ConditionalAny(cs) => {
            assert_eq!(cs.len(), 1);
        }
        _ => panic!("Expected ConditionalAny variant"),
    }
}

#[test]
fn test_step_type_dynamic_condition() {
    // 测试动态条件步骤类型
    let step_type = StepType::DynamicCondition(Box::new(|_result: &FormResult| true));
    match step_type {
        StepType::DynamicCondition(_) => {}
        _ => panic!("Expected DynamicCondition variant"),
    }
}

// ==================== FormStep 结构体测试 ====================

#[test]
fn test_form_step_creation() {
    // 测试创建 FormStep（覆盖 types.rs:194-203）
    let step = FormStep {
        id: Some("step1".to_string()),
        step_type: StepType::Unconditional,
        fields: vec![],
        skip_if_false: false,
    };

    assert_eq!(step.id, Some("step1".to_string()));
    assert_eq!(step.fields.len(), 0);
    assert!(!step.skip_if_false);
}

#[test]
fn test_form_step_without_id() {
    // 测试没有 ID 的 FormStep
    let step = FormStep {
        id: None,
        step_type: StepType::Unconditional,
        fields: vec![],
        skip_if_false: true,
    };

    assert_eq!(step.id, None);
    assert!(step.skip_if_false);
}

// ==================== GroupConfig 结构体测试 ====================

#[test]
fn test_group_config_required() {
    // 测试创建必填组配置（覆盖 types.rs:222-229）
    let config = GroupConfig::required();
    assert!(!config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

#[test]
fn test_group_config_optional() {
    // 测试创建可选组配置（覆盖 types.rs:232-239）
    let config = GroupConfig::optional();
    assert!(config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

#[test]
fn test_group_config_with_title() {
    // 测试设置组标题（覆盖 types.rs:242-245）
    let config = GroupConfig::required().with_title("Test Title");
    assert_eq!(config.title, Some("Test Title".to_string()));
}

#[test]
fn test_group_config_with_title_string() {
    // 测试 with_title() 接受 String
    let config = GroupConfig::required().with_title("Test Title".to_string());
    assert_eq!(config.title, Some("Test Title".to_string()));
}

#[test]
fn test_group_config_with_description() {
    // 测试设置组描述（覆盖 types.rs:248-251）
    let config = GroupConfig::required().with_description("Test Description");
    assert_eq!(config.description, Some("Test Description".to_string()));
}

#[test]
fn test_group_config_with_default_enabled() {
    // 测试设置默认启用（覆盖 types.rs:254-257）
    let config = GroupConfig::optional().with_default_enabled(true);
    assert!(config.default_enabled);

    let config2 = GroupConfig::optional().with_default_enabled(false);
    assert!(!config2.default_enabled);
}

#[test]
fn test_group_config_chain_calls() {
    // 测试链式调用所有方法
    let config = GroupConfig::optional()
        .with_title("Title")
        .with_description("Description")
        .with_default_enabled(true);

    assert!(config.optional);
    assert_eq!(config.title, Some("Title".to_string()));
    assert_eq!(config.description, Some("Description".to_string()));
    assert!(config.default_enabled);
}

#[test]
fn test_group_config_default() {
    // 测试 GroupConfig 的 Default trait（覆盖 types.rs:260-263）
    let config = GroupConfig::default();
    assert!(!config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

#[test]
fn test_group_config_clone() {
    // 测试 GroupConfig 的 Clone trait
    let config1 = GroupConfig::optional().with_title("Title");
    let config2 = config1.clone();
    assert_eq!(config1.title, config2.title);
    assert_eq!(config1.optional, config2.optional);
}

#[test]
fn test_group_config_debug() {
    // 测试 GroupConfig 的 Debug trait
    let config = GroupConfig::required().with_title("Title");
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("Title") || debug_str.contains("optional"));
}

// ==================== FormGroup 结构体测试 ====================

#[test]
fn test_form_group_creation() {
    // 测试创建 FormGroup（覆盖 types.rs:267-280）
    let group = FormGroup {
        id: "group1".to_string(),
        title: Some("Group Title".to_string()),
        description: Some("Group Description".to_string()),
        optional: false,
        default_enabled: false,
        steps: vec![],
    };

    assert_eq!(group.id, "group1");
    assert_eq!(group.title, Some("Group Title".to_string()));
    assert_eq!(group.description, Some("Group Description".to_string()));
    assert!(!group.optional);
    assert!(!group.default_enabled);
    assert_eq!(group.steps.len(), 0);
}

#[test]
fn test_form_group_optional() {
    // 测试可选组
    let group = FormGroup {
        id: "group2".to_string(),
        title: None,
        description: None,
        optional: true,
        default_enabled: true,
        steps: vec![],
    };

    assert!(group.optional);
    assert!(group.default_enabled);
}

// ==================== FormField 结构体测试 ====================

#[test]
fn test_form_field_creation() {
    // 测试创建 FormField（覆盖 types.rs:97-121）
    let field = FormField {
        name: "field1".to_string(),
        field_type: FormFieldType::Text,
        message: "Enter value".to_string(),
        choices: vec![],
        default_choice: None,
        default_value: None,
        required: false,
        allow_empty: true,
        validator: None,
        condition: None,
    };

    assert_eq!(field.name, "field1");
    assert_eq!(field.field_type, FormFieldType::Text);
    assert_eq!(field.message, "Enter value");
    assert!(!field.required);
    assert!(field.allow_empty);
}

#[test]
fn test_form_field_with_condition() {
    // 测试带条件的字段
    let condition = Condition {
        field_name: "other_field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };
    let field = FormField {
        name: "field1".to_string(),
        field_type: FormFieldType::Text,
        message: "Enter value".to_string(),
        choices: vec![],
        default_choice: None,
        default_value: None,
        required: true,
        allow_empty: false,
        validator: None,
        condition: Some(condition.clone()),
    };

    assert!(field.required);
    assert!(!field.allow_empty);
    assert_eq!(field.condition.as_ref().unwrap().field_name, condition.field_name);
}

#[test]
fn test_form_field_clone() {
    // 测试 FormField 的 Clone trait
    let field1 = FormField {
        name: "field1".to_string(),
        field_type: FormFieldType::Text,
        message: "Message".to_string(),
        choices: vec![],
        default_choice: None,
        default_value: None,
        required: false,
        allow_empty: true,
        validator: None,
        condition: None,
    };
    let field2 = field1.clone();
    assert_eq!(field1.name, field2.name);
    assert_eq!(field1.field_type, field2.field_type);
}
