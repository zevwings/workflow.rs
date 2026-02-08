//! 模板引擎封装
//!
//! 使用 handlebars 提供统一的模板渲染接口。

use std::time::{SystemTime, UNIX_EPOCH};

use handlebars::Handlebars;
use serde::Serialize;

use crate::template::TemplateError;

/// 模板引擎类型
#[derive(Debug, Clone, Copy)]
pub enum TemplateEngineType {
    /// Handlebars 模板引擎
    Handlebars,
}

/// 模板引擎封装
///
/// 提供统一的模板渲染接口。
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// 创建新的模板引擎
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        handlebars.register_escape_fn(handlebars::no_escape); // 不转义 HTML

        Self { handlebars }
    }

    /// 注册模板
    ///
    /// # 参数
    ///
    /// * `name` - 模板名称
    /// * `template` - 模板字符串
    pub fn register_template(
        &mut self,
        name: impl AsRef<str>,
        template: impl AsRef<str>,
    ) -> Result<(), TemplateError> {
        self.handlebars.register_template_string(name.as_ref(), template.as_ref())?;
        Ok(())
    }

    /// 使用变量渲染模板
    ///
    /// # 参数
    ///
    /// * `name` - 模板名称
    /// * `vars` - 模板变量（必须实现 Serialize）
    ///
    /// # 返回
    ///
    /// 渲染后的模板字符串
    pub fn render<T: Serialize>(
        &self,
        name: impl AsRef<str>,
        vars: &T,
    ) -> Result<String, TemplateError> {
        Ok(self.handlebars.render(name.as_ref(), vars)?)
    }

    /// 直接渲染模板字符串（无需注册）
    ///
    /// # 参数
    ///
    /// * `template` - 模板字符串
    /// * `vars` - 模板变量（必须实现 Serialize）
    ///
    /// # 返回
    ///
    /// 渲染后的模板字符串
    pub fn render_string<T: Serialize>(
        &self,
        template: impl AsRef<str>,
        vars: &T,
    ) -> Result<String, TemplateError> {
        // 使用临时名称注册模板
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .map_err(|_| {
                TemplateError::SystemTime("系统时间早于 Unix 纪元".to_string())
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
