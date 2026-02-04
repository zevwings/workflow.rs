//! Group 构建器，用于在 Group 内构建 Step

use crate::form::builder::step_builder::StepBuilder;
use crate::form::types::{FormResult, FormStep, StepType};

/// Group 构建器
///
/// 用于在 Group 内构建 Step，提供 `step`、`step_if` 等方法。
pub struct GroupBuilder {
    /// 组内的步骤
    steps: Vec<FormStep>,
}

impl GroupBuilder {
    /// 创建新的 Group 构建器
    pub(crate) fn new(_group_id: &str) -> Self {
        Self { steps: Vec::new() }
    }

    /// 构建步骤的公共逻辑
    fn build_step<F>(&mut self, builder: F, step_type: StepType)
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
    {
        let temp_builder = StepBuilder::new();
        let built = builder(temp_builder);

        let step = FormStep {
            step_type,
            fields: built.into_fields(),
        };

        self.steps.push(step);
    }

    /// 添加一个无条件步骤（总是执行）
    pub fn add_step<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
    {
        self.build_step(builder, StepType::Unconditional);
        self
    }

    /// 添加一个条件步骤（基于条件函数）
    pub fn add_step_if<F, G>(mut self, condition_fn: G, builder: F) -> Self
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
        G: Fn(&FormResult) -> bool + Send + Sync + 'static,
    {
        self.build_step(builder, StepType::Conditional(Box::new(condition_fn)));
        self
    }

    /// 添加一个条件步骤（基于字段名和值，简化版）
    ///
    /// 这是一个便捷方法，用于检查字段值是否等于指定值。
    pub fn step_if<F>(
        self,
        field_name: impl Into<String>,
        value: impl Into<String>,
        builder: F,
    ) -> Self
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
    {
        let field_name = field_name.into();
        let value = value.into();
        self.add_step_if(
            move |result| {
                result
                    .get_raw(&field_name)
                    .and_then(|v| v.downcast_ref::<String>())
                    .map(|v| v == &value)
                    .unwrap_or(false)
            },
            builder,
        )
    }

    /// 添加一个多条件步骤（所有条件都满足，AND 逻辑）
    pub fn add_step_if_all<F, I, G>(mut self, conditions: I, builder: F) -> Self
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
        I: IntoIterator<Item = G>,
        G: Fn(&FormResult) -> bool + Send + Sync + 'static,
    {
        type ConditionFn = Box<dyn Fn(&FormResult) -> bool + Send + Sync>;
        let conditions: Vec<ConditionFn> =
            conditions.into_iter().map(|c| Box::new(c) as ConditionFn).collect();

        self.build_step(builder, StepType::ConditionalAll(conditions));
        self
    }

    /// 添加一个多条件步骤（任一条件满足，OR 逻辑）
    pub fn add_step_if_any<F, I, G>(mut self, conditions: I, builder: F) -> Self
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
        I: IntoIterator<Item = G>,
        G: Fn(&FormResult) -> bool + Send + Sync + 'static,
    {
        type ConditionFn = Box<dyn Fn(&FormResult) -> bool + Send + Sync>;
        let conditions: Vec<ConditionFn> =
            conditions.into_iter().map(|c| Box::new(c) as ConditionFn).collect();

        self.build_step(builder, StepType::ConditionalAny(conditions));
        self
    }

    /// 添加一个动态条件步骤（基于运行时值）
    pub fn add_step_if_dynamic<F, G>(mut self, condition_fn: G, builder: F) -> Self
    where
        F: FnOnce(StepBuilder) -> StepBuilder,
        G: Fn(&FormResult) -> bool + Send + Sync + 'static,
    {
        self.build_step(builder, StepType::DynamicCondition(Box::new(condition_fn)));
        self
    }

    /// 获取组内的步骤（用于 FormBuilder）
    pub(crate) fn into_steps(self) -> Vec<FormStep> {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form::field::{ConfirmFormField, FieldType, InputFormField};

    #[test]
    fn test_group_builder_new() {
        let builder = GroupBuilder::new("test_group");
        let steps = builder.into_steps();
        assert!(steps.is_empty());
    }

    #[test]
    fn test_group_builder_add_step() {
        let builder = GroupBuilder::new("group1")
            .add_step(|s| s.add_input(InputFormField::new("name", "Name")));

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 1);
        assert!(matches!(steps[0].step_type, StepType::Unconditional));
        assert_eq!(steps[0].fields.len(), 1);
    }

    #[test]
    fn test_group_builder_add_multiple_steps() {
        let builder = GroupBuilder::new("group1")
            .add_step(|s| s.add_input(InputFormField::new("field1", "Field 1")))
            .add_step(|s| s.add_input(InputFormField::new("field2", "Field 2")))
            .add_step(|s| s.add_confirm(ConfirmFormField::new("confirm", "Confirm?")));

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_group_builder_add_step_if() {
        let builder = GroupBuilder::new("group1").add_step_if(
            |result| result.get_bool("enabled"),
            |s| s.add_input(InputFormField::new("conditional", "Conditional field")),
        );

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 1);
        assert!(matches!(steps[0].step_type, StepType::Conditional(_)));
    }

