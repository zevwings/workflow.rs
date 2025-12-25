//! Base/Dialog/Form Condition Evaluator 模块测试
//!
//! 测试条件评估器的各种操作符和逻辑。

use std::collections::HashMap;
use workflow::base::dialog::{Condition, ConditionEvaluator, ConditionOperator, ConditionValue};

// ==================== Field Not Found Tests ====================

#[test]
fn test_condition_evaluator_with_field_not_found_returns_false() {
    // Arrange: 准备字段不存在的情况
    let condition = Condition {
        field_name: "nonexistent".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("value".to_string()),
    };
    let field_values = HashMap::new();

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证字段不存在应该返回 false
    assert!(!result);
}

// ==================== Equals Operator Tests ====================

#[test]
fn test_condition_evaluator_equals_with_matching_single_value_returns_true() {
    // Arrange: 准备匹配的条件和字段值
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("value1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证匹配时返回 true
    assert!(result);
}

#[test]
fn test_condition_evaluator_equals_with_non_matching_single_value_returns_false() {
    // Arrange: 准备不匹配的条件和字段值
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("value1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value2".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证不匹配时返回 false
    assert!(!result);
}

#[test]
fn test_condition_evaluator_equals_with_case_insensitive_match_returns_true() {
    // Arrange: 准备大小写不同的匹配条件
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Single("VALUE1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证大小写不敏感匹配时返回 true
    assert!(result);
}

#[test]
fn test_condition_evaluator_equals_with_multiple_value_returns_false() {
    // Arrange: 准备 Multiple 值的条件（Equals 不支持）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::Multiple(vec!["value1".to_string(), "value2".to_string()]),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证 Equals 不支持 Multiple，应该返回 false
    assert!(!result);
}

// ==================== NotEquals Operator Tests ====================

#[test]
fn test_condition_evaluator_not_equals_with_non_matching_single_value_returns_true() {
    // Arrange: 准备不匹配的条件（NotEquals 应该返回 true）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Single("value1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value2".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证不相等时返回 true
    assert!(result);
}

#[test]
fn test_condition_evaluator_not_equals_with_matching_single_value_returns_false() {
    // Arrange: 准备匹配的条件（NotEquals 应该返回 false）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Single("value1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证相等时返回 false
    assert!(!result);
}

#[test]
fn test_condition_evaluator_not_equals_with_case_insensitive_match_returns_false() {
    // Arrange: 准备大小写不同的匹配条件
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Single("VALUE1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证大小写不敏感匹配时返回 false
    assert!(!result);
}

#[test]
fn test_condition_evaluator_not_equals_with_multiple_value_returns_false() {
    // Arrange: 准备 Multiple 值的条件（NotEquals 不支持）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotEquals,
        value: ConditionValue::Multiple(vec!["value1".to_string(), "value2".to_string()]),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value3".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证 NotEquals 不支持 Multiple，应该返回 false
    assert!(!result);
}

// ==================== In Operator Tests ====================

#[test]
fn test_condition_evaluator_in_with_matching_value_returns_true() {
    // Arrange: 准备匹配的条件（值在列表中）
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

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证值在列表中时返回 true
    assert!(result);
}

#[test]
fn test_condition_evaluator_in_with_non_matching_value_returns_false() {
    // Arrange: 准备不匹配的条件（值不在列表中）
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

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证值不在列表中时返回 false
    assert!(!result);
}

#[test]
fn test_condition_evaluator_in_with_case_insensitive_match_returns_true() {
    // Arrange: 准备大小写不同的匹配条件
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::In,
        value: ConditionValue::Multiple(vec!["VALUE1".to_string(), "VALUE2".to_string()]),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证大小写不敏感匹配时返回 true
    assert!(result);
}

#[test]
fn test_condition_evaluator_in_with_single_value_returns_false() {
    // Arrange: 准备 Single 值的条件（In 不支持）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::In,
        value: ConditionValue::Single("value1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证 In 不支持 Single，应该返回 false
    assert!(!result);
}

// ==================== NotIn Operator Tests ====================

#[test]
fn test_condition_evaluator_not_in_with_non_matching_value_returns_true() {
    // Arrange: 准备不匹配的条件（值不在列表中，应该返回 true）
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

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证值不在列表中时返回 true
    assert!(result);
}

#[test]
fn test_condition_evaluator_not_in_with_matching_value_returns_false() {
    // Arrange: 准备匹配的条件（值在列表中，应该返回 false）
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

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证值在列表中时返回 false
    assert!(!result);
}

#[test]
fn test_condition_evaluator_not_in_with_case_insensitive_match_returns_false() {
    // Arrange: 准备大小写不同的匹配条件
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotIn,
        value: ConditionValue::Multiple(vec!["VALUE1".to_string(), "VALUE2".to_string()]),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证大小写不敏感匹配时返回 false
    assert!(!result);
}

#[test]
fn test_condition_evaluator_not_in_with_single_value_returns_true() {
    // Arrange: 准备 Single 值的条件（NotIn 对 Single 值总是返回 true）
    let condition = Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::NotIn,
        value: ConditionValue::Single("value1".to_string()),
    };
    let mut field_values = HashMap::new();
    field_values.insert("field1".to_string(), "value1".to_string());

    // Act: 评估条件
    let result = ConditionEvaluator::evaluate(&condition, &field_values);

    // Assert: 验证 NotIn 对 Single 值总是返回 true
    assert!(result);
}
