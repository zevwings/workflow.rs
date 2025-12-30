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

#[cfg(test)]
mod tests {
    use crate::base::dialog::{FormBuilder, GroupConfig};

    // ==================== FormGroupBuilder Step Tests ====================

    /// 测试FormGroupBuilder添加无条件步骤
    ///
    /// ## 测试目的
    /// 验证 `FormGroupBuilder::step()` 方法能够添加无条件步骤（通过FormBuilder间接测试）。
    ///
    /// ## 测试场景
    /// 1. 创建FormBuilder并添加组
    /// 2. 在组中添加无条件步骤
    /// 3. 验证步骤添加成功
    ///
    /// ## 注意事项
    /// - GroupBuilder::new是pub(crate)，通过FormBuilder间接测试
    ///
    /// ## 预期结果
    /// - 步骤添加成功，不会panic
    #[test]
    fn test_group_builder_step_with_unconditional_step_adds_step() {
        // Arrange: 准备FormBuilder
        // 注意：GroupBuilder::new是pub(crate)，我们通过FormBuilder间接测试

        // Act: 添加无条件步骤
        let _builder = FormBuilder::new().add_group(
            "test",
            |g| g.step(|f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证可以创建组
    }

    /// 测试FormGroupBuilder添加条件步骤
    ///
    /// ## 测试目的
    /// 验证 `FormGroupBuilder::step_if()` 方法能够添加条件步骤（当指定字段等于指定值时显示）。
    ///
    /// ## 测试场景
    /// 1. 创建FormBuilder并添加组
    /// 2. 在组中添加条件步骤（field1 == "value1"）
    /// 3. 验证步骤添加成功
    ///
    /// ## 预期结果
    /// - 条件步骤添加成功，不会panic
    #[test]
    fn test_group_builder_step_if_with_condition_adds_conditional_step() {
        // Arrange: 准备FormBuilder和条件
        let field_name = "field1";
        let field_value = "value1";

        // Act: 添加条件步骤
        let _builder = FormBuilder::new().add_group(
            "test",
            |g| g.step_if(field_name, field_value, |f| f.add_text("field2", "Field 2")),
            GroupConfig::required(),
        );

        // Assert: 验证可以创建条件步骤
    }

    /// 测试FormGroupBuilder添加多条件步骤（AND逻辑）
    ///
    /// ## 测试目的
    /// 验证 `FormGroupBuilder::step_if_all()` 方法能够添加多条件步骤（所有条件都满足时显示，AND逻辑）。
    ///
    /// ## 测试场景
    /// 1. 创建FormBuilder并添加组
    /// 2. 在组中添加多条件步骤（field1 == "value1" AND field2 == "value2"）
    /// 3. 验证步骤添加成功
    ///
    /// ## 预期结果
    /// - 多条件步骤添加成功，不会panic
    #[test]
    fn test_group_builder_step_if_all_with_all_conditions_adds_step() {
        // Arrange: 准备FormBuilder和多个条件（AND逻辑）
        let conditions = vec![("field1", "value1"), ("field2", "value2")];

        // Act: 添加多条件步骤（AND逻辑）
        let _builder = FormBuilder::new().add_group(
            "test",
            |g| g.step_if_all(conditions, |f| f.add_text("field3", "Field 3")),
            GroupConfig::required(),
        );

        // Assert: 验证可以创建多条件步骤
    }

    /// 测试FormGroupBuilder添加多条件步骤（OR逻辑）
    ///
    /// ## 测试目的
    /// 验证 `FormGroupBuilder::step_if_any()` 方法能够添加多条件步骤（任一条件满足时显示，OR逻辑）。
    ///
    /// ## 测试场景
    /// 1. 创建FormBuilder并添加组
    /// 2. 在组中添加多条件步骤（field1 == "value1" OR field2 == "value2"）
    /// 3. 验证步骤添加成功
    ///
    /// ## 预期结果
    /// - 多条件步骤添加成功，不会panic
    #[test]
    fn test_group_builder_step_if_any_with_any_condition_adds_step() {
        // Arrange: 准备FormBuilder和多个条件（OR逻辑）
        let conditions = vec![("field1", "value1"), ("field2", "value2")];

        // Act: 添加多条件步骤（OR逻辑）
        let _builder = FormBuilder::new().add_group(
            "test",
            |g| g.step_if_any(conditions, |f| f.add_text("field3", "Field 3")),
            GroupConfig::required(),
        );

        // Assert: 验证可以创建多条件步骤
    }

    /// 测试FormGroupBuilder添加动态条件步骤
    ///
    /// ## 测试目的
    /// 验证 `FormGroupBuilder::step_if_dynamic()` 方法能够添加动态条件步骤（使用函数动态判断条件）。
    ///
    /// ## 测试场景
    /// 1. 创建FormBuilder并添加组
    /// 2. 在组中添加动态条件步骤（使用函数判断条件）
    /// 3. 验证步骤添加成功
    ///
    /// ## 预期结果
    /// - 动态条件步骤添加成功，不会panic
    #[test]
    fn test_group_builder_step_if_dynamic_with_dynamic_condition_adds_step() {
        // Arrange: 准备FormBuilder和动态条件函数

        // Act: 添加动态条件步骤
        let _builder = FormBuilder::new().add_group(
            "test",
            |g| g.step_if_dynamic(|_result| true, |f| f.add_text("field1", "Field 1")),
            GroupConfig::required(),
        );

        // Assert: 验证可以创建动态条件步骤
    }

    /// 测试FormGroupBuilder添加多个步骤
    ///
    /// ## 测试目的
    /// 验证 `FormGroupBuilder` 能够链式调用添加多个步骤（包括无条件步骤和条件步骤）。
    ///
    /// ## 测试场景
    /// 1. 创建FormBuilder并添加组
    /// 2. 在组中链式添加多个步骤（无条件步骤和条件步骤）
    /// 3. 验证所有步骤添加成功
    ///
    /// ## 预期结果
    /// - 所有步骤添加成功，不会panic
    /// - 链式调用正常工作
    #[test]
    fn test_group_builder_multiple_steps_with_multiple_steps_adds_all_steps() {
        // Arrange: 准备FormBuilder

        // Act: 添加多个步骤
        let _builder = FormBuilder::new().add_group(
            "test",
            |g| {
                g.step(|f| f.add_text("field1", "Field 1"))
                    .step(|f| f.add_text("field2", "Field 2"))
                    .step_if("field1", "value1", |f| f.add_text("field3", "Field 3"))
            },
            GroupConfig::required(),
        );

        // Assert: 验证可以添加多个步骤
    }
}
