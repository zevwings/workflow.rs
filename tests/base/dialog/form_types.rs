//! Base/Dialog/Form Types 模块测试
//!
//! 测试表单类型定义的核心功能。

use color_eyre::Result;
use workflow::base::dialog::{
    Condition, ConditionOperator, ConditionValue, FieldDefaultValue, FormField, FormFieldType,
    FormGroup, FormResult, FormStep, GroupConfig, StepType,
};

// ==================== FormFieldType 枚举测试 ====================

/// 测试 FormFieldType 枚举的所有变体
///
/// ## 测试目的
/// 验证 FormFieldType 枚举的所有变体（Text、Password、Selection、Confirmation）能够正确创建和比较。
///
/// ## 预期结果
/// - 所有变体都能正确创建
/// - 相等性比较正确
/// - 不同变体之间不相等
#[test]
fn test_form_field_type_variants_with_all_types_creates_and_compares() {
    // Arrange: 准备所有 FormFieldType 变体

    // Act: 创建各种变体
    let text = FormFieldType::Text;
    let password = FormFieldType::Password;
    let selection = FormFieldType::Selection;
    let confirmation = FormFieldType::Confirmation;

    // Assert: 验证变体可以创建且相等性正确
    assert_eq!(text, FormFieldType::Text);
    assert_eq!(password, FormFieldType::Password);
    assert_eq!(selection, FormFieldType::Selection);
    assert_eq!(confirmation, FormFieldType::Confirmation);
    assert_ne!(text, password);
    assert_ne!(selection, confirmation);
}

/// 测试 FormFieldType 枚举的克隆功能
///
/// ## 测试目的
/// 验证 FormFieldType 枚举能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_form_field_type_clone_with_valid_type_creates_clone() {
    // Arrange: 准备 FormFieldType 实例

    // Act: 克隆类型
    let field_type = FormFieldType::Text;
    let cloned = field_type;

    // Assert: 验证克隆成功
    assert_eq!(field_type, cloned);
}

/// 测试 FormFieldType 枚举的调试格式化
///
/// ## 测试目的
/// 验证 FormFieldType 枚举能够正确格式化调试字符串。
///
/// ## 预期结果
/// - 调试字符串包含类型信息
#[test]
fn test_form_field_type_debug_with_valid_type_returns_debug_string() {
    // Arrange: 准备 FormFieldType 实例

    // Act: 格式化调试字符串
    let field_type = FormFieldType::Text;
    let debug_str = format!("{:?}", field_type);

    // Assert: 验证调试字符串包含类型信息
    assert!(debug_str.contains("Text"));
}

// ==================== ConditionOperator 枚举测试 ====================

/// 测试 ConditionOperator 枚举的所有变体
///
/// ## 测试目的
/// 验证 ConditionOperator 枚举的所有变体（Equals、NotEquals、In、NotIn）能够正确创建和比较。
///
/// ## 预期结果
/// - 所有变体都能正确创建
/// - 相等性比较正确
/// - 不同变体之间不相等
#[test]
fn test_condition_operator_variants_with_all_operators_creates_and_compares() {
    // Arrange: 准备所有 ConditionOperator 变体

    // Act: 创建各种操作符
    let equals = ConditionOperator::Equals;
    let not_equals = ConditionOperator::NotEquals;
    let in_op = ConditionOperator::In;
    let not_in = ConditionOperator::NotIn;

    // Assert: 验证操作符可以创建且相等性正确
    assert_eq!(equals, ConditionOperator::Equals);
    assert_eq!(not_equals, ConditionOperator::NotEquals);
    assert_eq!(in_op, ConditionOperator::In);
    assert_eq!(not_in, ConditionOperator::NotIn);
    assert_ne!(equals, not_equals);
    assert_ne!(in_op, not_in);
}

/// 测试 ConditionOperator 枚举的克隆功能
///
/// ## 测试目的
/// 验证 ConditionOperator 枚举能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_condition_operator_clone_with_valid_operator_creates_clone() {
    // Arrange: 准备 ConditionOperator 实例

    // Act: 克隆操作符
    let op = ConditionOperator::Equals;
    let cloned = op;

    // Assert: 验证克隆成功
    assert_eq!(op, cloned);
}

// ==================== ConditionValue 枚举测试 ====================

