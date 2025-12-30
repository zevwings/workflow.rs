//! Template engine wrapper
//!
//! Provides a unified interface for template rendering using handlebars.

use crate::base::format::date::get_unix_timestamp_nanos;
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use handlebars::Handlebars;
use serde::Serialize;

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
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.handlebars
            .register_template_string(name, template)
            .wrap_err_with(|| format!("Failed to register template: {}", name))?;
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
    pub fn render<T: Serialize>(&self, name: &str, vars: &T) -> Result<String> {
        self.handlebars
            .render(name, vars)
            .map_err(|e| eyre!("Failed to render template '{}': {}", name, e))
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
    pub fn render_string<T: Serialize>(&self, template: &str, vars: &T) -> Result<String> {
        // Register template with a temporary name
        let timestamp = get_unix_timestamp_nanos();
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
    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::json;
    use std::collections::HashMap;

    // ==================== Helper Functions ====================

    /// 创建测试用的变量 HashMap
    fn create_test_variables() -> HashMap<String, serde_json::Value> {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), json!("John Doe"));
        vars.insert("age".to_string(), json!(30));
        vars.insert("active".to_string(), json!(true));
        vars.insert("items".to_string(), json!(["item1", "item2", "item3"]));
        vars
    }

    /// 创建复杂的嵌套变量
    fn create_nested_variables() -> serde_json::Value {
        json!({
            "user": {
                "name": "Alice",
                "profile": {
                    "email": "alice@example.com",
                    "role": "admin"
                }
            },
            "project": {
                "name": "workflow-rs",
                "version": "1.0.0",
                "features": ["git", "jira", "pr"]
            }
        })
    }

    // ==================== Test Cases ====================

    /// 测试引擎创建
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine::new() 能够成功创建模板引擎实例，并能够正常渲染模板。
    ///
    /// ## 测试场景
    /// 1. 调用 new() 方法创建模板引擎
    /// 2. 使用 render_string() 方法渲染简单模板
    /// 3. 验证渲染结果正确
    ///
    /// ## 预期结果
    /// - 引擎创建成功
    /// - 模板渲染成功，返回正确的结果
    /// - 变量替换正确（"Hello {{name}}" + {"name": "World"} -> "Hello World"）
    #[test]
    fn test_new_engine_creation_return_ok() -> Result<()> {
        let engine = TemplateEngine::new();

        // Assert: 验证引擎创建成功（通过尝试渲染一个简单模板）
        let result = engine.render_string("Hello {{name}}", &json!({"name": "World"}));
        assert!(result.is_ok());
        assert_eq!(result?, "Hello World");
        Ok(())
    }

    /// 测试默认实现
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine::default() 能够创建模板引擎实例，行为与 new() 方法一致。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 方法创建模板引擎
    /// 2. 使用 render_string() 方法渲染模板
    /// 3. 验证渲染结果正确
    ///
    /// ## 预期结果
    /// - 默认引擎创建成功
    /// - 模板渲染成功，返回正确的结果
    /// - default() 和 new() 的行为一致
    #[test]
    fn test_default_engine_creation_return_ok() -> Result<()> {
        let engine = TemplateEngine::default();

        // Assert: 验证默认引擎与 new() 创建的引擎行为一致
        let result = engine.render_string("Test {{value}}", &json!({"value": "default"}));
        assert!(result.is_ok());
        assert_eq!(result?, "Test default");
        Ok(())
    }

    /// 测试简单模板渲染
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine 能够注册模板并正确渲染包含变量的简单模板。
    ///
    /// ## 测试场景
    /// 1. 创建模板引擎实例
    /// 2. 使用 register_template() 注册模板
    /// 3. 使用 render() 方法渲染模板
    /// 4. 验证渲染结果正确
    ///
    /// ## 预期结果
    /// - 模板注册成功
    /// - 模板渲染成功，变量正确替换
    /// - 渲染结果与预期一致
    #[test]
    fn test_render_simple_template_return_ok() -> Result<()> {
        let mut engine = TemplateEngine::new();

        // 注册简单模板
        let template = "Hello {{name}}!";
        let result = engine.register_template("greeting", template);
        assert!(result.is_ok());

        // 渲染模板
        let vars = json!({"name": "Alice"});
        let rendered = engine.render("greeting", &vars);
        assert!(rendered.is_ok());
        assert_eq!(rendered?, "Hello Alice!");
        Ok(())
    }

    /// 测试变量渲染（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 TemplateEngine 能够正确渲染包含变量的模板。
    ///
    /// ## 测试场景
    /// 1. 测试基本变量替换（name, age, active）
    /// 2. 测试嵌套对象访问（user.name, user.profile.email）
    ///
    /// ## 预期结果
    /// - 所有变量都能正确渲染并替换为实际值
    #[rstest]
    #[case(
        "Name: {{name}}, Age: {{age}}, Active: {{active}}",
        "basic",
        "Name: John Doe, Age: 30, Active: true"
    )]
    #[case(
        "User: {{user.name}}, Email: {{user.profile.email}}, Role: {{user.profile.role}}",
        "nested",
        "User: Alice, Email: alice@example.com, Role: admin"
    )]
    fn test_render_with_variables_return_ok(
        #[case] template: &str,
        #[case] vars_type: &str,
        #[case] expected: &str,
    ) -> Result<()> {
        let engine = TemplateEngine::new();

        // Arrange: 准备测试变量替换
        let vars: serde_json::Value = match vars_type {
            "basic" => {
                let mut map = serde_json::Map::new();
                for (k, v) in create_test_variables() {
                    map.insert(k, v);
                }
                serde_json::Value::Object(map)
            }
            "nested" => create_nested_variables(),
            _ => json!({}),
        };

        // Act: 渲染模板
        let result = engine.render_string(template, &vars);

        // Assert: 验证渲染结果
        assert!(result.is_ok());
        assert_eq!(result?, expected);
        Ok(())
    }

    /// 测试条件渲染（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 TemplateEngine 能够正确处理条件渲染（if/unless）。
    ///
    /// ## 测试场景
    /// 测试 if 条件为 true/false 和 unless 条件
    ///
    /// ## 预期结果
    /// - 所有条件都能正确渲染
    #[rstest]
    #[case(
        "{{#if active}}User is active{{else}}User is inactive{{/if}}",
        json!({"active": true}),
        "User is active"
    )]
    #[case(
        "{{#if active}}User is active{{else}}User is inactive{{/if}}",
        json!({"active": false}),
        "User is inactive"
    )]
    #[case(
        "{{#unless disabled}}Feature enabled{{else}}Feature disabled{{/unless}}",
        json!({"disabled": false}),
        "Feature enabled"
    )]
    /// 测试条件渲染（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 TemplateEngine 能够正确处理条件渲染（if/unless）。
    ///
    /// ## 测试场景
    /// 1. 测试 if 条件为 true 时渲染 then 分支
    /// 2. 测试 if 条件为 false 时渲染 else 分支
    /// 3. 测试 unless 条件（反向 if）
    ///
    /// ## 预期结果
    /// - 所有条件都能正确评估并渲染相应的分支
    fn test_render_with_conditions_return_ok(
        #[case] template: &str,
        #[case] vars: serde_json::Value,
        #[case] expected: &str,
    ) -> Result<()> {
        let engine = TemplateEngine::new();

        // Arrange: 准备测试条件渲染（通过参数提供）

        // Act: 渲染模板
        let result = engine.render_string(template, &vars);

        // Assert: 验证渲染结果
        assert!(result.is_ok());
        assert_eq!(result?, expected);
        Ok(())
    }

    /// 测试循环渲染（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 TemplateEngine 能够正确处理循环渲染（each）。
    ///
    /// ## 测试场景
    /// 测试简单数组循环和对象数组循环
    ///
    /// ## 预期结果
    /// - 所有循环都能正确渲染
    #[rstest]
    #[case(
        "Items: {{#each items}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}",
        json!({"items": ["apple", "banana", "cherry"]}),
        "Items: apple, banana, cherry"
    )]
    #[case(
        "{{#each users}}{{@index}}: {{name}} ({{role}}){{#unless @last}}\n{{/unless}}{{/each}}",
        json!({
            "users": [
                {"name": "Alice", "role": "admin"},
                {"name": "Bob", "role": "user"}
            ]
        }),
        "0: Alice (admin)\n1: Bob (user)"
    )]
    fn test_render_with_loops_return_ok(
        #[case] template: &str,
        #[case] vars: serde_json::Value,
        #[case] expected: &str,
    ) -> Result<()> {
        let engine = TemplateEngine::new();

        // Arrange: 准备测试循环渲染（通过参数提供）

        // Act: 渲染模板
        let result = engine.render_string(template, &vars);

        // Assert: 验证渲染结果
        assert!(result.is_ok());
        assert_eq!(result?, expected);
        Ok(())
    }

    /// 测试模板注册
    #[test]
    fn test_register_template_return_ok() -> Result<()> {
        let mut engine = TemplateEngine::new();

        // Arrange: 准备测试成功注册
        let result1 = engine.register_template("template1", "Hello {{name}}");
        assert!(result1.is_ok());

        let result2 = engine.register_template("template2", "Goodbye {{name}}");
        assert!(result2.is_ok());

        // Assert: 验证两个模板都能正常渲染
        let vars = json!({"name": "World"});

        let render1 = engine.render("template1", &vars);
        assert!(render1.is_ok());
        assert_eq!(render1?, "Hello World");

        let render2 = engine.render("template2", &vars);
        assert!(render2.is_ok());
        assert_eq!(render2?, "Goodbye World");

        // Arrange: 准备测试覆盖已存在的模板
        let result3 = engine.register_template("template1", "Hi {{name}}!");
        assert!(result3.is_ok());

        let render3 = engine.render("template1", &vars);
        assert!(render3.is_ok());
        assert_eq!(render3?, "Hi World!");
        Ok(())
    }

    /// 测试无效模板错误处理
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine 能够正确处理无效模板和不存在模板的错误情况。
    ///
    /// ## 测试场景
    /// 1. 创建模板引擎实例
    /// 2. 尝试注册语法错误的模板（缺少闭合括号）
    /// 3. 验证注册失败并返回错误
    /// 4. 尝试渲染不存在的模板
    /// 5. 验证渲染失败并返回包含有用信息的错误
    ///
    /// ## 预期结果
    /// - 语法错误的模板注册失败（返回错误）
    /// - 不存在的模板渲染失败（返回错误）
    /// - 错误信息包含有用的信息（模板名称等）
    #[test]
    fn test_render_invalid_template_return_ok() -> Result<()> {
        let mut engine = TemplateEngine::new();

        // Arrange: 准备测试语法错误的模板
        let invalid_template = "Hello {{name"; // 缺少闭合括号
        let result = engine.register_template("invalid", invalid_template);
        assert!(result.is_err());

        // Arrange: 准备测试渲染不存在的模板
        let vars = json!({"name": "World"});
        let render_result = engine.render("nonexistent", &vars);
        assert!(render_result.is_err());

        // Assert: 验证错误信息包含有用信息
        let error_msg = render_result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to render template"));
        assert!(error_msg.contains("nonexistent"));
        Ok(())
    }

    /// 测试缺失变量处理
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine 在非严格模式下能够正确处理缺失的变量，将其渲染为空字符串。
    ///
    /// ## 测试场景
    /// 1. 创建模板引擎实例
    /// 2. 使用包含变量的模板，但只提供部分变量
    /// 3. 验证缺失的变量被渲染为空字符串
    /// 4. 使用完全没有变量的情况测试
    ///
    /// ## 预期结果
    /// - 缺失的变量被渲染为空字符串
    /// - 模板渲染成功（不产生错误）
    /// - 提供的变量正确替换，缺失的变量为空
    #[test]
    fn test_render_missing_variables_return_ok() -> Result<()> {
        let engine = TemplateEngine::new();

        // Arrange: 准备测试缺失变量（非严格模式下应该渲染为空字符串）
        let template = "Hello {{name}}, your age is {{age}}";
        let incomplete_vars = json!({"name": "Alice"}); // 缺少 age 变量

        let result = engine.render_string(template, &incomplete_vars);
        assert!(result.is_ok());
        assert_eq!(result?, "Hello Alice, your age is ");

        // Arrange: 准备测试完全没有变量的情况
        let empty_vars = json!({});
        let empty_result = engine.render_string(template, &empty_vars);
        assert!(empty_result.is_ok());
        assert_eq!(empty_result?, "Hello , your age is ");
        Ok(())
    }

    /// 测试 TemplateEngineType 枚举
    ///
    /// ## 测试目的
    /// 验证 TemplateEngineType 枚举实现了 Debug、Clone 和 Copy trait。
    ///
    /// ## 测试场景
    /// 1. 创建 TemplateEngineType::Handlebars 实例
    /// 2. 测试 Debug trait 实现（格式化输出）
    /// 3. 测试 Clone trait 实现
    /// 4. 测试 Copy trait 实现
    ///
    /// ## 预期结果
    /// - Debug 格式化输出为 "Handlebars"
    /// - Clone 和 Copy 操作成功
    /// - 所有 trait 实现正常工作
    #[test]
    fn test_template_engine_type_return_ok() -> Result<()> {
        // Arrange: 准备测试枚举值
        let engine_type = TemplateEngineType::Handlebars;

        // Assert: 验证 Debug 实现
        let debug_str = format!("{:?}", engine_type);
        assert_eq!(debug_str, "Handlebars");

        // Arrange: 准备测试 Clone 实现
        let cloned_type = engine_type;
        assert!(matches!(cloned_type, TemplateEngineType::Handlebars));

        // Arrange: 准备测试 Copy 实现
        let copied_type = engine_type;
        assert!(matches!(copied_type, TemplateEngineType::Handlebars));
        Ok(())
    }

    /// 测试复杂模板场景
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine 能够正确处理包含条件语句、循环和嵌套变量的复杂模板。
    ///
    /// ## 测试场景
    /// 1. 创建包含条件（{{#if}}）、循环（{{#each}}）和嵌套变量的复杂模板
    /// 2. 准备包含嵌套结构的变量数据
    /// 3. 渲染模板并验证结果
    /// 4. 验证条件语句、循环和嵌套变量都正确渲染
    ///
    /// ## 预期结果
    /// - 复杂模板渲染成功
    /// - 嵌套变量正确替换（project.name、project.version）
    /// - 条件语句正确执行（显示 admin 信息）
    /// - 循环正确渲染所有项目特性
    #[test]
    fn test_complex_template_scenario_return_ok() -> Result<()> {
        let engine = TemplateEngine::new();

        // 创建复杂的模板，包含条件、循环和嵌套变量
        let complex_template = r#"
Project: {{project.name}} v{{project.version}}
{{#if user.profile.role}}
Admin: {{user.name}} ({{user.profile.email}})
{{/if}}

Features:
{{#each project.features}}
- {{this}}{{#unless @last}}
{{/unless}}{{/each}}

{{#if project.features}}
Total features: 3
{{else}}
No features available
{{/if}}"#;

        let vars = create_nested_variables();
        let result = engine.render_string(complex_template, &vars);
        assert!(result.is_ok());

        let rendered = result?;
        assert!(rendered.contains("Project: workflow-rs v1.0.0"));
        assert!(rendered.contains("Admin: Alice (alice@example.com)"));
        assert!(rendered.contains("- git"));
        assert!(rendered.contains("- jira"));
        assert!(rendered.contains("- pr"));
        Ok(())
    }

    /// 测试特殊字符处理
    ///
    /// ## 测试目的
    /// 验证 TemplateEngine 能够正确处理包含特殊字符（引号、尖括号、路径分隔符等）的变量值。
    ///
    /// ## 测试场景
    /// 1. 创建包含特殊字符的变量（引号、尖括号、路径、代码）
    /// 2. 使用这些变量渲染模板
    /// 3. 验证特殊字符在输出中正确保留（不被转义）
    ///
    /// ## 预期结果
    /// - 模板渲染成功
    /// - 特殊字符（引号、尖括号、路径分隔符）在输出中正确保留
    /// - 代码片段中的特殊字符也正确保留
    #[test]
    fn test_special_characters_handling_return_ok() -> Result<()> {
        let engine = TemplateEngine::new();

        // Arrange: 准备测试包含特殊字符的变量
        let vars = json!({
            "message": "Hello \"World\" & <Universe>!",
            "path": "/path/to/file.txt",
            "code": "if (x > 0) { return true; }"
        });

        let template = "Message: {{message}}\nPath: {{path}}\nCode: {{code}}";
        let result = engine.render_string(template, &vars);
        assert!(result.is_ok());

        let rendered = result?;
        // Assert: 验证特殊字符没有被转义（因为设置了 no_escape）
        assert!(rendered.contains("Hello \"World\" & <Universe>!"));
        assert!(rendered.contains("/path/to/file.txt"));
        assert!(rendered.contains("if (x > 0) { return true; }"));
        Ok(())
    }
}
