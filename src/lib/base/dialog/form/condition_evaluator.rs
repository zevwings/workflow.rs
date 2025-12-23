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
