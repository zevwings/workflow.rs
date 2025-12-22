use color_eyre::Result;
use std::sync::Arc;

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

/// 表单字段
#[derive(Debug, Clone)]
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
    /// 可选的验证函数
    pub validator: Option<ValidatorFn>,
    /// 条件：必须满足此条件才会显示此字段
    pub condition: Option<Condition>,
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

/// 条件组，表示一组条件字段
#[derive(Debug, Clone)]
pub struct ConditionalGroup {
    /// 条件
    pub condition: Condition,
    /// 组内的字段
    pub fields: Vec<FormField>,
    /// 嵌套的条件组
    pub nested_groups: Vec<ConditionalGroup>,
}

/// 表单结果
#[derive(Debug, Clone)]
pub struct FormResult {
    /// 字段值，以字段名为键
    pub values: std::collections::HashMap<String, String>,
}

impl FormResult {
    /// 创建新的表单结果
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
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
        self.values
            .get(name)
            .cloned()
            .unwrap_or_else(|| default.into())
    }

    /// 检查字段是否存在
    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
}

impl Default for FormResult {
    fn default() -> Self {
        Self::new()
    }
}