/// 测试 ConditionValue::single() 方法使用字符串切片创建单个值
///
/// ## 测试目的
/// 验证 ConditionValue::single() 方法能够使用字符串切片创建 Single 变体。
///
/// ## 预期结果
/// - 创建 ConditionValue::Single 变体
/// - 值正确
#[test]
fn test_condition_value_single_with_str_creates_single_value() {
    // Arrange: 准备字符串值（覆盖 types.rs:58-60）

    // Act: 创建单个值
    let value = ConditionValue::single("test");

    // Assert: 验证创建了 Single 变体
    match value {
        ConditionValue::Single(s) => assert_eq!(s, "test"),
        ConditionValue::Multiple(_) => panic!("Expected Single variant"),
    }
}

/// 测试 ConditionValue::single() 方法使用 String 创建单个值
///
/// ## 测试目的
/// 验证 ConditionValue::single() 方法能够使用 String 创建 Single 变体。
///
/// ## 预期结果
/// - 创建 ConditionValue::Single 变体
/// - 值正确
#[test]
fn test_condition_value_single_with_string_creates_single_value() {
    // Arrange: 准备 String 值

    // Act: 创建单个值（接受 String）
    let value = ConditionValue::single("test".to_string());

    // Assert: 验证创建了 Single 变体
    match value {
        ConditionValue::Single(s) => assert_eq!(s, "test"),
        ConditionValue::Multiple(_) => panic!("Expected Single variant"),
    }
}

/// 测试 ConditionValue::multiple() 方法创建多个值
///
/// ## 测试目的
/// 验证 ConditionValue::multiple() 方法能够使用 Vec<String> 创建 Multiple 变体。
///
/// ## 预期结果
/// - 创建 ConditionValue::Multiple 变体
/// - 值列表正确
#[test]
fn test_condition_value_multiple_with_vec_creates_multiple_values() {
    // Arrange: 准备多个值（覆盖 types.rs:63-65）
    let values = vec!["value1".to_string(), "value2".to_string()];

    // Act: 创建多个值
    let value = ConditionValue::multiple(values.clone());

    // Assert: 验证创建了 Multiple 变体
    match value {
        ConditionValue::Single(_) => panic!("Expected Multiple variant"),
        ConditionValue::Multiple(v) => assert_eq!(v, values),
    }
}

/// 测试 ConditionValue 枚举的克隆功能
///
/// ## 测试目的
/// 验证 ConditionValue 枚举能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_condition_value_clone_with_valid_value_creates_clone() {
    // Arrange: 准备 ConditionValue 实例

    // Act: 克隆值
    let value1 = ConditionValue::single("test");
    let value2 = value1.clone();

    // Assert: 验证克隆成功
    match (value1, value2) {
        (ConditionValue::Single(s1), ConditionValue::Single(s2)) => assert_eq!(s1, s2),
        _ => panic!("Expected Single variants"),
    }
}

// ==================== Condition 结构体测试 ====================

/// 测试 Condition 结构体的创建
///
/// ## 测试目的
/// 验证 Condition 结构体能够使用有效字段正确创建。
///
/// ## 预期结果
/// - 所有字段正确设置
/// - 字段值正确
#[test]
fn test_condition_creation_with_valid_fields_creates_condition() {
    // Arrange: 准备条件字段（覆盖 types.rs:36-45）

    // Act: 创建条件
    let condition = Condition {
        field_name: "test_field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("test_value"),
    };

    // Assert: 验证条件字段正确
    assert_eq!(condition.field_name, "test_field");
    assert_eq!(condition.operator, ConditionOperator::Equals);
    match condition.value {
        ConditionValue::Single(s) => assert_eq!(s, "test_value"),
        ConditionValue::Multiple(_) => panic!("Expected Single variant"),
    }
}

/// 测试 Condition 结构体的克隆功能
///
/// ## 测试目的
/// 验证 Condition 结构体能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_condition_clone_with_valid_condition_creates_clone() {
    // Arrange: 准备 Condition 实例

    // Act: 克隆条件
    let condition1 = Condition {
        field_name: "field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };
    let condition2 = condition1.clone();

    // Assert: 验证克隆成功
    assert_eq!(condition1.field_name, condition2.field_name);
    assert_eq!(condition1.operator, condition2.operator);
}

