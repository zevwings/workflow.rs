//! Base/Dialog/Form Types 模块测试
//!
//! 测试表单类型定义的核心功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::base::dialog::{
    Condition, ConditionOperator, ConditionValue, FormResult, GroupConfig,
};

// FormFieldType 不是直接导出的，我们通过其他方式测试

#[test]
fn test_condition_operator_variants() {
    // 测试 ConditionOperator 的所有变体
    assert_eq!(ConditionOperator::Equals as u8, 0);
    assert_eq!(ConditionOperator::NotEquals as u8, 1);
    assert_eq!(ConditionOperator::In as u8, 2);
    assert_eq!(ConditionOperator::NotIn as u8, 3);
}

#[test]
fn test_condition_value_single() {
    // 测试 ConditionValue::single() 方法（覆盖 types.rs:58-60）
    let value = ConditionValue::single("test");
    match value {
        ConditionValue::Single(s) => assert_eq!(s, "test"),
        ConditionValue::Multiple(_) => panic!("Expected Single"),
    }
}

#[test]
fn test_condition_value_single_string() {
    // 测试 ConditionValue::single() 接受 String
    let value = ConditionValue::single("test".to_string());
    match value {
        ConditionValue::Single(s) => assert_eq!(s, "test"),
        ConditionValue::Multiple(_) => panic!("Expected Single"),
    }
}

#[test]
fn test_condition_value_multiple() {
    // 测试 ConditionValue::multiple() 方法（覆盖 types.rs:63-65）
    let values = vec!["value1".to_string(), "value2".to_string()];
    let value = ConditionValue::multiple(values.clone());
    match value {
        ConditionValue::Single(_) => panic!("Expected Multiple"),
        ConditionValue::Multiple(v) => assert_eq!(v, values),
    }
}

// FieldDefaultValue 不是直接导出的，我们通过 FormBuilder 间接测试

#[test]
fn test_form_result_new() {
    // 测试 FormResult::new() 方法（覆盖 types.rs:132-136）
    let result = FormResult::new();
    assert!(result.values.is_empty());
}

#[test]
fn test_form_result_default() {
    // 测试 FormResult::default() 方法（覆盖 types.rs:173-176）
    let result = FormResult::default();
    assert!(result.values.is_empty());
}

#[test]
fn test_form_result_get() {
    // 测试 FormResult::get() 方法（覆盖 types.rs:139-141）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert_eq!(result.get("field1"), Some(&"value1".to_string()));
    assert_eq!(result.get("field2"), None);
}

#[test]
fn test_form_result_get_required() -> Result<()> {
    // 测试 FormResult::get_required() 方法（覆盖 types.rs:144-149）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert_eq!(result.get_required("field1")?, "value1");

    let error = result.get_required("field2");
    assert!(error.is_err());
    assert!(error.unwrap_err().to_string().contains("Field 'field2' is required"));

    Ok(())
}

#[test]
fn test_form_result_get_or_default() {
    // 测试 FormResult::get_or_default() 方法（覆盖 types.rs:152-154）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert_eq!(result.get_or_default("field1", "default"), "value1");
    assert_eq!(result.get_or_default("field2", "default"), "default");
}

#[test]
fn test_form_result_has() {
    // 测试 FormResult::has() 方法（覆盖 types.rs:157-159）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    assert!(result.has("field1"));
    assert!(!result.has("field2"));
}

#[test]
fn test_form_result_get_bool() {
    // 测试 FormResult::get_bool() 方法（覆盖 types.rs:162-164）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "yes".to_string());
    result.values.insert("field2".to_string(), "no".to_string());
    result.values.insert("field3".to_string(), "maybe".to_string());

    assert_eq!(result.get_bool("field1"), Some(true));
    assert_eq!(result.get_bool("field2"), Some(false));
    assert_eq!(result.get_bool("field3"), Some(false)); // "maybe" != "yes"
    assert_eq!(result.get_bool("field4"), None);
}

#[test]
fn test_form_result_get_required_bool() -> Result<()> {
    // 测试 FormResult::get_required_bool() 方法（覆盖 types.rs:167-170）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "yes".to_string());

    assert_eq!(result.get_required_bool("field1")?, true);

    let error = result.get_required_bool("field2");
    assert!(error.is_err());
    assert!(error.unwrap_err().to_string().contains("Field 'field2' is required"));

    Ok(())
}

