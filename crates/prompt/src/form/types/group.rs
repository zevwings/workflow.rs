//! Group 和 Step 相关类型定义

use crate::form::{field::FormField, types::FormResult};

/// 条件函数类型
type ConditionFn = Box<dyn Fn(&FormResult) -> bool + Send + Sync>;

/// 步骤类型
#[allow(clippy::type_complexity)]
pub enum StepType {
    /// 无条件步骤（总是执行）
    Unconditional,
    /// 单条件步骤（基于条件函数）
    Conditional(ConditionFn),
    /// 多条件步骤（AND 逻辑，所有条件都满足）
    ConditionalAll(Vec<ConditionFn>),
    /// 多条件步骤（OR 逻辑，任一条件满足）
    ConditionalAny(Vec<ConditionFn>),
    /// 动态条件步骤（基于运行时值）
    DynamicCondition(ConditionFn),
}

/// 表单步骤
pub struct FormStep {
    /// 步骤类型
    pub step_type: StepType,
    /// 步骤中的字段
    pub fields: Vec<FormField>,
}

/// 组配置
///
/// 用于配置表单组的显示和行为选项。
#[derive(Clone, Debug)]
pub struct GroupConfig {
    /// 是否可选组（可选组会先询问用户是否配置）
    pub optional: bool,
    /// 组的标题（用于显示，可选）
    pub title: Option<String>,
    /// 组的描述（用于显示，可选）
    pub description: Option<String>,
    /// 默认是否启用（当 optional 为 true 时有效）
    pub default_enabled: bool,
}

impl GroupConfig {
    /// 创建必填组配置
    pub fn required() -> Self {
        Self {
            optional: false,
            title: None,
            description: None,
            default_enabled: false,
        }
    }

    /// 创建可选组配置
    pub fn optional() -> Self {
        Self {
            optional: true,
            title: None,
            description: None,
            default_enabled: false,
        }
    }

    /// 设置组标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置组描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 设置默认是否启用（仅对可选组有效）
    pub fn with_default_enabled(mut self, enabled: bool) -> Self {
        self.default_enabled = enabled;
        self
    }
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self::required()
    }
}

/// 表单组
pub struct FormGroup {
    /// 组标题（用于显示）
    pub title: Option<String>,
    /// 组描述（用于显示）
    pub description: Option<String>,
    /// 是否可选
    pub optional: bool,
    /// 默认是否启用（当 optional 为 true 时）
    pub default_enabled: bool,
    /// 组内的步骤
    pub steps: Vec<FormStep>,
}
