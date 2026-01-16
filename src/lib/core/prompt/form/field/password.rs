//! 密码字段配置

use super::types::Condition;
use crate::core::prompt::dialog::Validator;
use std::sync::Arc;

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

    /// 设置验证器
    pub fn validator(mut self, validator: Arc<dyn Validator + Send + Sync>) -> Self {
        self.validator = Some(validator);
        self
    }

    /// 设置条件函数
    pub fn condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }
}
