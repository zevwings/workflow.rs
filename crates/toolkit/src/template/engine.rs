//! Template engine wrapper
//!
//! Provides a unified interface for template rendering using handlebars.

use std::time::{SystemTime, UNIX_EPOCH};

use handlebars::Handlebars;
use serde::Serialize;

use crate::template::TemplateError;

/// Template engine type
#[derive(Debug, Clone, Copy)]
pub enum TemplateEngineType {
    /// Handlebars template engine
    Handlebars,
}

/// Template engine wrapper
///
/// Provides a unified interface for template rendering.
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        handlebars.register_escape_fn(handlebars::no_escape); // Don't escape HTML

        Self { handlebars }
    }

    /// Register a template
    ///
    /// # Arguments
    ///
    /// * `name` - Template name
    /// * `template` - Template string
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<(), TemplateError> {
        self.handlebars.register_template_string(name, template)?;
        Ok(())
    }

    /// Render a template with variables
    ///
    /// # Arguments
    ///
    /// * `name` - Template name
    /// * `vars` - Template variables (must implement Serialize)
    ///
    /// # Returns
    ///
    /// Rendered template string
    pub fn render<T: Serialize>(&self, name: &str, vars: &T) -> Result<String, TemplateError> {
        Ok(self.handlebars.render(name, vars)?)
    }

    /// Render a template string directly (without registration)
    ///
    /// # Arguments
    ///
    /// * `template` - Template string
    /// * `vars` - Template variables (must implement Serialize)
    ///
    /// # Returns
    ///
    /// Rendered template string
    pub fn render_string<T: Serialize>(
        &self,
        template: &str,
        vars: &T,
    ) -> Result<String, TemplateError> {
        // Register template with a temporary name
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .map_err(|_| {
                TemplateError::SystemTime("System time is before Unix epoch".to_string())
            })?;

        let temp_name = format!("__temp_{}", timestamp);
        let mut engine = TemplateEngine::new();
        engine.register_template(&temp_name, template)?;
        engine.render(&temp_name, vars)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_engine() {
        let engine = TemplateEngine::new();
        // 验证引擎可以创建
        assert!(std::mem::size_of_val(&engine) > 0);
    }

    #[test]
    fn test_default_engine() {
        let engine = TemplateEngine::default();
        assert!(std::mem::size_of_val(&engine) > 0);
    }

    #[test]
    fn test_register_and_render_template() {
        let mut engine = TemplateEngine::new();

        // 注册模板
        let result = engine.register_template("greeting", "Hello, {{name}}!");
        assert!(result.is_ok());

        // 渲染模板
        let vars = json!({"name": "World"});
        let rendered = engine.render("greeting", &vars);
        assert!(rendered.is_ok());
        assert_eq!(rendered.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_render_with_multiple_variables() {
        let mut engine = TemplateEngine::new();

        engine
            .register_template("message", "{{greeting}}, {{name}}! Today is {{day}}.")
            .unwrap();

        let vars = json!({
            "greeting": "Hi",
            "name": "Alice",
            "day": "Monday"
        });

        let rendered = engine.render("message", &vars).unwrap();
        assert_eq!(rendered, "Hi, Alice! Today is Monday.");
    }

    #[test]
    fn test_render_with_nested_variables() {
        let mut engine = TemplateEngine::new();

        engine.register_template("user", "{{user.name}} - {{user.email}}").unwrap();

        let vars = json!({
            "user": {
                "name": "Bob",
                "email": "bob@example.com"
            }
        });

        let rendered = engine.render("user", &vars).unwrap();
        assert_eq!(rendered, "Bob - bob@example.com");
    }

    #[test]
    fn test_render_missing_variable_not_strict() {
        let mut engine = TemplateEngine::new();

        // 非严格模式下，缺少的变量应该被忽略（返回空字符串）
        engine.register_template("test", "Hello, {{name}}!").unwrap();

        let vars = json!({});
        let rendered = engine.render("test", &vars);
        assert!(rendered.is_ok());
        assert_eq!(rendered.unwrap(), "Hello, !");
    }

    #[test]
    fn test_render_string_direct() {
        let engine = TemplateEngine::new();

        let vars = json!({"value": 42});
        let rendered = engine.render_string("The answer is {{value}}", &vars);
        assert!(rendered.is_ok());
        assert_eq!(rendered.unwrap(), "The answer is 42");
    }

    #[test]
    fn test_render_string_with_complex_template() {
        let engine = TemplateEngine::new();

        let template = r#"{{#each items}}
- {{this}}
{{/each}}"#;

        let vars = json!({
            "items": ["apple", "banana", "cherry"]
        });

        let rendered = engine.render_string(template, &vars).unwrap();
        assert!(rendered.contains("- apple"));
        assert!(rendered.contains("- banana"));
        assert!(rendered.contains("- cherry"));
    }

    #[test]
    fn test_render_invalid_template_syntax() {
        let mut engine = TemplateEngine::new();

        // 无效的模板语法：未闭合的 block
        let result = engine.register_template("bad", "{{#if active}}not closed");
        assert!(result.is_err());
    }

    #[test]
    fn test_render_nonexistent_template() {
        let engine = TemplateEngine::new();

        let vars = json!({});
        let result = engine.render("nonexistent", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_preserves_html() {
        let mut engine = TemplateEngine::new();

        // 验证 HTML 不被转义（no_escape 设置）
        engine.register_template("html", "{{content}}").unwrap();

        let vars = json!({"content": "<b>bold</b>"});
        let rendered = engine.render("html", &vars).unwrap();
        assert_eq!(rendered, "<b>bold</b>");
    }

    #[test]
    fn test_render_with_conditionals() {
        let mut engine = TemplateEngine::new();

        engine
            .register_template("conditional", "{{#if active}}Active{{else}}Inactive{{/if}}")
            .unwrap();

        let vars_true = json!({"active": true});
        assert_eq!(engine.render("conditional", &vars_true).unwrap(), "Active");

        let vars_false = json!({"active": false});
        assert_eq!(
            engine.render("conditional", &vars_false).unwrap(),
            "Inactive"
        );
    }

    #[test]
    fn test_template_engine_type() {
        // 验证枚举类型可以正常使用
        let engine_type = TemplateEngineType::Handlebars;
        assert!(matches!(engine_type, TemplateEngineType::Handlebars));
    }
}
