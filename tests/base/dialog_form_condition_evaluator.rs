//! Base/Dialog/Form Condition Evaluator 模块测试
//!
//! 测试条件评估器的各种操作符和逻辑。

use std::collections::HashMap;
use workflow::base::dialog::{Condition, ConditionEvaluator, ConditionOperator, ConditionValue};

#[test]
fn test_condition_evaluator_field_not_found() {
    // 测试字段不存在的情况
    let condition = Condition {
        field_name: "nonexistent".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("value".to_string()),
    };

    let field_values = HashMap::new();
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    assert!(!result); // 字段不存在应该返回 false
}

#[test]
fn test_condition_evaluator_equals_single_match() {
    // 测试 Equals 操作符 - 匹配
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("value1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result);
}

#[test]
fn test_condition_evaluator_equals_single_no_match() {
    // 测试 Equals 操作符 - 不匹配
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("value1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value2".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result);
}

#[test]
fn test_condition_evaluator_equals_case_insensitive() {
    // 测试 Equals 操作符 - 大小写不敏感
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("VALUE1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result); // 应该匹配（大小写不敏感）
}

#[test]
fn test_condition_evaluator_equals_multiple() {
    // 测试 Equals 操作符 - Multiple 值应该返回 false
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Multiple(vec!["value1".to_string(), "value2".to_string()]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // Equals 不支持 Multiple，应该返回 false
}

#[test]
fn test_condition_evaluator_not_equals_single_match() {
    // 测试 NotEquals 操作符 - 不匹配（应该返回 true）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Single("value1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value2".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result); // 不相等，应该返回 true
}

#[test]
fn test_condition_evaluator_not_equals_single_no_match() {
    // 测试 NotEquals 操作符 - 匹配（应该返回 false）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Single("value1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // 相等，应该返回 false
}

#[test]
fn test_condition_evaluator_not_equals_case_insensitive() {
    // 测试 NotEquals 操作符 - 大小写不敏感
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Single("VALUE1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // 大小写不敏感，应该相等，返回 false
}

#[test]
fn test_condition_evaluator_not_equals_multiple() {
    // 测试 NotEquals 操作符 - Multiple 值应该返回 false
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Multiple(vec!["value1".to_string(), "value2".to_string()]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value3".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // NotEquals 不支持 Multiple，应该返回 false
}

#[test]
fn test_condition_evaluator_in_match() {
    // 测试 In 操作符 - 匹配
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::In,
        value: ConditionValue::Multiple(vec![
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value2".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result); // value2 在列表中，应该返回 true
}

#[test]
fn test_condition_evaluator_in_no_match() {
    // 测试 In 操作符 - 不匹配
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::In,
        value: ConditionValue::Multiple(vec![
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value4".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // value4 不在列表中，应该返回 false
}

#[test]
fn test_condition_evaluator_in_case_insensitive() {
    // 测试 In 操作符 - 大小写不敏感
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::In,
        value: ConditionValue::Multiple(vec!["VALUE1".to_string(), "VALUE2".to_string()]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result); // 大小写不敏感，应该匹配
}

#[test]
fn test_condition_evaluator_in_single() {
    // 测试 In 操作符 - Single 值应该返回 false
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::In,
        value: ConditionValue::Single("value1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // In 不支持 Single，应该返回 false
}

#[test]
fn test_condition_evaluator_not_in_match() {
    // 测试 NotIn 操作符 - 不在列表中（应该返回 true）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotIn,
        value: ConditionValue::Multiple(vec![
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value4".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result); // value4 不在列表中，应该返回 true
}

#[test]
fn test_condition_evaluator_not_in_no_match() {
    // 测试 NotIn 操作符 - 在列表中（应该返回 false）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotIn,
        value: ConditionValue::Multiple(vec![
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value2".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // value2 在列表中，应该返回 false
}

#[test]
fn test_condition_evaluator_not_in_case_insensitive() {
    // 测试 NotIn 操作符 - 大小写不敏感
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotIn,
        value: ConditionValue::Multiple(vec!["VALUE1".to_string(), "VALUE2".to_string()]),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(!result); // 大小写不敏感，应该匹配，返回 false
}

#[test]
fn test_condition_evaluator_not_in_single() {
    // 测试 NotIn 操作符 - Single 值应该返回 true
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotIn,
        value: ConditionValue::Single("value1".to_string()),
    };

    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    let result = ConditionEvaluator::evaluate(&condition, &field_values);
    assert!(result); // NotIn 对 Single 值总是返回 true
}