#[test]
fn test_group_config_required() {
    // 测试 GroupConfig::required() 方法（覆盖 types.rs:222-228）
    let config = GroupConfig::required();
    assert!(!config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

#[test]
fn test_group_config_optional() {
    // 测试 GroupConfig::optional() 方法（覆盖 types.rs:232-238）
    let config = GroupConfig::optional();
    assert!(config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

#[test]
fn test_group_config_with_title() {
    // 测试 GroupConfig::with_title() 方法（覆盖 types.rs:242-245）
    let config = GroupConfig::required().with_title("Test Group");
    assert_eq!(config.title, Some("Test Group".to_string()));
}

#[test]
fn test_group_config_with_description() {
    // 测试 GroupConfig::with_description() 方法（覆盖 types.rs:248-251）
    let config = GroupConfig::required().with_description("Test Description");
    assert_eq!(config.description, Some("Test Description".to_string()));
}

#[test]
fn test_group_config_with_default_enabled() {
    // 测试 GroupConfig::with_default_enabled() 方法（覆盖 types.rs:254-257）
    let config = GroupConfig::optional().with_default_enabled(true);
    assert!(config.default_enabled);
}

#[test]
fn test_group_config_chain_all() {
    // 测试 GroupConfig 的链式调用
    let config = GroupConfig::optional()
        .with_title("Test Group")
        .with_description("Test Description")
        .with_default_enabled(true);

    assert!(config.optional);
    assert_eq!(config.title, Some("Test Group".to_string()));
    assert_eq!(config.description, Some("Test Description".to_string()));
    assert!(config.default_enabled);
}

#[test]
fn test_group_config_default() {
    // 测试 GroupConfig::default() 方法（覆盖 types.rs:260-263）
    let config = GroupConfig::default();
    assert!(!config.optional); // default 应该返回 required
}

#[test]
fn test_condition_clone() {
    // 测试 Condition 的 Clone trait
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value1"),
    };

    let cloned = condition.clone();
    assert_eq!(condition.field_name, cloned.field_name);
    assert_eq!(condition.operator as u8, cloned.operator as u8);
}

#[test]
fn test_condition_value_clone() {
    // 测试 ConditionValue 的 Clone trait
    let value = ConditionValue::single("test");
    let cloned = value.clone();

    match (value, cloned) {
        (ConditionValue::Single(s1), ConditionValue::Single(s2)) => assert_eq!(s1, s2),
        _ => panic!("Expected Single"),
    }
}

// FieldDefaultValue 不是直接导出的，我们通过 FormBuilder 间接测试

#[test]
fn test_form_result_clone() {
    // 测试 FormResult 的 Clone trait
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    let cloned = result.clone();
    assert_eq!(result.values, cloned.values);
}

#[test]
fn test_group_config_clone() {
    // 测试 GroupConfig 的 Clone trait
    let config = GroupConfig::optional().with_title("Test").with_description("Description");

    let cloned = config.clone();
    assert_eq!(config.optional, cloned.optional);
    assert_eq!(config.title, cloned.title);
    assert_eq!(config.description, cloned.description);
}

#[test]
fn test_condition_operator_all_variants() {
    // 测试 ConditionOperator 的所有变体（覆盖 types.rs:22-32）
    assert_eq!(ConditionOperator::Equals, ConditionOperator::Equals);
    assert_eq!(ConditionOperator::NotEquals, ConditionOperator::NotEquals);
    assert_eq!(ConditionOperator::In, ConditionOperator::In);
    assert_eq!(ConditionOperator::NotIn, ConditionOperator::NotIn);
    assert_ne!(ConditionOperator::Equals, ConditionOperator::NotEquals);
}

#[test]
fn test_condition_operator_debug() {
    // 测试 ConditionOperator 的 Debug trait
    let op = ConditionOperator::Equals;
    let debug_str = format!("{:?}", op);
    assert!(debug_str.contains("Equals") || debug_str.contains("ConditionOperator"));
}

#[test]
fn test_condition_value_debug() {
    // 测试 ConditionValue 的 Debug trait
    let value = ConditionValue::single("test");
    let debug_str = format!("{:?}", value);
    assert!(debug_str.contains("test") || debug_str.contains("Single") || debug_str.contains("ConditionValue"));
}
