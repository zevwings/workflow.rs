//! 单选字段配置

use super::types::Condition;

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
