//! 表单构建器的类型定义

use color_eyre::Result;
use std::collections::HashMap;

use crate::base::dialog::types::ValidatorFn;

/// 表单字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldType {
    /// 文本输入字段
    Text,
    /// 密码输入字段（隐藏输入）
    Password,
    /// 选择字段
    Selection,
    /// 确认字段（yes/no）
    Confirmation,
}

/// 条件操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionOperator {
    /// 等于
    Equals,
    /// 不等于
    NotEquals,
    /// 在列表中
    In,
    /// 不在列表中
    NotIn,
}

/// 条件，用于条件字段
#[derive(Debug, Clone)]
pub struct Condition {
    /// 要检查的字段名
    pub field_name: String,
    /// 条件操作符
    pub operator: ConditionOperator,
    /// 要比较的值
    /// 对于 Equals/NotEquals：可以是任何类型（转换为字符串比较）
    /// 对于 In/NotIn：必须是 Vec<String>
    pub value: ConditionValue,
}

/// 条件值类型
#[derive(Debug, Clone)]
pub enum ConditionValue {
    /// 单个值（用于 Equals/NotEquals）
    Single(String),
    /// 多个值（用于 In/NotIn）
    Multiple(Vec<String>),
}

impl ConditionValue {
    /// 创建单个值
    pub fn single(value: impl Into<String>) -> Self {
        Self::Single(value.into())
    }

    /// 创建多个值
    pub fn multiple(values: Vec<String>) -> Self {
        Self::Multiple(values)
    }
}

/// 字段默认值类型
#[derive(Debug, Clone)]
pub enum FieldDefaultValue {
    /// 字符串默认值
    String(String),
    /// 布尔默认值
    Bool(bool),
}

impl FieldDefaultValue {
    /// 获取字符串值
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            Self::Bool(_) => None,
        }
    }

    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::String(_) => None,
            Self::Bool(b) => Some(*b),
        }
    }
}

/// 表单字段
#[derive(Clone)]
pub struct FormField {
    /// 字段名，用作结果映射的键
    pub name: String,
    /// 字段类型
    pub field_type: FormFieldType,
    /// 提示消息
    pub message: String,
    /// 选择字段的选项列表
    pub choices: Vec<String>,
    /// 选择字段的默认选项
    pub default_choice: Option<String>,
    /// 默认值
    /// 对于文本/密码字段：String
    /// 对于确认字段：bool
    /// 对于选择字段：String（同时设置 default_choice）
    pub default_value: Option<FieldDefaultValue>,
    /// 是否必填
    pub required: bool,
    /// 是否允许空值（仅对文本和密码字段有效）
    pub allow_empty: bool,
    /// 可选的验证函数
    pub validator: Option<ValidatorFn>,
    /// 条件：必须满足此条件才会显示此字段
    pub condition: Option<Condition>,
}

/// 表单结果
#[derive(Debug, Clone)]
pub struct FormResult {
    /// 字段值，以字段名为键
    pub values: HashMap<String, String>,
}

impl FormResult {
    /// 创建新的表单结果
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// 获取字段值
    pub fn get(&self, name: &str) -> Option<&String> {
        self.values.get(name)
    }

    /// 获取字段值，如果不存在返回错误
    pub fn get_required(&self, name: &str) -> Result<String> {
        self.values
            .get(name)
            .cloned()
            .ok_or_else(|| color_eyre::eyre::eyre!("Field '{}' is required", name))
    }

    /// 获取字段值，如果不存在返回默认值
    pub fn get_or_default(&self, name: &str, default: impl Into<String>) -> String {
        self.values.get(name).cloned().unwrap_or_else(|| default.into())
    }

    /// 检查字段是否存在
    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// 获取布尔值（将 "yes" 转换为 true，"no" 转换为 false）
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).map(|v| v == "yes")
    }

    /// 获取布尔值，如果不存在返回错误
    pub fn get_required_bool(&self, name: &str) -> Result<bool> {
        self.get_bool(name)
            .ok_or_else(|| color_eyre::eyre::eyre!("Field '{}' is required", name))
    }
}

impl Default for FormResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 步骤类型（用于流式 API）
pub enum StepType {
    /// 无条件步骤（总是执行）
    Unconditional,
    /// 单条件步骤（基于单个字段值）
    Conditional(Condition),
    /// 多条件步骤（AND 逻辑）
    ConditionalAll(Vec<Condition>),
    /// 多条件步骤（OR 逻辑）
    ConditionalAny(Vec<Condition>),
    /// 动态条件步骤（基于运行时值）
    DynamicCondition(Box<dyn Fn(&FormResult) -> bool + Send + Sync>),
}

/// 表单步骤
pub struct FormStep {
    /// 步骤 ID（可选，用于调试和日志）
    pub id: Option<String>,
    /// 步骤类型
    pub step_type: StepType,
    /// 步骤中的字段
    pub fields: Vec<FormField>,
    /// 是否跳过（如果条件不满足）
    pub skip_if_false: bool,
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
    /// 组 ID
    pub id: String,
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
