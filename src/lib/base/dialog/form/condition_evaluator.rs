//! 条件评估器
//!
//! 提供条件评估逻辑，用于判断步骤和字段是否应该执行。

use std::collections::HashMap;

use crate::base::dialog::form::{Condition, ConditionOperator, ConditionValue};

/// 条件评估器
///
/// 提供条件评估功能，用于判断条件是否满足。
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// 评估条件
    ///
    /// 根据字段值和条件操作符判断条件是否满足。
    ///
    /// # 参数
    ///
    /// * `condition` - 要评估的条件
    /// * `field_values` - 当前已收集的字段值
    ///
    /// # 返回
    ///
    /// 如果字段还未收集，返回 `false`。
    /// 否则根据条件操作符返回相应的布尔值。
    pub fn evaluate(condition: &Condition, field_values: &HashMap<String, String>) -> bool {
        let field_value = match field_values.get(&condition.field_name) {
            Some(v) => v.clone(),
            None => return false, // 字段还未收集
        };

        match condition.operator {
            ConditionOperator::Equals => match &condition.value {
                ConditionValue::Single(expected) => field_value.eq_ignore_ascii_case(expected),
                ConditionValue::Multiple(_) => false,
            },
            ConditionOperator::NotEquals => match &condition.value {
                ConditionValue::Single(expected) => !field_value.eq_ignore_ascii_case(expected),
                ConditionValue::Multiple(_) => false,
            },
            ConditionOperator::In => match &condition.value {
                ConditionValue::Single(_) => false,
                ConditionValue::Multiple(values) => {
                    values.iter().any(|v| field_value.eq_ignore_ascii_case(v))
                }
            },
            ConditionOperator::NotIn => match &condition.value {
                ConditionValue::Single(_) => true,
                ConditionValue::Multiple(values) => {
                    !values.iter().any(|v| field_value.eq_ignore_ascii_case(v))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::dialog::form::{
        Condition, ConditionEvaluator, ConditionOperator, ConditionValue,
    };
    use std::collections::HashMap;

    // ==================== Field Not Found Tests ====================

    /// 测试字段不存在时的条件评估
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 在字段不存在时返回 false。
    ///
    /// ## 测试场景
    /// 1. 创建指向不存在字段的条件
    /// 2. 使用空的字段值映射评估条件
    /// 3. 验证返回 false
    ///
    /// ## 预期结果
    /// - 字段不存在时返回 false
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

    /// 测试 Equals 操作符匹配单个值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 Equals 操作符在值匹配时返回 true。
    ///
    /// ## 测试场景
    /// 1. 创建 Equals 条件和匹配的字段值
    /// 2. 评估条件
    /// 3. 验证返回 true
    ///
    /// ## 预期结果
    /// - 值匹配时返回 true
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

    /// 测试 Equals 操作符不匹配单个值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 Equals 操作符在值不匹配时返回 false。
    ///
    /// ## 测试场景
    /// 1. 创建 Equals 条件和不匹配的字段值
    /// 2. 评估条件
    /// 3. 验证返回 false
    ///
    /// ## 预期结果
    /// - 值不匹配时返回 false
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

    /// 测试 Equals 操作符大小写不敏感匹配
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 Equals 操作符支持大小写不敏感匹配。
    ///
    /// ## 测试场景
    /// 1. 创建大小写不同的 Equals 条件
    /// 2. 评估条件
    /// 3. 验证大小写不敏感匹配时返回 true
    ///
    /// ## 预期结果
    /// - 大小写不敏感匹配时返回 true
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

    /// 测试 Equals 操作符不支持 Multiple 值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 Equals 操作符不支持 Multiple 值类型。
    ///
    /// ## 测试场景
    /// 1. 创建使用 Multiple 值的 Equals 条件
    /// 2. 评估条件
    /// 3. 验证返回 false（不支持）
    ///
    /// ## 预期结果
    /// - Equals 不支持 Multiple 值，返回 false
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

    /// 测试 NotEquals 操作符不匹配单个值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotEquals 操作符在值不匹配时返回 true。
    ///
    /// ## 测试场景
    /// 1. 创建 NotEquals 条件和不匹配的字段值
    /// 2. 评估条件
    /// 3. 验证返回 true
    ///
    /// ## 预期结果
    /// - 值不相等时返回 true
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

    /// 测试 NotEquals 操作符匹配单个值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotEquals 操作符在值匹配时返回 false。
    ///
    /// ## 测试场景
    /// 1. 创建 NotEquals 条件和匹配的字段值
    /// 2. 评估条件
    /// 3. 验证返回 false
    ///
    /// ## 预期结果
    /// - 值相等时返回 false
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

    /// 测试 NotEquals 操作符大小写不敏感匹配
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotEquals 操作符支持大小写不敏感匹配。
    ///
    /// ## 测试场景
    /// 1. 创建大小写不同的 NotEquals 条件
    /// 2. 评估条件
    /// 3. 验证大小写不敏感匹配时返回 false
    ///
    /// ## 预期结果
    /// - 大小写不敏感匹配时返回 false
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

    /// 测试 NotEquals 操作符不支持 Multiple 值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotEquals 操作符不支持 Multiple 值类型。
    ///
    /// ## 测试场景
    /// 1. 创建使用 Multiple 值的 NotEquals 条件
    /// 2. 评估条件
    /// 3. 验证返回 false（不支持）
    ///
    /// ## 预期结果
    /// - NotEquals 不支持 Multiple 值，返回 false
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

    /// 测试 In 操作符匹配值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 In 操作符在值在列表中时返回 true。
    ///
    /// ## 测试场景
    /// 1. 创建 In 条件和匹配的字段值（值在列表中）
    /// 2. 评估条件
    /// 3. 验证返回 true
    ///
    /// ## 预期结果
    /// - 值在列表中时返回 true
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

    /// 测试 In 操作符不匹配值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 In 操作符在值不在列表中时返回 false。
    ///
    /// ## 测试场景
    /// 1. 创建 In 条件和不匹配的字段值（值不在列表中）
    /// 2. 评估条件
    /// 3. 验证返回 false
    ///
    /// ## 预期结果
    /// - 值不在列表中时返回 false
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

    /// 测试 In 操作符大小写不敏感匹配
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 In 操作符支持大小写不敏感匹配。
    ///
    /// ## 测试场景
    /// 1. 创建大小写不同的 In 条件
    /// 2. 评估条件
    /// 3. 验证大小写不敏感匹配时返回 true
    ///
    /// ## 预期结果
    /// - 大小写不敏感匹配时返回 true
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

    /// 测试 In 操作符不支持 Single 值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 In 操作符不支持 Single 值类型。
    ///
    /// ## 测试场景
    /// 1. 创建使用 Single 值的 In 条件
    /// 2. 评估条件
    /// 3. 验证返回 false（不支持）
    ///
    /// ## 预期结果
    /// - In 不支持 Single 值，返回 false
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

    /// 测试 NotIn 操作符不匹配值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotIn 操作符在值不在列表中时返回 true。
    ///
    /// ## 测试场景
    /// 1. 创建 NotIn 条件和不匹配的字段值（值不在列表中）
    /// 2. 评估条件
    /// 3. 验证返回 true
    ///
    /// ## 预期结果
    /// - 值不在列表中时返回 true
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

    /// 测试 NotIn 操作符匹配值
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotIn 操作符在值在列表中时返回 false。
    ///
    /// ## 测试场景
    /// 1. 创建 NotIn 条件和匹配的字段值（值在列表中）
    /// 2. 评估条件
    /// 3. 验证返回 false
    ///
    /// ## 预期结果
    /// - 值在列表中时返回 false
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

    /// 测试 NotIn 操作符大小写不敏感匹配
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotIn 操作符支持大小写不敏感匹配。
    ///
    /// ## 测试场景
    /// 1. 创建大小写不同的 NotIn 条件
    /// 2. 评估条件
    /// 3. 验证大小写不敏感匹配时返回 false
    ///
    /// ## 预期结果
    /// - 大小写不敏感匹配时返回 false
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

    /// 测试 NotIn 操作符对 Single 值的处理
    ///
    /// ## 测试目的
    /// 验证 ConditionEvaluator 的 NotIn 操作符对 Single 值总是返回 true。
    ///
    /// ## 测试场景
    /// 1. 创建使用 Single 值的 NotIn 条件
    /// 2. 评估条件
    /// 3. 验证返回 true
    ///
    /// ## 预期结果
    /// - NotIn 对 Single 值总是返回 true
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
}
