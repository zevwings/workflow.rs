//! 组构建器，用于在组内构建步骤

use crate::base::dialog::form::field_builder::FieldBuilder;
use crate::base::dialog::form::types::{
    Condition, ConditionOperator, ConditionValue, FormStep, StepType,
};

/// 组构建器
///
/// 用于在组内构建步骤，提供 `step`、`step_if` 等方法。
pub struct GroupBuilder {
    /// 组 ID
    _group_id: String,
    /// 组内的步骤
    steps: Vec<FormStep>,
}

impl GroupBuilder {
    /// 创建新的组构建器
    pub(crate) fn new(group_id: &str) -> Self {
        Self {
            _group_id: group_id.to_string(),
            steps: Vec::new(),
        }
    }

    /// 构建步骤的公共逻辑
    ///
    /// 创建一个 FieldBuilder，使用 builder 闭包构建字段，然后创建 FormStep。
    fn build_step<F>(&mut self, builder: F, step_type: StepType, skip_if_false: bool)
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
    {
        let temp_builder = FieldBuilder::new();
        let built = builder(temp_builder);

        let step = FormStep {
            id: None,
            step_type,
            fields: built.fields,
            skip_if_false,
        };

        self.steps.push(step);
    }

    /// 添加一个无条件步骤（总是执行）
    pub fn step<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
    {
        self.build_step(builder, StepType::Unconditional, false);
        self
    }

    /// 添加一个条件步骤（基于单个字段值）
    pub fn step_if<F>(
        mut self,
        field_name: impl Into<String>,
        value: impl Into<String>,
        builder: F,
    ) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
    {
        let condition = Condition {
            field_name: field_name.into(),
            operator: ConditionOperator::Equals,
            value: ConditionValue::single(value.into()),
        };

        self.build_step(builder, StepType::Conditional(condition), true);
        self
    }

    /// 添加一个多条件步骤（所有条件都满足，AND 逻辑）
    pub fn step_if_all<F, I, K, V>(mut self, conditions: I, builder: F) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let conditions: Vec<Condition> = conditions
            .into_iter()
            .map(|(field, value)| Condition {
                field_name: field.into(),
                operator: ConditionOperator::Equals,
                value: ConditionValue::single(value.into()),
            })
            .collect();

        self.build_step(builder, StepType::ConditionalAll(conditions), true);
        self
    }

    /// 添加一个多条件步骤（任一条件满足，OR 逻辑）
    pub fn step_if_any<F, I, K, V>(mut self, conditions: I, builder: F) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let conditions: Vec<Condition> = conditions
            .into_iter()
            .map(|(field, value)| Condition {
                field_name: field.into(),
                operator: ConditionOperator::Equals,
                value: ConditionValue::single(value.into()),
            })
            .collect();

        self.build_step(builder, StepType::ConditionalAny(conditions), true);
        self
    }

    /// 添加一个动态条件步骤（基于运行时值）
    pub fn step_if_dynamic<F, G>(mut self, condition_fn: G, builder: F) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
        G: Fn(&crate::base::dialog::form::FormResult) -> bool + Send + Sync + 'static,
    {
        self.build_step(
            builder,
            StepType::DynamicCondition(Box::new(condition_fn)),
            true,
        );
        self
    }

    /// 获取组内的步骤（用于 FormBuilder）
    pub(crate) fn into_steps(self) -> Vec<FormStep> {
        self.steps
    }
}
