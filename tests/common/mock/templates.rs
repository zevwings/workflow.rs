//! Mock 响应模板系统
//!
//! 提供响应模板解析和变量替换功能，支持动态响应生成。

use color_eyre::Result;
use std::collections::HashMap;

/// 响应生成器 trait
///
/// 定义响应生成器的接口，支持基于请求参数动态生成响应。
pub trait ResponseGenerator: Send + Sync {
    /// 生成响应
    ///
    /// # 参数
    ///
    /// * `request` - Mock请求信息（包含方法、路径、参数等）
    ///
    /// # 返回
    ///
    /// 生成的响应体字符串
    fn generate(&self, request: &MockRequest) -> Result<String>;
}

/// Mock 请求信息
///
/// 包含生成响应所需的所有请求信息。
#[derive(Debug, Clone)]
pub struct MockRequest {
    /// HTTP 方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 路径参数（从路径中提取的变量）
    pub path_params: HashMap<String, String>,
    /// 查询参数
    pub query_params: HashMap<String, String>,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Option<String>,
}

impl MockRequest {
    /// 创建新的 Mock 请求
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// 添加路径参数
    #[allow(dead_code)]
    pub fn with_path_param(mut self, key: String, value: String) -> Self {
        self.path_params.insert(key, value);
        self
    }

    /// 添加查询参数
    #[allow(dead_code)]
    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.insert(key, value);
        self
    }

    /// 添加请求头
    #[allow(dead_code)]
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// 设置请求体
    #[allow(dead_code)]
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
}

/// 模板响应生成器
///
/// 基于模板字符串和变量替换生成响应。
pub struct TemplateResponseGenerator {
    template: String,
    variables: HashMap<String, String>,
}

impl TemplateResponseGenerator {
    /// 创建新的模板响应生成器
    ///
    /// # 参数
    ///
    /// * `template` - 响应模板字符串，支持 `{{variable}}` 格式的变量占位符
    /// * `variables` - 变量映射表
    pub fn new(template: String, variables: HashMap<String, String>) -> Self {
        Self {
            template,
            variables,
        }
    }

    /// 替换模板中的变量
    fn replace_variables(&self, request: &MockRequest) -> String {
        let mut result = self.template.clone();

        // 替换预定义的变量
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 替换路径参数
        for (key, value) in &request.path_params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 替换查询参数
        for (key, value) in &request.query_params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

impl ResponseGenerator for TemplateResponseGenerator {
    fn generate(&self, request: &MockRequest) -> Result<String> {
        Ok(self.replace_variables(request))
    }
}

/// 条件响应生成器
///
/// 基于请求参数选择不同的响应生成器。
pub struct ConditionalResponseGenerator {
    conditions: Vec<(Box<dyn RequestMatcher>, Box<dyn ResponseGenerator>)>,
    default: Box<dyn ResponseGenerator>,
}

impl ConditionalResponseGenerator {
    /// 创建新的条件响应生成器
    ///
    /// # 参数
    ///
    /// * `default` - 默认响应生成器（当没有条件匹配时使用）
    pub fn new(default: Box<dyn ResponseGenerator>) -> Self {
        Self {
            conditions: Vec::new(),
            default,
        }
    }

    /// 添加条件分支
    ///
    /// # 参数
    ///
    /// * `matcher` - 请求匹配器
    /// * `generator` - 对应的响应生成器
    pub fn add_condition(
        mut self,
        matcher: Box<dyn RequestMatcher>,
        generator: Box<dyn ResponseGenerator>,
    ) -> Self {
        self.conditions.push((matcher, generator));
        self
    }
}

impl ResponseGenerator for ConditionalResponseGenerator {
    fn generate(&self, request: &MockRequest) -> Result<String> {
        // 按顺序检查条件
        for (matcher, generator) in &self.conditions {
            if matcher.matches(request) {
                return generator.generate(request);
            }
        }

        // 使用默认生成器
        self.default.generate(request)
    }
}

/// 请求匹配器 trait
///
/// 定义请求匹配逻辑，用于条件响应生成。
pub trait RequestMatcher: Send + Sync {
    /// 检查请求是否匹配
    fn matches(&self, request: &MockRequest) -> bool;
}

/// 路径匹配器
///
/// 基于路径模式匹配请求。
pub struct PathMatcher {
    pattern: String,
}

impl PathMatcher {
    /// 创建新的路径匹配器
    ///
    /// # 参数
    ///
    /// * `pattern` - 路径模式，支持通配符 `*` 和变量 `{name}`
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }
}

impl RequestMatcher for PathMatcher {
    fn matches(&self, request: &MockRequest) -> bool {
        // 简单的路径匹配实现
        // 可以扩展支持更复杂的模式匹配
        self.pattern == request.path || self.pattern == "*"
    }
}

/// 方法匹配器
///
/// 基于 HTTP 方法匹配请求。
pub struct MethodMatcher {
    method: String,
}

impl MethodMatcher {
    /// 创建新的方法匹配器
    pub fn new(method: String) -> Self {
        Self {
            method: method.to_uppercase(),
        }
    }
}

impl RequestMatcher for MethodMatcher {
    fn matches(&self, request: &MockRequest) -> bool {
        self.method == request.method.to_uppercase()
    }
}

/// 组合匹配器
///
/// 组合多个匹配器，支持 AND 和 OR 逻辑。
pub struct CompositeMatcher {
    matchers: Vec<Box<dyn RequestMatcher>>,
    logic: MatchLogic,
}

#[derive(Debug, Clone)]
pub enum MatchLogic {
    And, // 所有匹配器都必须匹配
    Or,  // 任一匹配器匹配即可
}

impl CompositeMatcher {
    /// 创建新的组合匹配器
    ///
    /// # 参数
    ///
    /// * `logic` - 匹配逻辑（AND 或 OR）
    pub fn new(logic: MatchLogic) -> Self {
        Self {
            matchers: Vec::new(),
            logic,
        }
    }

