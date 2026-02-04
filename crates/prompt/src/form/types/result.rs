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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_result_new() {
        let result = FormResult::new();
        assert!(result.values.is_empty());
    }

    #[test]
    fn test_form_result_default() {
        let result = FormResult::default();
        assert!(result.values.is_empty());
    }

    #[test]
    fn test_form_result_set_and_get_string() {
        let mut result = FormResult::new();
        result.set("name".to_string(), "Alice".to_string());
        assert_eq!(result.get_string("name"), "Alice");
    }

    #[test]
    fn test_form_result_get_string_missing() {
        let result = FormResult::new();
        assert_eq!(result.get_string("missing"), "");
    }

    #[test]
    fn test_form_result_set_and_get_bool() {
        let mut result = FormResult::new();
        result.set("enabled".to_string(), true);
        assert!(result.get_bool("enabled"));

        result.set("disabled".to_string(), false);
        assert!(!result.get_bool("disabled"));
    }

    #[test]
    fn test_form_result_get_bool_missing() {
        let result = FormResult::new();
        assert!(!result.get_bool("missing"));
    }

    #[test]
    fn test_form_result_set_and_get_int() {
        let mut result = FormResult::new();
        result.set("count".to_string(), 42usize);
        assert_eq!(result.get_int("count"), 42);
    }

    #[test]
    fn test_form_result_get_int_missing() {
        let result = FormResult::new();
        assert_eq!(result.get_int("missing"), 0);
    }

    #[test]
    fn test_form_result_set_and_get_int_slice() {
        let mut result = FormResult::new();
        result.set("indices".to_string(), vec![0usize, 2, 4]);
        assert_eq!(result.get_int_slice("indices"), vec![0, 2, 4]);
    }

    #[test]
    fn test_form_result_get_int_slice_missing() {
        let result = FormResult::new();
        assert!(result.get_int_slice("missing").is_empty());
    }

    #[test]
    fn test_form_result_set_boxed() {
        let mut result = FormResult::new();
        let boxed_value: Box<dyn std::any::Any + Send + Sync> = Box::new("boxed".to_string());
        result.set_boxed("key".to_string(), boxed_value);
        assert_eq!(result.get_string("key"), "boxed");
    }

    #[test]
    fn test_form_result_get_raw() {
        let mut result = FormResult::new();
        result.set("key".to_string(), "value".to_string());

        let raw = result.get_raw("key");
        assert!(raw.is_some());
        assert!(raw.unwrap().downcast_ref::<String>().is_some());

        assert!(result.get_raw("missing").is_none());
    }

    #[test]
    fn test_form_result_nested_form() {
        let mut inner = FormResult::new();
        inner.set("inner_key".to_string(), "inner_value".to_string());

        let mut outer = FormResult::new();
        outer.set("nested".to_string(), inner);

        let retrieved = outer.get_form("nested");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get_string("inner_key"), "inner_value");
    }

    #[test]
    fn test_form_result_get_form_missing() {
        let result = FormResult::new();
        assert!(result.get_form("missing").is_none());
    }

    #[test]
    fn test_form_result_clone() {
        let mut result = FormResult::new();
        result.set("string_key".to_string(), "string_value".to_string());
        result.set("bool_key".to_string(), true);
        result.set("int_key".to_string(), 123usize);
        result.set("slice_key".to_string(), vec![1usize, 2, 3]);

        let cloned = result.clone();

        assert_eq!(cloned.get_string("string_key"), "string_value");
        assert!(cloned.get_bool("bool_key"));
        assert_eq!(cloned.get_int("int_key"), 123);
        assert_eq!(cloned.get_int_slice("slice_key"), vec![1, 2, 3]);
    }

    #[test]
    fn test_form_result_clone_with_nested() {
        let mut inner = FormResult::new();
        inner.set("inner".to_string(), "value".to_string());

        let mut outer = FormResult::new();
        outer.set("nested".to_string(), inner);

        let cloned = outer.clone();
        let nested = cloned.get_form("nested");
        assert!(nested.is_some());
        assert_eq!(nested.unwrap().get_string("inner"), "value");
    }

    #[test]
    fn test_form_result_multiple_values() {
        let mut result = FormResult::new();
        result.set("name".to_string(), "Test User".to_string());
        result.set("email".to_string(), "test@example.com".to_string());
        result.set("active".to_string(), true);
        result.set("level".to_string(), 5usize);
        result.set("roles".to_string(), vec![0usize, 1, 2]);

        assert_eq!(result.get_string("name"), "Test User");
        assert_eq!(result.get_string("email"), "test@example.com");
        assert!(result.get_bool("active"));
        assert_eq!(result.get_int("level"), 5);
        assert_eq!(result.get_int_slice("roles"), vec![0, 1, 2]);
    }

    #[test]
    fn test_form_result_overwrite() {
        let mut result = FormResult::new();
        result.set("key".to_string(), "first".to_string());
        result.set("key".to_string(), "second".to_string());
        assert_eq!(result.get_string("key"), "second");
    }
}
