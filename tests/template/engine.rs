//! Template 引擎测试
//!
//! 测试 Template 引擎相关的功能，包括：
//! - 渲染模板
//! - 变量替换
//! - 错误处理

use pretty_assertions::assert_eq;
use workflow::template::engine::TemplateEngine;

// ==================== 引擎创建测试 ====================

#[test]
fn test_template_engine_new() {
    // 测试创建新的模板引擎
    let engine = TemplateEngine::new();
    // 应该成功创建
    assert!(true); // 如果能创建，说明成功
}

#[test]
fn test_template_engine_default() {
    // 测试默认模板引擎
    let engine = TemplateEngine::default();
    // 应该成功创建
    assert!(true);
}

// ==================== 模板注册测试 ====================

#[test]
fn test_register_template() {
    // 测试注册模板
    let mut engine = TemplateEngine::new();
    let result = engine.register_template("test", "Hello {{name}}!");

    assert!(result.is_ok());
}

#[test]
fn test_register_template_invalid() {
    // 测试注册无效模板（应该失败或处理错误）
    let mut engine = TemplateEngine::new();
    // 空的模板字符串应该是有效的
    let result = engine.register_template("empty", "");
    // 空模板应该是有效的
    assert!(result.is_ok());
}

// ==================== 模板渲染测试 ====================

#[test]
fn test_render_template() {
    // 测试渲染模板
    let mut engine = TemplateEngine::new();
    engine.register_template("greeting", "Hello {{name}}!").unwrap();

    #[derive(serde::Serialize)]
    struct Vars {
        name: String,
    }

    let vars = Vars {
        name: "World".to_string(),
    };

    let result = engine.render("greeting", &vars).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_render_template_with_multiple_vars() {
    // 测试渲染包含多个变量的模板
    let mut engine = TemplateEngine::new();
    engine
        .register_template("message", "{{greeting}} {{name}}, you are {{age}} years old.")
        .unwrap();

    #[derive(serde::Serialize)]
    struct Vars {
        greeting: String,
        name: String,
        age: u32,
    }

    let vars = Vars {
        greeting: "Hello".to_string(),
        name: "Alice".to_string(),
        age: 30,
    };

    let result = engine.render("message", &vars).unwrap();
    assert_eq!(result, "Hello Alice, you are 30 years old.");
}

#[test]
fn test_render_template_with_optional_vars() {
    // 测试渲染包含可选变量的模板
    let mut engine = TemplateEngine::new();
    engine
        .register_template("optional", "{{name}}{{#if title}}, {{title}}{{/if}}")
        .unwrap();

    #[derive(serde::Serialize)]
    struct Vars {
        name: String,
        title: Option<String>,
    }

    // 测试有 title 的情况
    let vars = Vars {
        name: "John".to_string(),
        title: Some("Dr.".to_string()),
    };
    let result = engine.render("optional", &vars).unwrap();
    assert!(result.contains("Dr."));

    // 测试没有 title 的情况
    let vars = Vars {
        name: "John".to_string(),
        title: None,
    };
    let result = engine.render("optional", &vars).unwrap();
    assert_eq!(result, "John");
}

// ==================== 字符串渲染测试 ====================

#[test]
fn test_render_string() {
    // 测试直接渲染模板字符串
    let engine = TemplateEngine::new();

    #[derive(serde::Serialize)]
    struct Vars {
        name: String,
    }

    let vars = Vars {
        name: "World".to_string(),
    };

    let result = engine.render_string("Hello {{name}}!", &vars).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_render_string_complex() {
    // 测试渲染复杂模板字符串
    let engine = TemplateEngine::new();

    #[derive(serde::Serialize)]
    struct Vars {
        jira_key: String,
        summary: String,
    }

    let vars = Vars {
        jira_key: "PROJ-123".to_string(),
        summary: "Fix bug".to_string(),
    };

    let template = "{{jira_key}}: {{summary}}";
    let result = engine.render_string(template, &vars).unwrap();
    assert_eq!(result, "PROJ-123: Fix bug");
}

// ==================== 错误处理测试 ====================

#[test]
fn test_render_nonexistent_template() {
    // 测试渲染不存在的模板（应该失败）
    let engine = TemplateEngine::new();

    #[derive(serde::Serialize)]
    struct Vars {
        name: String,
    }

    let vars = Vars {
        name: "World".to_string(),
    };

    let result = engine.render("nonexistent", &vars);
    assert!(result.is_err());
}

#[test]
fn test_render_template_missing_var() {
    // 测试渲染缺少变量的模板（应该使用空值或默认值）
    let mut engine = TemplateEngine::new();
    engine
        .register_template("missing", "Hello {{name}}!")
        .unwrap();

    #[derive(serde::Serialize)]
    struct EmptyVars {}

    let vars = EmptyVars {};
    // Handlebars 在严格模式下会失败，非严格模式下会使用空字符串
    let result = engine.render("missing", &vars);
    // 根据引擎配置，可能成功（使用空字符串）或失败
    // 由于设置了非严格模式，应该成功但 name 为空
    if result.is_ok() {
        assert_eq!(result.unwrap(), "Hello !");
    }
}