/// 测试 Condition 结构体的调试格式化
///
/// ## 测试目的
/// 验证 Condition 结构体能够正确格式化调试字符串。
///
/// ## 预期结果
/// - 调试字符串包含字段信息
#[test]
fn test_condition_debug_with_valid_condition_returns_debug_string() {
    // Arrange: 准备 Condition 实例

    // Act: 格式化调试字符串
    let condition = Condition {
        field_name: "field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };
    let debug_str = format!("{:?}", condition);

    // Assert: 验证调试字符串包含字段信息
    assert!(debug_str.contains("field"));
}

// ==================== FieldDefaultValue 枚举测试 ====================

/// 测试 FieldDefaultValue 枚举的 String 变体
///
/// ## 测试目的
/// 验证 FieldDefaultValue 枚举能够使用字符串创建 String 变体。
///
/// ## 预期结果
/// - 创建 FieldDefaultValue::String 变体
/// - 值正确
#[test]
fn test_field_default_value_string_with_str_creates_string_value() {
    // Arrange: 准备字符串值

    // Act: 创建字符串默认值
    let value = FieldDefaultValue::String("test".to_string());

    // Assert: 验证创建了 String 变体
    match value {
        FieldDefaultValue::String(s) => assert_eq!(s, "test"),
        FieldDefaultValue::Bool(_) => panic!("Expected String variant"),
    }
}

/// 测试 FieldDefaultValue 枚举的 Bool 变体
///
/// ## 测试目的
/// 验证 FieldDefaultValue 枚举能够使用布尔值创建 Bool 变体。
///
/// ## 预期结果
/// - 创建 FieldDefaultValue::Bool 变体
/// - 值正确
#[test]
fn test_field_default_value_bool_with_bool_creates_bool_value() {
    // Arrange: 准备布尔值

    // Act: 创建布尔默认值
    let value = FieldDefaultValue::Bool(true);

    // Assert: 验证创建了 Bool 变体
    match value {
        FieldDefaultValue::String(_) => panic!("Expected Bool variant"),
        FieldDefaultValue::Bool(b) => assert!(b),
    }
}

/// 测试 FieldDefaultValue::as_string() 方法
///
/// ## 测试目的
/// 验证 FieldDefaultValue::as_string() 方法能够正确返回字符串值。
///
/// ## 预期结果
/// - String 变体返回 Some(String)
/// - Bool 变体返回 None
#[test]
fn test_field_default_value_as_string_with_string_value_returns_string() {
    // Arrange: 准备字符串和布尔值（覆盖 types.rs:79-83）

    // Act & Assert: 测试 as_string() 方法
    let string_value = FieldDefaultValue::String("test".to_string());
    assert_eq!(string_value.as_string(), Some("test".to_string()));

    let bool_value = FieldDefaultValue::Bool(true);
    assert_eq!(bool_value.as_string(), None);
}

/// 测试 FieldDefaultValue::as_bool() 方法
///
/// ## 测试目的
/// 验证 FieldDefaultValue::as_bool() 方法能够正确返回布尔值。
///
/// ## 预期结果
/// - Bool 变体返回 Some(bool)
/// - String 变体返回 None
#[test]
fn test_field_default_value_as_bool_with_bool_value_returns_bool() {
    // Arrange: 准备布尔和字符串值（覆盖 types.rs:87-91）

    // Act & Assert: 测试 as_bool() 方法
    let bool_value = FieldDefaultValue::Bool(true);
    assert_eq!(bool_value.as_bool(), Some(true));

    let string_value = FieldDefaultValue::String("test".to_string());
    assert_eq!(string_value.as_bool(), None);
}

/// 测试 FieldDefaultValue 枚举的克隆功能
///
/// ## 测试目的
/// 验证 FieldDefaultValue 枚举能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_field_default_value_clone_with_valid_value_creates_clone() {
    // Arrange: 准备 FieldDefaultValue 实例

    // Act: 克隆值
    let value1 = FieldDefaultValue::String("test".to_string());
    let value2 = value1.clone();

    // Assert: 验证克隆成功
    match (value1, value2) {
        (FieldDefaultValue::String(s1), FieldDefaultValue::String(s2)) => assert_eq!(s1, s2),
        _ => panic!("Expected String variants"),
    }
}

// ==================== FormResult 结构体测试 ====================

/// 测试 FormResult::new() 方法创建空结果
///
/// ## 测试目的
/// 验证 FormResult::new() 方法能够创建空的表单结果。
///
/// ## 预期结果
/// - 创建空的 FormResult
/// - values 字段为空
#[test]
fn test_form_result_new_creates_empty_result() {
    // Arrange: 准备创建 FormResult（覆盖 types.rs:132-136）

    // Act: 创建新的 FormResult
    let result = FormResult::new();

    // Assert: 验证结果为空
    assert!(result.values.is_empty());
}

