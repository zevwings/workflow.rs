//! HTTP Header 工具

use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// 可转换为 HeaderMap 的类型
pub trait IntoHeaderMap {
    /// 转换为 HeaderMap
    fn into_header_map(self) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>>;
}

impl IntoHeaderMap for HeaderMap {
    fn into_header_map(self) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self)
    }
}

impl IntoHeaderMap for HashMap<String, String> {
    fn into_header_map(self) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();
        for (key, value) in self {
            let name = HeaderName::try_from(key)?;
            let value = HeaderValue::from_str(&value)?;
            headers.insert(name, value);
        }
        Ok(headers)
    }
}

impl IntoHeaderMap for Vec<(String, String)> {
    fn into_header_map(self) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();
        for (key, value) in self {
            let name = HeaderName::try_from(key)?;
            let value = HeaderValue::from_str(&value)?;
            headers.insert(name, value);
        }
        Ok(headers)
    }
}

impl<const N: usize> IntoHeaderMap for [(&str, &str); N] {
    fn into_header_map(self) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();
        for (key, value) in self {
            let name = HeaderName::try_from(key)?;
            let value = HeaderValue::from_str(value)?;
            headers.insert(name, value);
        }
        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_map_into_header_map() {
        let mut map = HeaderMap::new();
        map.insert("content-type", HeaderValue::from_static("application/json"));
        let result = map.into_header_map().unwrap();
        assert!(result.contains_key("content-type"));
    }

    #[test]
    fn test_hashmap_into_header_map() {
        let mut map = HashMap::new();
        map.insert("Content-Type".to_string(), "application/json".to_string());
        let result = map.into_header_map().unwrap();
        assert!(result.contains_key("content-type"));
    }

    #[test]
    fn test_vec_into_header_map() {
        let vec = vec![("Content-Type".to_string(), "application/json".to_string())];
        let result = vec.into_header_map().unwrap();
        assert!(result.contains_key("content-type"));
    }

    #[test]
    fn test_array_into_header_map() {
        let arr = [("Content-Type", "application/json")];
        let result = arr.into_header_map().unwrap();
        assert!(result.contains_key("content-type"));
    }
}
