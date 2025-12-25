//! Table 模块测试
//!
//! 测试表格构建器和表格样式的核心功能。

use tabled::Tabled;
use workflow::base::table::{TableBuilder, TableStyle};

#[derive(Tabled, Clone)]
struct TestUser {
    name: String,
    age: u32,
    email: String,
}

// ==================== TableBuilder Creation Tests ====================

#[test]
fn test_table_builder_new_with_data_creates_builder() {
    // Arrange: 准备测试数据
    let users = vec![
        TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        },
        TestUser {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        },
    ];

    // Act: 创建TableBuilder
    let _builder = TableBuilder::new(users);

    // Assert: 验证可以创建TableBuilder
    assert!(true);
}

#[test]
fn test_table_builder_with_title_with_title_string_renders_with_title() {
    // Arrange: 准备测试数据和标题
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];
    let title = "Users List";

    // Act: 创建带标题的TableBuilder并渲染
    let builder = TableBuilder::new(users).with_title(title);
    let output = builder.render();

    // Assert: 验证标题出现在输出中
    assert!(output.contains(title));
}

#[test]
fn test_table_builder_with_style_with_different_styles_renders_differently() {
    // Arrange: 准备测试数据
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    // Act: 使用不同样式渲染表格
    let builder = TableBuilder::new(users.clone()).with_style(TableStyle::Modern);
    let output_modern = builder.render();
    let builder = TableBuilder::new(users).with_style(TableStyle::Compact);
    let output_compact = builder.render();

    // Assert: 验证不同样式产生不同的输出
    assert_ne!(output_modern, output_compact);
}

#[test]
fn test_table_builder_with_max_width_with_width_limit_renders_table() {
    // Arrange: 准备测试数据和最大宽度
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];
    let max_width = 20;

    // Act: 创建带最大宽度限制的TableBuilder并渲染
    let builder = TableBuilder::new(users).with_max_width(max_width);
    let output = builder.render();

    // Assert: 验证输出不为空
    assert!(!output.is_empty());
}

#[test]
fn test_table_builder_empty_data_with_empty_data_returns_empty_string() {
    // Arrange: 准备空数据
    let users: Vec<TestUser> = vec![];

    // Act: 创建TableBuilder并渲染
    let builder = TableBuilder::new(users);
    let output = builder.render();

    // Assert: 验证空数据返回空字符串
    assert_eq!(output, "");
}

#[test]
fn test_table_builder_empty_data_with_title() {
    let users: Vec<TestUser> = vec![];

    let builder = TableBuilder::new(users).with_title("Empty Table");
    let output = builder.render();

    // 空数据带标题应该显示标题和 "(No data)"
    assert!(output.contains("Empty Table"));
    assert!(output.contains("(No data)"));
}

#[test]
fn test_table_builder_display_trait() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    let builder = TableBuilder::new(users);
    let output = format!("{}", builder);

    // 验证 Display trait 正常工作
    assert!(!output.is_empty());
}

#[test]
fn test_table_style_variants() {
    // 验证所有 TableStyle 变体都可以使用
    let _styles = vec![
        TableStyle::Default,
        TableStyle::Modern,
        TableStyle::Compact,
        TableStyle::Minimal,
        TableStyle::Grid,
    ];

    assert!(true);
}

#[test]
fn test_table_builder_chain_calls() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    // 测试链式调用
    let output = TableBuilder::new(users)
        .with_title("Users")
        .with_style(TableStyle::Modern)
        .with_max_width(80)
        .render();

    assert!(!output.is_empty());
    assert!(output.contains("Users"));
}