/// 测试 FormResult::default() 方法创建空结果
///
/// ## 测试目的
/// 验证 FormResult 的 Default trait 实现能够创建空的表单结果。
///
/// ## 预期结果
/// - 创建空的 FormResult
/// - values 字段为空
#[test]
fn test_form_result_default_creates_empty_result() {
    // Arrange: 准备使用 Default trait（覆盖 types.rs:173-176）

    // Act: 创建默认 FormResult
    let result = FormResult::default();

    // Assert: 验证结果为空
    assert!(result.values.is_empty());
}

/// 测试 FormResult::get() 方法获取字段值
///
/// ## 测试目的
/// 验证 FormResult::get() 方法能够正确获取字段值。
///
/// ## 预期结果
/// - 存在的字段返回 Some(value)
/// - 不存在的字段返回 None
#[test]
fn test_form_result_get_with_existing_field_returns_value() {
    // Arrange: 准备包含字段的 FormResult（覆盖 types.rs:139-141）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    // Act & Assert: 测试 get() 方法
    assert_eq!(result.get("field1"), Some(&"value1".to_string()));
    assert_eq!(result.get("field2"), None);
}

/// 测试 FormResult::get_required() 方法获取必需字段值
///
/// ## 测试目的
/// 验证 FormResult::get_required() 方法能够正确获取必需字段值，不存在时返回错误。
///
/// ## 预期结果
/// - 存在的字段返回 Ok(value)
/// - 不存在的字段返回错误
#[test]
fn test_form_result_get_required_with_existing_field_return_ok() -> Result<()> {
    // Arrange: 准备包含字段的 FormResult（覆盖 types.rs:144-149）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    // Act & Assert: 测试 get_required() 方法
    assert_eq!(result.get_required("field1")?, "value1");
    assert!(result.get_required("field2").is_err());

    Ok(())
}

/// 测试 FormResult::get_or_default() 方法获取字段值或默认值
///
/// ## 测试目的
/// 验证 FormResult::get_or_default() 方法能够返回字段值或默认值。
///
/// ## 预期结果
/// - 存在的字段返回字段值
/// - 不存在的字段返回默认值
#[test]
fn test_form_result_get_or_default_with_existing_field_returns_value() {
    // Arrange: 准备包含字段的 FormResult（覆盖 types.rs:152-154）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    // Act & Assert: 测试 get_or_default() 方法
    assert_eq!(result.get_or_default("field1", "default"), "value1");
    assert_eq!(result.get_or_default("field2", "default"), "default");
}

/// 测试 FormResult::has() 方法检查字段是否存在
///
/// ## 测试目的
/// 验证 FormResult::has() 方法能够正确检查字段是否存在。
///
/// ## 预期结果
/// - 存在的字段返回 true
/// - 不存在的字段返回 false
#[test]
fn test_form_result_has_with_existing_field_returns_true() {
    // Arrange: 准备包含字段的 FormResult（覆盖 types.rs:157-159）
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());

    // Act & Assert: 测试 has() 方法
    assert!(result.has("field1"));
    assert!(!result.has("field2"));
}

/// 测试 FormResult::get_bool() 方法获取布尔值
///
/// ## 测试目的
/// 验证 FormResult::get_bool() 方法能够正确解析字符串值为布尔值。
///
/// ## 预期结果
/// - "yes" 返回 Some(true)
/// - "no" 或其他值返回 Some(false)
/// - 不存在的字段返回 None
#[test]
fn test_form_result_get_bool_with_various_values_returns_bool() {
    // Arrange: 准备包含各种值的 FormResult（覆盖 types.rs:162-164）
    let mut result = FormResult::new();
    result.values.insert("yes_field".to_string(), "yes".to_string());
    result.values.insert("no_field".to_string(), "no".to_string());
    result.values.insert("other_field".to_string(), "other".to_string());

    // Act & Assert: 测试 get_bool() 方法
    assert_eq!(result.get_bool("yes_field"), Some(true));
    assert_eq!(result.get_bool("no_field"), Some(false));
    assert_eq!(result.get_bool("other_field"), Some(false)); // "other" != "yes"
    assert_eq!(result.get_bool("nonexistent"), None);
}

