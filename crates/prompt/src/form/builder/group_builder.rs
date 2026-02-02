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
        let conditions: Vec<ConditionFn> = conditions
            .into_iter()
            .map(|c| Box::new(c) as ConditionFn)
            .collect();

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
        let conditions: Vec<ConditionFn> = conditions
            .into_iter()
            .map(|c| Box::new(c) as ConditionFn)
            .collect();

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
