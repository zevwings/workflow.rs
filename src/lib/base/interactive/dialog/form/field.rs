//! 表单字段定义

use crate::base::interactive::dialog::form::result::FormResult;
use crate::base::interactive::dialog::input::Validator;
use std::sync::Arc;

/// 字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// 确认字段（bool）
    Confirm,
    /// 文本输入字段（String）
    Input,
    /// 密码输入字段（String）
    Password,
    /// 单选字段（usize）
    Select,
    /// 多选字段（Vec<usize>）
    MultiSelect,
    /// 嵌套表单字段（FormResult）
    Form,
}

/// 条件函数类型
/// 基于前面字段的值决定是否执行当前字段
pub type Condition = Box<dyn Fn(&FormResult) -> bool + Send + Sync>;

/// 表单字段定义
pub struct FormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 字段类型
    pub field_type: FieldType,
    /// 提示消息
    pub prompt: String,
    /// 默认值（可选）
    pub default_value: Option<Box<dyn std::any::Any + Send + Sync>>,
    /// 验证器（可选，仅用于 input/password 字段）
    /// 使用 Arc 以便可以克隆并在多个地方使用
    pub validator: Option<Arc<dyn Validator + Send + Sync>>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 嵌套表单（仅用于 FieldType::Form）
    pub nested_form: Option<crate::base::interactive::dialog::form::builder::FormBuilder>,
    /// 选项列表（仅用于 FieldType::Select 和 FieldType::MultiSelect）
    pub options: Vec<String>,
    /// 默认选中的索引（仅用于 FieldType::Select）
    pub default_index: Option<usize>,
    /// 默认选中的索引列表（仅用于 FieldType::MultiSelect）
    pub default_selected: Vec<usize>,
}

/// 确认字段配置
pub struct ConfirmFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值
    pub default_value: bool,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

/// 输入字段配置
pub struct InputFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值（可选）
    pub default_value: String,
    /// 验证器（可选）
    /// 使用 Arc 以便可以克隆并在多个地方使用
    pub validator: Option<Arc<dyn Validator + Send + Sync>>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

/// 密码字段配置
pub struct PasswordFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 默认值（可选，空字符串表示无默认值）
    pub default_value: String,
    /// 验证器（可选）
    /// 使用 Arc 以便可以克隆并在多个地方使用
    pub validator: Option<Arc<dyn Validator + Send + Sync>>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

/// 单选字段配置
pub struct SelectFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 选项列表
    pub options: Vec<String>,
    /// 默认选中的索引
    pub default_index: usize,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

/// 多选字段配置
pub struct MultiSelectFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 选项列表
    pub options: Vec<String>,
    /// 默认选中的索引列表
    pub default_selected: Vec<usize>,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

/// 嵌套表单字段配置
pub struct NestedFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 嵌套表单
    pub nested_form: crate::base::interactive::dialog::form::builder::FormBuilder,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}