/// 测试 FormResult::get_required_bool() 方法获取必需布尔值
///
/// ## 测试目的
/// 验证 FormResult::get_required_bool() 方法能够正确获取必需布尔值，不存在时返回错误。
///
/// ## 预期结果
/// - 存在的字段返回 Ok(bool)
/// - 不存在的字段返回错误
#[test]
fn test_form_result_get_required_bool_with_existing_field_return_ok() -> Result<()> {
    // Arrange: 准备包含字段的 FormResult（覆盖 types.rs:167-170）
    let mut result = FormResult::new();
    result.values.insert("yes_field".to_string(), "yes".to_string());

    // Act & Assert: 测试 get_required_bool() 方法
    assert_eq!(result.get_required_bool("yes_field")?, true);
    assert!(result.get_required_bool("nonexistent").is_err());

    Ok(())
}

/// 测试 FormResult 结构体的克隆功能
///
/// ## 测试目的
/// 验证 FormResult 结构体能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_form_result_clone_with_valid_result_creates_clone() {
    // Arrange: 准备 FormResult 实例

    // Act: 克隆结果
    let mut result1 = FormResult::new();
    result1.values.insert("field1".to_string(), "value1".to_string());
    let result2 = result1.clone();

    // Assert: 验证克隆成功
    assert_eq!(result1.get("field1"), result2.get("field1"));
}

/// 测试 FormResult 结构体的调试格式化
///
/// ## 测试目的
/// 验证 FormResult 结构体能够正确格式化调试字符串。
///
/// ## 预期结果
/// - 调试字符串包含字段信息
#[test]
fn test_form_result_debug_with_valid_result_returns_debug_string() {
    // Arrange: 准备 FormResult 实例

    // Act: 格式化调试字符串
    let mut result = FormResult::new();
    result.values.insert("field1".to_string(), "value1".to_string());
    let debug_str = format!("{:?}", result);

    // Assert: 验证调试字符串包含字段信息
    assert!(debug_str.contains("field1") || debug_str.contains("value1"));
}

// ==================== StepType 枚举测试 ====================

/// 测试 StepType::Unconditional 变体
///
/// ## 测试目的
/// 验证 StepType 枚举能够创建 Unconditional 变体。
///
/// ## 预期结果
/// - 创建 StepType::Unconditional 变体
#[test]
fn test_step_type_unconditional_creates_unconditional_step() {
    // Arrange: 准备无条件步骤类型

    // Act: 创建无条件步骤
    let step_type = StepType::Unconditional;

    // Assert: 验证创建了 Unconditional 变体
    match step_type {
        StepType::Unconditional => {}
        _ => panic!("Expected Unconditional variant"),
    }
}

