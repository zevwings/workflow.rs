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

    /// 获取 Select 字段的选项值（通过索引查找）
    /// 注意：这个方法需要 options 列表，但 FormResult 没有存储 options
    /// 所以这个方法暂时不可用，Select 字段现在直接返回选项值（String）
    #[allow(dead_code)]
    pub fn get_select_value(&self, key: &str, options: &[String]) -> Option<String> {
        if let Some(value) = self.values.get(key) {
            // 如果是 String，直接返回
            if let Some(s) = value.downcast_ref::<String>() {
                return Some(s.clone());
            }
            // 如果是 usize（索引），从 options 中查找
            if let Some(i) = value.downcast_ref::<usize>() {
                return options.get(*i).cloned();
            }
        }
        None
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

    /// 获取字段值（兼容旧 API，返回 Option<String>）
    /// 注意：对于 Select 字段，返回的是选项值（通过索引查找），而不是索引本身
    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(value) = self.values.get(key) {
            // 尝试作为 String
            if let Some(s) = value.downcast_ref::<String>() {
                return Some(s.clone());
            }
            // 尝试作为 &str
            if let Some(s) = value.downcast_ref::<&str>() {
                return Some(s.to_string());
            }
            // 尝试作为 bool（转换为 "yes" 或 "no"）
            if let Some(b) = value.downcast_ref::<bool>() {
                return Some(if *b {
                    "yes".to_string()
                } else {
                    "no".to_string()
                });
            }
            // 尝试作为 usize（Select 字段的索引，需要从 options 中查找）
            // 注意：这里无法获取 options，所以返回索引的字符串形式
            // 实际使用中，应该使用 get_string() 或 get_int() 方法
            if let Some(i) = value.downcast_ref::<usize>() {
                return Some(i.to_string());
            }
        }
        None
    }

    /// 获取字段值，如果不存在返回错误（兼容旧 API）
    pub fn get_required(&self, key: &str) -> color_eyre::Result<String> {
        self.get(key)
            .ok_or_else(|| color_eyre::eyre::eyre!("Field '{}' is required", key))
    }

    /// 检查字段是否存在（兼容旧 API）
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// 获取布尔值（兼容旧 API，将 "yes" 转换为 true，"no" 转换为 false）
    /// 注意：新模块的 Confirm 字段直接返回 bool，所以这个方法会优先检查 bool 值
    pub fn get_bool_opt(&self, key: &str) -> Option<bool> {
        if let Some(value) = self.values.get(key) {
            // 优先检查 bool 值（新模块的 Confirm 字段）
            if let Some(b) = value.downcast_ref::<bool>() {
                return Some(*b);
            }
            // 检查字符串值（旧模块的 Confirmation 字段返回 "yes"/"no"）
            if let Some(s) = value.downcast_ref::<String>() {
                return Some(s == "yes");
            }
            if let Some(s) = value.downcast_ref::<&str>() {
                return Some(*s == "yes");
            }
        }
        None
    }
}
