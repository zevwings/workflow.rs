//! 表单构建器核心

use crate::backend::Backend;
use crate::form::field::FormField;
use crate::form::types::FormGroup;
use crate::form::FormExecutor;

/// 表单构建器（链式 API）
pub struct FormBuilder {
    /// 字段列表（用于简单模式，不使用 Group）
    pub(crate) fields: Vec<FormField>,
    /// 组列表（用于 Group/Step 模式）
    pub(crate) groups: Vec<FormGroup>,
    /// 表单标题
    pub(crate) title: Option<String>,
}

impl FormBuilder {
    /// 创建新的表单构建器
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            groups: Vec::new(),
            title: None,
        }
    }

    /// 设置表单标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 获取字段列表（内部使用，用于简单模式）
    pub(crate) fn get_fields(&self) -> &[FormField] {
        &self.fields
    }

    /// 获取组列表（内部使用，用于 Group/Step 模式）
    pub(crate) fn get_groups(&self) -> &[FormGroup] {
        &self.groups
    }

    /// 检查是否使用 Group 模式
    pub(crate) fn has_groups(&self) -> bool {
        !self.groups.is_empty()
    }

    /// 获取表单标题
    pub fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// 执行表单并收集用户输入（使用默认终端后端）
    ///
    /// 内部使用 `FormExecutor` 来执行表单。
    pub fn run(self) -> crate::Result<crate::form::FormResult> {
        FormExecutor::new().execute(&self)
    }

    /// 使用指定后端执行表单并收集用户输入（内部使用，仅测试时调用）
    #[allow(dead_code)]
    pub(crate) fn run_with_backend<B: Backend>(
        self,
        backend: &mut B,
    ) -> crate::Result<crate::form::FormResult> {
        FormExecutor::new().execute_with_backend(&self, backend)
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}
