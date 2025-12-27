//! Mock 请求验证器
//!
//! 提供请求验证功能，包括请求体、请求头和请求参数的验证。

use crate::common::mock::templates::MockRequest;
use color_eyre::Result;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// 请求验证器
///
/// 用于验证 Mock 请求的各种组成部分。
pub struct RequestValidator {
    body_validator: Option<Box<dyn BodyValidator>>,
    header_validators: Vec<HeaderValidator>,
    param_validators: Vec<ParamValidator>,
}

impl RequestValidator {
    /// 创建新的请求验证器
    pub fn new() -> Self {
        Self {
            body_validator: None,
            header_validators: Vec::new(),
            param_validators: Vec::new(),
        }
    }

    /// 添加请求体验证
    pub fn require_body_json(mut self, pattern: &str) -> Self {
        self.body_validator = Some(Box::new(JsonBodyValidator::new(pattern.to_string())));
        self
    }

    /// 添加请求头验证
    pub fn require_header(mut self, name: &str, pattern: &str) -> Self {
        self.header_validators.push(HeaderValidator {
            name: name.to_string(),
            pattern: pattern.to_string(),
        });
        self
    }

    /// 添加查询参数验证
    pub fn require_query_param(mut self, name: &str, value: &str) -> Self {
        self.param_validators.push(ParamValidator {
            name: name.to_string(),
            value: value.to_string(),
            param_type: ParamType::Query,
        });
        self
    }

    /// 添加路径参数验证
    #[allow(dead_code)]
    pub fn require_path_param(mut self, name: &str, value: &str) -> Self {
        self.param_validators.push(ParamValidator {
            name: name.to_string(),
            value: value.to_string(),
            param_type: ParamType::Path,
        });
        self
    }