    /// 添加匹配器
    pub fn add_matcher(mut self, matcher: Box<dyn RequestMatcher>) -> Self {
        self.matchers.push(matcher);
        self
    }
}

impl RequestMatcher for CompositeMatcher {
    fn matches(&self, request: &MockRequest) -> bool {
        match self.logic {
            MatchLogic::And => self.matchers.iter().all(|m| m.matches(request)),
            MatchLogic::Or => self.matchers.iter().any(|m| m.matches(request)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_response_generator() -> Result<()> {
        let mut vars = HashMap::new();
        vars.insert("pr_number".to_string(), "123".to_string());
        vars.insert("owner".to_string(), "test-owner".to_string());

        let generator = TemplateResponseGenerator::new(
            r#"{"number": {{pr_number}}, "owner": "{{owner}}"}"#.to_string(),
            vars,
        );

        let request = MockRequest::new("GET".to_string(), "/test".to_string());
        let response = generator.generate(&request)?;

        assert!(response.contains("\"number\": 123"));
        assert!(response.contains("\"owner\": \"test-owner\""));
        Ok(())
    }

    #[test]
    fn test_conditional_response_generator() -> Result<()> {
        let default_vars = HashMap::new();
        let default_generator: Box<dyn ResponseGenerator> = Box::new(
            TemplateResponseGenerator::new(r#"{"default": true}"#.to_string(), default_vars),
        );

        let mut success_vars = HashMap::new();
        success_vars.insert("status".to_string(), "success".to_string());
        let success_generator: Box<dyn ResponseGenerator> = Box::new(
            TemplateResponseGenerator::new(r#"{"status": "{{status}}"}"#.to_string(), success_vars),
        );

        let matcher: Box<dyn RequestMatcher> = Box::new(MethodMatcher::new("POST".to_string()));

        let conditional = ConditionalResponseGenerator::new(default_generator)
            .add_condition(matcher, success_generator);

        // POST 请求应该匹配成功生成器
        let post_request = MockRequest::new("POST".to_string(), "/test".to_string());
        let response = conditional.generate(&post_request)?;
        assert!(response.contains("success"));

        // GET 请求应该使用默认生成器
        let get_request = MockRequest::new("GET".to_string(), "/test".to_string());
        let response = conditional.generate(&get_request)?;
        assert!(response.contains("default"));

        Ok(())
    }

    #[test]
    fn test_path_matcher() {
        let matcher = PathMatcher::new("/api/test".to_string());
        let request = MockRequest::new("GET".to_string(), "/api/test".to_string());
        assert!(matcher.matches(&request));

        let request = MockRequest::new("GET".to_string(), "/api/other".to_string());
        assert!(!matcher.matches(&request));
    }

    #[test]
    fn test_method_matcher() {
        let matcher = MethodMatcher::new("POST".to_string());
        let request = MockRequest::new("POST".to_string(), "/test".to_string());
        assert!(matcher.matches(&request));

        let request = MockRequest::new("GET".to_string(), "/test".to_string());
        assert!(!matcher.matches(&request));
    }

    #[test]
    fn test_composite_matcher() {
        let path_matcher: Box<dyn RequestMatcher> =
            Box::new(PathMatcher::new("/api/test".to_string()));
        let method_matcher: Box<dyn RequestMatcher> =
            Box::new(MethodMatcher::new("POST".to_string()));

        // AND 逻辑：两个条件都必须匹配
        let and_matcher = CompositeMatcher::new(MatchLogic::And)
            .add_matcher(Box::new(PathMatcher::new("/api/test".to_string())))
            .add_matcher(Box::new(MethodMatcher::new("POST".to_string())));

        let request = MockRequest::new("POST".to_string(), "/api/test".to_string());
        assert!(and_matcher.matches(&request));

        let request = MockRequest::new("GET".to_string(), "/api/test".to_string());
        assert!(!and_matcher.matches(&request));

        // OR 逻辑：任一条件匹配即可
        let or_matcher = CompositeMatcher::new(MatchLogic::Or)
            .add_matcher(path_matcher)
            .add_matcher(method_matcher);

        let request = MockRequest::new("POST".to_string(), "/api/other".to_string());
        assert!(or_matcher.matches(&request));
    }
}
