//! TemplateEngine 业务逻辑测试
//!
//! 测试 TemplateEngine 模块的核心功能，包括：
//! - 模板引擎创建和初始化
//! - 模板注册和渲染
//! - 变量替换和条件渲染
//! - 错误处理和边界情况

use pretty_assertions::assert_eq;
use color_eyre::Result;
use serde_json::json;
use std::collections::HashMap;
use workflow::template::{TemplateEngine, TemplateEngineType};

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

// ==================== 测试用例 ====================

/// 测试引擎创建
#[test]
fn test_new_engine_creation() -> Result<()> {
    let engine = TemplateEngine::new();

    // 验证引擎创建成功（通过尝试渲染一个简单模板）
    let result = engine.render_string("Hello {{name}}", &json!({"name": "World"}));
    assert!(result.is_ok());
    assert_eq!(result?, "Hello World");
Ok(())
}

/// 测试默认实现
#[test]
fn test_default_engine_creation() -> Result<()> {
    let engine = TemplateEngine::default();

    // 验证默认引擎与 new() 创建的引擎行为一致
    let result = engine.render_string("Test {{value}}", &json!({"value": "default"}));
    assert!(result.is_ok());
    assert_eq!(result?, "Test default");
Ok(())
}

/// 测试简单模板渲染
#[test]
fn test_render_simple_template() -> Result<()> {
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

/// 测试变量替换渲染
#[test]
fn test_render_with_variables() -> Result<()> {
    let engine = TemplateEngine::new();
    let vars = create_test_variables();

    // 测试基本变量替换
    let template = "Name: {{name}}, Age: {{age}}, Active: {{active}}";
    let result = engine.render_string(template, &vars);
    assert!(result.is_ok());
    assert_eq!(result?, "Name: John Doe, Age: 30, Active: true");

    // 测试嵌套变量访问
    let nested_vars = create_nested_variables();
    let nested_template =
        "User: {{user.name}}, Email: {{user.profile.email}}, Role: {{user.profile.role}}";
    let nested_result = engine.render_string(nested_template, &nested_vars);
    assert!(nested_result.is_ok());
    assert_eq!(
        nested_result?,
        "User: Alice, Email: alice@example.com, Role: admin"
    );
Ok(())
}

/// 测试条件渲染
#[test]
fn test_render_with_conditions() -> Result<()> {
    let engine = TemplateEngine::new();

    // 测试 if 条件
    let template = "{{#if active}}User is active{{else}}User is inactive{{/if}}";

    // 测试条件为 true
    let vars_true = json!({"active": true});
    let result_true = engine.render_string(template, &vars_true);
    assert!(result_true.is_ok());
    assert_eq!(result_true?, "User is active");

    // 测试条件为 false
    let vars_false = json!({"active": false});
    let result_false = engine.render_string(template, &vars_false);
    assert!(result_false.is_ok());
    assert_eq!(result_false?, "User is inactive");

    // 测试 unless 条件
    let unless_template = "{{#unless disabled}}Feature enabled{{else}}Feature disabled{{/unless}}";
    let vars_enabled = json!({"disabled": false});
    let result_enabled = engine.render_string(unless_template, &vars_enabled);
    assert!(result_enabled.is_ok());
    assert_eq!(result_enabled?, "Feature enabled");
Ok(())
}

/// 测试循环渲染
#[test]
fn test_render_with_loops() -> Result<()> {
    let engine = TemplateEngine::new();

    // 测试简单数组循环
    let template = "Items: {{#each items}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}";
    let vars = json!({"items": ["apple", "banana", "cherry"]});
    let result = engine.render_string(template, &vars);
    assert!(result.is_ok());
    assert_eq!(result?, "Items: apple, banana, cherry");

    // 测试对象数组循环
    let object_template =
        "{{#each users}}{{@index}}: {{name}} ({{role}}){{#unless @last}}\n{{/unless}}{{/each}}";
    let object_vars = json!({
        "users": [
            {"name": "Alice", "role": "admin"},
            {"name": "Bob", "role": "user"}
        ]
    });
    let object_result = engine.render_string(object_template, &object_vars);
    assert!(object_result.is_ok());
    assert_eq!(object_result?, "0: Alice (admin)\n1: Bob (user)");
Ok(())
}

/// 测试模板注册
#[test]
fn test_register_template() -> Result<()> {
    let mut engine = TemplateEngine::new();

    // 测试成功注册
    let result1 = engine.register_template("template1", "Hello {{name}}");
    assert!(result1.is_ok());

    let result2 = engine.register_template("template2", "Goodbye {{name}}");
    assert!(result2.is_ok());

    // 验证两个模板都能正常渲染
    let vars = json!({"name": "World"});

    let render1 = engine.render("template1", &vars);
    assert!(render1.is_ok());
    assert_eq!(render1?, "Hello World");

    let render2 = engine.render("template2", &vars);
    assert!(render2.is_ok());
    assert_eq!(render2?, "Goodbye World");

    // 测试覆盖已存在的模板
    let result3 = engine.register_template("template1", "Hi {{name}}!");
    assert!(result3.is_ok());

    let render3 = engine.render("template1", &vars);
    assert!(render3.is_ok());
    assert_eq!(render3?, "Hi World!");
Ok(())
}

/// 测试无效模板错误处理
#[test]
fn test_render_invalid_template() -> Result<()> {
    let mut engine = TemplateEngine::new();

    // 测试语法错误的模板
    let invalid_template = "Hello {{name"; // 缺少闭合括号
    let result = engine.register_template("invalid", invalid_template);
    assert!(result.is_err());

    // 测试渲染不存在的模板
    let vars = json!({"name": "World"});
    let render_result = engine.render("nonexistent", &vars);
    assert!(render_result.is_err());

    // 验证错误信息包含有用信息
    let error_msg = render_result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to render template"));
    assert!(error_msg.contains("nonexistent"));
    Ok(())
}

/// 测试缺失变量处理
#[test]
fn test_render_missing_variables() -> Result<()> {
    let engine = TemplateEngine::new();

    // 测试缺失变量（非严格模式下应该渲染为空字符串）
    let template = "Hello {{name}}, your age is {{age}}";
    let incomplete_vars = json!({"name": "Alice"}); // 缺少 age 变量

    let result = engine.render_string(template, &incomplete_vars);
    assert!(result.is_ok());
    assert_eq!(result?, "Hello Alice, your age is ");

    // 测试完全没有变量的情况
    let empty_vars = json!({});
    let empty_result = engine.render_string(template, &empty_vars);
    assert!(empty_result.is_ok());
    assert_eq!(empty_result?, "Hello , your age is ");
Ok(())
}

/// 测试 TemplateEngineType 枚举
#[test]
fn test_template_engine_type() -> Result<()> {
    // 测试枚举值
    let engine_type = TemplateEngineType::Handlebars;

    // 验证 Debug 实现
    let debug_str = format!("{:?}", engine_type);
    assert_eq!(debug_str, "Handlebars");

    // 测试 Clone 实现
    let cloned_type = engine_type.clone();
    assert!(matches!(cloned_type, TemplateEngineType::Handlebars));

    // 测试 Copy 实现
    let copied_type = engine_type;
    assert!(matches!(copied_type, TemplateEngineType::Handlebars));
Ok(())
}

/// 测试复杂模板场景
#[test]
fn test_complex_template_scenario() -> Result<()> {
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
#[test]
fn test_special_characters_handling() -> Result<()> {
    let engine = TemplateEngine::new();

    // 测试包含特殊字符的变量
    let vars = json!({
        "message": "Hello \"World\" & <Universe>!",
        "path": "/path/to/file.txt",
        "code": "if (x > 0) { return true; }"
    });

    let template = "Message: {{message}}\nPath: {{path}}\nCode: {{code}}";
    let result = engine.render_string(template, &vars);
    assert!(result.is_ok());

    let rendered = result?;
    // 验证特殊字符没有被转义（因为设置了 no_escape）
    assert!(rendered.contains("Hello \"World\" & <Universe>!"));
    assert!(rendered.contains("/path/to/file.txt"));
    assert!(rendered.contains("if (x > 0) { return true; }"));
Ok(())
}
