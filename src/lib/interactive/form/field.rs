//! 表单字段定义

use crate::interactive::dialog::Validator;
use crate::interactive::form::result::FormResult;
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
    pub nested_form: Option<crate::interactive::form::builder::FormBuilder>,
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

impl ConfirmFormField {
    /// 创建新的确认字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            default_value: false,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认值
    pub fn default(mut self, value: bool) -> Self {
        self.default_value = value;
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
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

impl InputFormField {
    /// 创建新的输入字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            default_value: String::new(),
            validator: None,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认值
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_value = value.into();
        self
    }

    /// 设置验证器
    pub fn validator(mut self, validator: Arc<dyn Validator + Send + Sync>) -> Self {
        self.validator = Some(validator);
        self
    }

    /// 标记字段为必填（兼容旧 API）
    pub fn required(mut self) -> Self {
        let key = self.key.clone();
        let validator = Arc::new(move |input: &str| {
            if input.trim().is_empty() {
                Err(format!("Field '{}' is required", key))
            } else {
                Ok(())
            }
        });
        self.validator = Some(validator);
        self
    }

    /// 允许字段为空（兼容旧 API）
    /// 注意：新模块默认允许空值，这个方法主要用于兼容性
    pub fn allow_empty(self, _allow: bool) -> Self {
        // 新模块默认允许空值，如果需要必填，使用 required() 方法
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
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

impl PasswordFormField {
    /// 创建新的密码字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            default_value: String::new(),
            validator: None,
            result_title: None,
            condition: None,
        }
    }

    /// 标记字段为必填（兼容旧 API）
    pub fn required(mut self) -> Self {
        let key = self.key.clone();
        let validator = Arc::new(move |input: &str| {
            if input.trim().is_empty() {
                Err(format!("Field '{}' is required", key))
            } else {
                Ok(())
            }
        });
        self.validator = Some(validator);
        self
    }

    /// 允许字段为空（兼容旧 API）
    /// 注意：新模块默认允许空值，这个方法主要用于兼容性
    pub fn allow_empty(self, _allow: bool) -> Self {
        // 新模块默认允许空值，如果需要必填，使用 required() 方法
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
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

impl SelectFormField {
    /// 创建新的单选字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            options,
            default_index: 0,
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认选中的索引
    pub fn default(mut self, index: usize) -> Self {
        self.default_index = index;
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
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

impl MultiSelectFormField {
    /// 创建新的多选字段
    pub fn new(key: impl Into<String>, prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            options,
            default_selected: Vec::new(),
            result_title: None,
            condition: None,
        }
    }

    /// 设置默认选中的索引列表
    pub fn default(mut self, indices: Vec<usize>) -> Self {
        self.default_selected = indices;
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
}

/// 嵌套表单字段配置
pub struct NestedFormField {
    /// 字段键名（用于结果映射）
    pub key: String,
    /// 提示消息
    pub prompt: String,
    /// 嵌套表单
    pub nested_form: crate::interactive::form::builder::FormBuilder,
    /// 输入完成后显示的 title（可选，字段级别）
    pub result_title: Option<String>,
    /// 条件函数（可选，基于前面字段的值决定是否执行）
    pub condition: Option<Condition>,
}

impl NestedFormField {
    /// 创建新的嵌套表单字段
    pub fn new(
        key: impl Into<String>,
        prompt: impl Into<String>,
        nested_form: crate::interactive::form::builder::FormBuilder,
    ) -> Self {
        Self {
            key: key.into(),
            prompt: prompt.into(),
            nested_form,
            result_title: None,
            condition: None,
        }
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
}
