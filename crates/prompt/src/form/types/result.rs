//! 表单执行结果核心结构

use std::collections::HashMap;

/// 表单执行结果
pub struct FormResult {
    /// 字段值映射
    pub(crate) values: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl FormResult {
    /// 创建新的表单结果
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// 设置字段值（内部使用）
    pub fn set<T: 'static + Send + Sync>(&mut self, key: String, value: T) {
        self.values.insert(key, Box::new(value));
    }

    /// 设置字段值（接受已装箱的值）
    /// 用于从 execute_field 传递 Box<dyn std::any::Any + Send + Sync>
    pub fn set_boxed(&mut self, key: String, value: Box<dyn std::any::Any + Send + Sync>) {
        self.values.insert(key, value);
    }

    /// 获取字段值（用于条件函数，内部使用）
    pub fn get_raw(&self, key: &str) -> Option<&Box<dyn std::any::Any + Send + Sync>> {
        self.values.get(key)
    }
}

impl Clone for FormResult {
    fn clone(&self) -> Self {
        let mut cloned = FormResult::new();
        for (key, value) in &self.values {
            // 尝试克隆不同类型的值
            if let Some(s) = value.downcast_ref::<String>() {
                cloned.set(key.clone(), s.clone());
            } else if let Some(b) = value.downcast_ref::<bool>() {
                cloned.set(key.clone(), *b);
            } else if let Some(i) = value.downcast_ref::<usize>() {
                cloned.set(key.clone(), *i);
            } else if let Some(v) = value.downcast_ref::<Vec<usize>>() {
                cloned.set(key.clone(), v.clone());
            } else if let Some(form) = value.downcast_ref::<FormResult>() {
                cloned.set(key.clone(), form.clone());
            }
            // 注意：其他类型无法克隆，会被跳过
        }
        cloned
    }
}

impl Default for FormResult {
    fn default() -> Self {
        Self::new()
    }
}

impl FormResult {
    /// 获取字符串值
    pub fn get_string(&self, key: &str) -> String {
        if let Some(value) = self.values.get(key) {
            if let Some(s) = value.downcast_ref::<String>() {
                return s.clone();
            }
            // 尝试转换为字符串
            if let Some(s) = value.downcast_ref::<&str>() {
                return s.to_string();
            }
        }
        String::new()
    }

    /// 获取布尔值
    pub fn get_bool(&self, key: &str) -> bool {
        if let Some(value) = self.values.get(key) {
            if let Some(b) = value.downcast_ref::<bool>() {
                return *b;
            }
        }
        false
    }

    /// 获取整数值
    pub fn get_int(&self, key: &str) -> usize {
        if let Some(value) = self.values.get(key) {
            if let Some(i) = value.downcast_ref::<usize>() {
                return *i;
            }
        }
        0
    }

    /// 获取整数切片
    pub fn get_int_slice(&self, key: &str) -> Vec<usize> {
        if let Some(value) = self.values.get(key) {
            if let Some(slice) = value.downcast_ref::<Vec<usize>>() {
                return slice.clone();
            }
        }
        Vec::new()
    }

    /// 获取嵌套表单结果
    pub fn get_form(&self, key: &str) -> Option<FormResult> {
        self.values
            .get(key)
            .and_then(|value| value.downcast_ref::<FormResult>())
            .cloned()
    }
}