    #[test]
    fn test_group_builder_step_if_string_value() {
        let builder = GroupBuilder::new("group1")
            .add_step(|s| s.add_input(InputFormField::new("mode", "Select mode")))
            .step_if("mode", "advanced", |s| {
                s.add_input(InputFormField::new("advanced_option", "Advanced option"))
            });

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 2);

        // 第一个步骤是无条件的
        assert!(matches!(steps[0].step_type, StepType::Unconditional));
        // 第二个步骤是条件的
        assert!(matches!(steps[1].step_type, StepType::Conditional(_)));
    }

    #[test]
    fn test_group_builder_add_step_if_all() {
        let conditions = vec![
            |result: &FormResult| result.get_bool("a"),
            |result: &FormResult| result.get_bool("b"),
        ];

        let builder = GroupBuilder::new("group1").add_step_if_all(conditions, |s| {
            s.add_input(InputFormField::new("combined", "Both conditions met"))
        });

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 1);
        assert!(matches!(steps[0].step_type, StepType::ConditionalAll(_)));
    }

    #[test]
    fn test_group_builder_add_step_if_any() {
        let conditions = vec![
            |result: &FormResult| result.get_bool("option_a"),
            |result: &FormResult| result.get_bool("option_b"),
        ];

        let builder = GroupBuilder::new("group1").add_step_if_any(conditions, |s| {
            s.add_input(InputFormField::new("either", "Either condition met"))
        });

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 1);
        assert!(matches!(steps[0].step_type, StepType::ConditionalAny(_)));
    }

    #[test]
    fn test_group_builder_add_step_if_dynamic() {
        let builder = GroupBuilder::new("group1").add_step_if_dynamic(
            |result| result.get_int("count") > 5,
            |s| s.add_input(InputFormField::new("extra", "Extra info")),
        );

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 1);
        assert!(matches!(steps[0].step_type, StepType::DynamicCondition(_)));
    }

    #[test]
    fn test_group_builder_mixed_step_types() {
        let builder = GroupBuilder::new("group1")
            .add_step(|s| s.add_confirm(ConfirmFormField::new("start", "Start?")))
            .add_step_if(
                |result| result.get_bool("start"),
                |s| s.add_input(InputFormField::new("name", "Name")),
            )
            .add_step_if_dynamic(
                |result| !result.get_string("name").is_empty(),
                |s| s.add_confirm(ConfirmFormField::new("confirm", "Confirm?")),
            );

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 3);
        assert!(matches!(steps[0].step_type, StepType::Unconditional));
        assert!(matches!(steps[1].step_type, StepType::Conditional(_)));
        assert!(matches!(steps[2].step_type, StepType::DynamicCondition(_)));
    }

    #[test]
    fn test_group_builder_step_with_multiple_fields() {
        let builder = GroupBuilder::new("group1").add_step(|s| {
            s.add_input(InputFormField::new("first_name", "First Name"))
                .add_input(InputFormField::new("last_name", "Last Name"))
                .add_input(InputFormField::new("email", "Email"))
        });

        let steps = builder.into_steps();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].fields.len(), 3);
        assert_eq!(steps[0].fields[0].key, "first_name");
        assert_eq!(steps[0].fields[1].key, "last_name");
        assert_eq!(steps[0].fields[2].key, "email");
    }

    #[test]
    fn test_group_builder_step_if_condition_evaluation() {
        // 测试 step_if 方法生成的条件函数
        let builder = GroupBuilder::new("group1").step_if("mode", "test", |s| {
            s.add_input(InputFormField::new("test_field", "Test field"))
        });

        let steps = builder.into_steps();

        // 获取条件函数并测试
        if let StepType::Conditional(condition) = &steps[0].step_type {
            // 创建一个模拟的 FormResult
            let mut result = FormResult::new();
            result.set("mode".to_string(), "test".to_string());
            assert!(condition(&result));

            // 测试不匹配的值
            let mut result2 = FormResult::new();
            result2.set("mode".to_string(), "other".to_string());
            assert!(!condition(&result2));

            // 测试缺失的字段
            let result3 = FormResult::new();
            assert!(!condition(&result3));
        } else {
            panic!("Expected Conditional step type");
        }
    }

    #[test]
    fn test_group_builder_preserves_field_types() {
        let builder = GroupBuilder::new("group1").add_step(|s| {
            s.add_confirm(ConfirmFormField::new("c1", "Confirm 1"))
                .add_input(InputFormField::new("i1", "Input 1"))
        });

        let steps = builder.into_steps();
        assert_eq!(steps[0].fields[0].field_type, FieldType::Confirm);
        assert_eq!(steps[0].fields[1].field_type, FieldType::Input);
    }
}