    /// 验证请求
    pub fn validate(&self, request: &MockRequest) -> ValidationResult {
        let mut errors = Vec::new();

        // 验证请求体
        if let Some(ref body_validator) = self.body_validator {
            if let Some(ref body) = request.body {
                if let Err(e) = body_validator.validate(body) {
                    errors.push(format!("请求体验证失败: {}", e));
                }
            } else {
                errors.push("请求体缺失".to_string());
            }
        }

        // 验证请求头
        for validator in &self.header_validators {
            if let Some(value) = request.headers.get(&validator.name) {
                if !validator.matches(value) {
                    errors.push(format!(
                        "请求头 '{}' 验证失败: 期望匹配 '{}', 实际值 '{}'",
                        validator.name, validator.pattern, value
                    ));
                }
            } else {
                errors.push(format!("必需的请求头 '{}' 缺失", validator.name));
            }
        }

        // 验证查询参数
        for validator in &self.param_validators {
            if validator.param_type == ParamType::Query {
                if let Some(value) = request.query_params.get(&validator.name) {
                    if value != &validator.value {
                        errors.push(format!(
                            "查询参数 '{}' 验证失败: 期望 '{}', 实际 '{}'",
                            validator.name, validator.value, value
                        ));
                    }
                } else {
                    errors.push(format!("必需的查询参数 '{}' 缺失", validator.name));
                }
            }
        }

        // 验证路径参数
        for validator in &self.param_validators {
            if validator.param_type == ParamType::Path {
                if let Some(value) = request.path_params.get(&validator.name) {
                    if value != &validator.value {
                        errors.push(format!(
                            "路径参数 '{}' 验证失败: 期望 '{}', 实际 '{}'",
                            validator.name, validator.value, value
                        ));
                    }
                } else {
                    errors.push(format!("必需的路径参数 '{}' 缺失", validator.name));
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }
}

impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求体验证器 trait
pub trait BodyValidator: Send + Sync {
    fn validate(&self, body: &str) -> Result<()>;
}

/// JSON 请求体验证器
///
/// 使用正则表达式验证 JSON 请求体的内容。
pub struct JsonBodyValidator {
    pattern: String,
    regex: Regex,
}

impl JsonBodyValidator {
    pub fn new(pattern: String) -> Self {
        // 将简单的模式转换为正则表达式
        // 例如: {"title": ".+"} -> \{"title"\s*:\s*".+"\}
        let pattern_clone = pattern.clone();
        let regex_pattern =
            pattern_clone.replace("{", r"\{").replace("}", r"\}").replace('"', r#"\""#);

        Self {
            pattern,
            regex: Regex::new(&regex_pattern).unwrap_or_else(|_| {
                // 如果正则表达式编译失败，使用简单的字符串匹配
                Regex::new(&regex::escape(&pattern_clone)).unwrap()
            }),
        }
    }
}

impl BodyValidator for JsonBodyValidator {
    fn validate(&self, body: &str) -> Result<()> {
        use color_eyre::eyre::Context;

        // 首先验证 JSON 格式
        let _: Value = serde_json::from_str(body).context("请求体不是有效的 JSON")?;

        // 然后验证内容匹配
        if self.regex.is_match(body) {
            Ok(())
        } else {
            Err(color_eyre::eyre::eyre!(
                "请求体内容不匹配模式: {}",
                self.pattern
            ))
        }
    }
}

/// 请求头验证器
#[derive(Debug, Clone)]
pub struct HeaderValidator {
    name: String,
    pattern: String,
}

impl HeaderValidator {
    fn matches(&self, value: &str) -> bool {
        // 简单的字符串匹配或正则表达式匹配
        if self.pattern.starts_with('^') || self.pattern.contains('.') {
            // 尝试作为正则表达式匹配
            Regex::new(&self.pattern)
                .map(|re| re.is_match(value))
                .unwrap_or_else(|_| value == self.pattern)
        } else {
            value == self.pattern
        }
    }
}

/// 参数验证器
#[derive(Debug, Clone)]
pub struct ParamValidator {
    name: String,
    value: String,
    param_type: ParamType,
}

#[derive(Debug, Clone, PartialEq)]
enum ParamType {
    Query,
    Path,
}

// MockRequest 从 mock_templates 模块导入

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否验证通过
    pub is_valid: bool,
    /// 错误信息列表
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// 获取验证是否通过
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// 获取错误信息
    #[allow(dead_code)]
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// 获取格式化的错误报告
    #[allow(dead_code)]
    pub fn to_report(&self) -> String {
        if self.is_valid {
            "验证通过".to_string()
        } else {
            format!(
                "验证失败 ({} 个错误):\n{}",
                self.errors.len(),
                self.errors.join("\n")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_validator_header() {
        let validator = RequestValidator::new().require_header("authorization", "token .+");

        let mut request = MockRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        // 缺少必需的请求头
        let result = validator.validate(&request);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("authorization")));

        // 添加请求头
        request.headers.insert("authorization".to_string(), "token abc123".to_string());
        let result = validator.validate(&request);
        assert!(result.is_valid);
    }

    #[test]
    fn test_request_validator_query_param() {
        let validator = RequestValidator::new().require_query_param("draft", "false");

        let mut request = MockRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        // 缺少必需的查询参数
        let result = validator.validate(&request);
        assert!(!result.is_valid);

        // 添加查询参数
        request.query_params.insert("draft".to_string(), "false".to_string());
        let result = validator.validate(&request);
        assert!(result.is_valid);
    }

    #[test]
    fn test_request_validator_body() {
        let validator = RequestValidator::new().require_body_json(r#"{"title": ".+"}"#);

        let mut request = MockRequest {
            method: "POST".to_string(),
            path: "/test".to_string(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        // 缺少请求体
        let result = validator.validate(&request);
        assert!(!result.is_valid);

        // 添加有效的请求体
        request.body = Some(r#"{"title": "Test PR"}"#.to_string());
        let result = validator.validate(&request);
        assert!(result.is_valid);
    }
}