/// 测试 StepType::Conditional 变体
///
/// ## 测试目的
/// 验证 StepType 枚举能够使用单个条件创建 Conditional 变体。
///
/// ## 预期结果
/// - 创建 StepType::Conditional 变体
/// - 条件正确设置
#[test]
fn test_step_type_conditional_with_condition_creates_conditional_step() {
    // Arrange: 准备条件
    let condition = Condition {
        field_name: "field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };

    // Act: 创建单条件步骤
    let step_type = StepType::Conditional(condition.clone());

    // Assert: 验证创建了 Conditional 变体
    match step_type {
        StepType::Conditional(c) => {
            assert_eq!(c.field_name, condition.field_name);
        }
        _ => panic!("Expected Conditional variant"),
    }
}

/// 测试 StepType::ConditionalAll 变体
///
/// ## 测试目的
/// 验证 StepType 枚举能够使用多个条件（AND 逻辑）创建 ConditionalAll 变体。
///
/// ## 预期结果
/// - 创建 StepType::ConditionalAll 变体
/// - 所有条件正确设置
#[test]
fn test_step_type_conditional_all_with_conditions_creates_conditional_all_step() {
    // Arrange: 准备多个条件（AND 逻辑）
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

    // Act: 创建多条件步骤（AND）
    let step_type = StepType::ConditionalAll(conditions.clone());

    // Assert: 验证创建了 ConditionalAll 变体
    match step_type {
        StepType::ConditionalAll(cs) => {
            assert_eq!(cs.len(), 2);
        }
        _ => panic!("Expected ConditionalAll variant"),
    }
}

/// 测试 StepType::ConditionalAny 变体
///
/// ## 测试目的
/// 验证 StepType 枚举能够使用多个条件（OR 逻辑）创建 ConditionalAny 变体。
///
/// ## 预期结果
/// - 创建 StepType::ConditionalAny 变体
/// - 所有条件正确设置
#[test]
fn test_step_type_conditional_any_with_conditions_creates_conditional_any_step() {
    // Arrange: 准备多个条件（OR 逻辑）
    let conditions = vec![Condition {
        field_name: "field1".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value1"),
    }];

    // Act: 创建多条件步骤（OR）
    let step_type = StepType::ConditionalAny(conditions.clone());

    // Assert: 验证创建了 ConditionalAny 变体
    match step_type {
        StepType::ConditionalAny(cs) => {
            assert_eq!(cs.len(), 1);
        }
        _ => panic!("Expected ConditionalAny variant"),
    }
}

/// 测试 StepType::DynamicCondition 变体
///
/// ## 测试目的
/// 验证 StepType 枚举能够使用动态条件函数创建 DynamicCondition 变体。
///
/// ## 预期结果
/// - 创建 StepType::DynamicCondition 变体
/// - 函数正确设置
#[test]
fn test_step_type_dynamic_condition_with_function_creates_dynamic_step() {
    // Arrange: 准备动态条件函数

    // Act: 创建动态条件步骤
    let step_type = StepType::DynamicCondition(Box::new(|_result: &FormResult| true));

    // Assert: 验证创建了 DynamicCondition 变体
    match step_type {
        StepType::DynamicCondition(_) => {}
        _ => panic!("Expected DynamicCondition variant"),
    }
}

// ==================== FormStep 结构体测试 ====================

/// 测试 FormStep 结构体的创建
///
/// ## 测试目的
/// 验证 FormStep 结构体能够使用有效字段正确创建。
///
/// ## 预期结果
/// - 所有字段正确设置
/// - ID、step_type、fields 等字段正确
#[test]
fn test_form_step_creation_with_valid_fields_creates_step() {
    // Arrange: 准备步骤字段（覆盖 types.rs:194-203）

    // Act: 创建 FormStep
    let step = FormStep {
        id: Some("step1".to_string()),
        step_type: StepType::Unconditional,
        fields: vec![],
        skip_if_false: false,
    };

    // Assert: 验证步骤字段正确
    assert_eq!(step.id, Some("step1".to_string()));
    assert_eq!(step.fields.len(), 0);
    assert!(!step.skip_if_false);
}

/// 测试 FormStep 结构体创建不带ID的步骤
///
/// ## 测试目的
/// 验证 FormStep 结构体能够创建不带ID的步骤。
///
/// ## 预期结果
/// - ID 为 None
/// - 其他字段正确设置
#[test]
fn test_form_step_creation_without_id_creates_step_without_id() {
    // Arrange: 准备没有 ID 的步骤字段

    // Act: 创建没有 ID 的 FormStep
    let step = FormStep {
        id: None,
        step_type: StepType::Unconditional,
        fields: vec![],
        skip_if_false: true,
    };

    // Assert: 验证步骤字段正确
    assert_eq!(step.id, None);
    assert!(step.skip_if_false);
}

// ==================== GroupConfig 结构体测试 ====================

/// 测试 GroupConfig::required() 方法创建必填配置
///
/// ## 测试目的
/// 验证 GroupConfig::required() 方法能够创建必填组配置。
///
/// ## 预期结果
/// - optional 字段为 false
/// - 其他字段为默认值
#[test]
fn test_group_config_required_creates_required_config() {
    // Arrange: 准备创建必填组配置（覆盖 types.rs:222-229）

    // Act: 创建必填组配置
    let config = GroupConfig::required();

    // Assert: 验证配置正确
    assert!(!config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

/// 测试 GroupConfig::optional() 方法创建可选配置
///
/// ## 测试目的
/// 验证 GroupConfig::optional() 方法能够创建可选组配置。
///
/// ## 预期结果
/// - optional 字段为 true
/// - 其他字段为默认值
#[test]
fn test_group_config_optional_creates_optional_config() {
    // Arrange: 准备创建可选组配置（覆盖 types.rs:232-239）

    // Act: 创建可选组配置
    let config = GroupConfig::optional();

    // Assert: 验证配置正确
    assert!(config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

/// 测试 GroupConfig::with_title() 方法设置标题（字符串切片）
///
/// ## 测试目的
/// 验证 GroupConfig::with_title() 方法能够使用字符串切片设置标题。
///
/// ## 预期结果
/// - title 字段正确设置
#[test]
fn test_group_config_with_title_with_str_sets_title() {
    // Arrange: 准备标题字符串（覆盖 types.rs:242-245）

    // Act: 设置组标题
    let config = GroupConfig::required().with_title("Test Title");

    // Assert: 验证标题设置成功
    assert_eq!(config.title, Some("Test Title".to_string()));
}

/// 测试 GroupConfig::with_title() 方法设置标题（String类型）
///
/// ## 测试目的
/// 验证 GroupConfig::with_title() 方法能够使用 String 类型设置标题。
///
/// ## 预期结果
/// - title 字段正确设置
#[test]
fn test_group_config_with_title_with_string_sets_title() {
    // Arrange: 准备 String 类型的标题

    // Act: 设置组标题（接受 String）
    let config = GroupConfig::required().with_title("Test Title".to_string());

    // Assert: 验证标题设置成功
    assert_eq!(config.title, Some("Test Title".to_string()));
}

/// 测试 GroupConfig::with_description() 方法设置描述
///
/// ## 测试目的
/// 验证 GroupConfig::with_description() 方法能够使用字符串切片设置描述。
///
/// ## 预期结果
/// - description 字段正确设置
#[test]
fn test_group_config_with_description_with_str_sets_description() {
    // Arrange: 准备描述字符串（覆盖 types.rs:248-251）

    // Act: 设置组描述
    let config = GroupConfig::required().with_description("Test Description");

    // Assert: 验证描述设置成功
    assert_eq!(config.description, Some("Test Description".to_string()));
}

/// 测试 GroupConfig::with_default_enabled() 方法设置默认启用状态
///
/// ## 测试目的
/// 验证 GroupConfig::with_default_enabled() 方法能够设置组的默认启用状态。
///
/// ## 预期结果
/// - default_enabled 字段正确设置
#[test]
fn test_group_config_with_default_enabled_with_bool_sets_default_enabled() {
    // Arrange: 准备布尔值（覆盖 types.rs:254-257）

    // Act: 设置默认启用
    let config = GroupConfig::optional().with_default_enabled(true);
    let config2 = GroupConfig::optional().with_default_enabled(false);

    // Assert: 验证默认启用设置成功
    assert!(config.default_enabled);
    assert!(!config2.default_enabled);
}

/// 测试 GroupConfig 的链式调用
///
/// ## 测试目的
/// 验证 GroupConfig 的所有方法能够链式调用，配置所有选项。
///
/// ## 预期结果
/// - 所有选项都能通过链式调用正确配置
/// - optional、title、description、default_enabled 都正确设置
#[test]
fn test_group_config_chain_calls_with_all_methods_configures_all_options() {
    // Arrange: 准备所有配置选项

    // Act: 链式调用所有方法
    let config = GroupConfig::optional()
        .with_title("Title")
        .with_description("Description")
        .with_default_enabled(true);

    // Assert: 验证所有选项配置成功
    assert!(config.optional);
    assert_eq!(config.title, Some("Title".to_string()));
    assert_eq!(config.description, Some("Description".to_string()));
    assert!(config.default_enabled);
}

/// 测试 GroupConfig::default() 方法创建默认配置
///
/// ## 测试目的
/// 验证 GroupConfig 的 Default trait 实现能够创建默认配置。
///
/// ## 预期结果
/// - 创建默认配置（optional=false，其他字段为None或false）
#[test]
fn test_group_config_default_creates_default_config() {
    // Arrange: 准备使用 Default trait（覆盖 types.rs:260-263）

    // Act: 创建默认配置
    let config = GroupConfig::default();

    // Assert: 验证默认配置正确
    assert!(!config.optional);
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(!config.default_enabled);
}

/// 测试 GroupConfig 结构体的克隆功能
///
/// ## 测试目的
/// 验证 GroupConfig 结构体能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
#[test]
fn test_group_config_clone_with_valid_config_creates_clone() {
    // Arrange: 准备 GroupConfig 实例

    // Act: 克隆配置
    let config1 = GroupConfig::optional().with_title("Title");
    let config2 = config1.clone();

    // Assert: 验证克隆成功
    assert_eq!(config1.title, config2.title);
    assert_eq!(config1.optional, config2.optional);
}

/// 测试 GroupConfig 结构体的调试格式化
///
/// ## 测试目的
/// 验证 GroupConfig 结构体能够正确格式化调试字符串。
///
/// ## 预期结果
/// - 调试字符串包含配置信息
#[test]
fn test_group_config_debug_with_valid_config_returns_debug_string() {
    // Arrange: 准备 GroupConfig 实例

    // Act: 格式化调试字符串
    let config = GroupConfig::required().with_title("Title");
    let debug_str = format!("{:?}", config);

    // Assert: 验证调试字符串包含配置信息
    assert!(debug_str.contains("Title") || debug_str.contains("optional"));
}

// ==================== FormGroup 结构体测试 ====================

/// 测试 FormGroup 结构体的创建
///
/// ## 测试目的
/// 验证 FormGroup 结构体能够使用有效字段正确创建。
///
/// ## 预期结果
/// - 所有字段正确设置
/// - id、title、description、optional、default_enabled、steps 等字段正确
#[test]
fn test_form_group_creation_with_valid_fields_creates_group() {
    // Arrange: 准备组字段（覆盖 types.rs:267-280）

    // Act: 创建 FormGroup
    let group = FormGroup {
        id: "group1".to_string(),
        title: Some("Group Title".to_string()),
        description: Some("Group Description".to_string()),
        optional: false,
        default_enabled: false,
        steps: vec![],
    };

    // Assert: 验证组字段正确
    assert_eq!(group.id, "group1");
    assert_eq!(group.title, Some("Group Title".to_string()));
    assert_eq!(group.description, Some("Group Description".to_string()));
    assert!(!group.optional);
    assert!(!group.default_enabled);
    assert_eq!(group.steps.len(), 0);
}

/// 测试 FormGroup 结构体创建可选组
///
/// ## 测试目的
/// 验证 FormGroup 结构体能够创建可选组配置。
///
/// ## 预期结果
/// - optional 字段为 true
/// - default_enabled 字段为 true
/// - 其他字段正确设置
#[test]
fn test_form_group_creation_with_optional_config_creates_optional_group() {
    // Arrange: 准备可选组字段

    // Act: 创建可选组
    let group = FormGroup {
        id: "group2".to_string(),
        title: None,
        description: None,
        optional: true,
        default_enabled: true,
        steps: vec![],
    };

    // Assert: 验证可选组配置正确
    assert!(group.optional);
    assert!(group.default_enabled);
}

// ==================== FormField 结构体测试 ====================

/// 测试 FormField 结构体的创建
///
/// ## 测试目的
/// 验证 FormField 结构体能够使用有效字段正确创建。
///
/// ## 预期结果
/// - 所有字段正确设置
/// - name、field_type、message、choices、default_choice、default_value、required、allow_empty、validator、condition 等字段正确
#[test]
fn test_form_field_creation_with_valid_fields_creates_field() {
    // Arrange: 准备字段属性（覆盖 types.rs:97-121）

    // Act: 创建 FormField
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

    // Assert: 验证字段属性正确
    assert_eq!(field.name, "field1");
    assert_eq!(field.field_type, FormFieldType::Text);
    assert_eq!(field.message, "Enter value");
    assert!(!field.required);
    assert!(field.allow_empty);
}

/// 测试 FormField 结构体创建带条件的字段
///
/// ## 测试目的
/// 验证 FormField 结构体能够创建带条件的字段。
///
/// ## 预期结果
/// - condition 字段正确设置
/// - 其他字段正确设置
#[test]
fn test_form_field_creation_with_condition_creates_field_with_condition() {
    // Arrange: 准备条件和字段属性
    let condition = Condition {
        field_name: "other_field".to_string(),
        operator: ConditionOperator::Equals,
        value: ConditionValue::single("value"),
    };

    // Act: 创建带条件的字段
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

    // Assert: 验证字段和条件正确
    assert!(field.required);
    assert!(!field.allow_empty);
    assert_eq!(
        field.condition.as_ref().expect("condition should exist").field_name,
        condition.field_name
    );
}

/// 测试 FormField 结构体的克隆功能
///
/// ## 测试目的
/// 验证 FormField 结构体能够正确克隆。
///
/// ## 预期结果
/// - 克隆后的值与原值相等
/// - 所有字段都正确克隆
#[test]
fn test_form_field_clone_with_valid_field_creates_clone() {
    // Arrange: 准备 FormField 实例

    // Act: 克隆字段
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

    // Assert: 验证克隆成功
    assert_eq!(field1.name, field2.name);
    assert_eq!(field1.field_type, field2.field_type);
}
